use std::collections::HashMap;
use std::path::PathBuf;

use directories_next;
use serde_json;
use serde::{ Serialize, Deserialize };
use async_std::prelude::*;
use async_std::fs::{ create_dir_all, File };

use crate::utils::{ gen_uuid, get_timestamp };

#[derive(Clone, Debug)]
pub enum StorageError {
  CreateError,
  ReadError,
  OpenError,
  WriteError,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Bookmark {
  pub title: String,
  pub link: String,
  pub note: Option<String>,
  pub tags: Vec<String>,
  pub uuid: String,
  pub timestamp: u64,
}

impl Bookmark {
  pub fn new(title: String, link: String, note: Option<String>, tags: Vec<String>, timestamp: Option<u64>) -> Bookmark {
    Bookmark {
      title,
      link,
      note,
      tags,
      uuid: gen_uuid(),
      timestamp: if timestamp.is_some() { timestamp.unwrap() } else { get_timestamp() },
    }
  }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Stored {
  //key is link?
  pub bookmarks: HashMap<String, Bookmark>,
}

pub struct Storage {
  pub stored: Option<Stored>,
}

impl Storage {
  pub fn new() -> Storage {
    Storage {
      stored: None,
    }
  }

  fn path() -> PathBuf {
    let mut save_path: PathBuf;
    let project_dir = directories_next::ProjectDirs::from("rs", "prussiacorp", "reservoir");
    if project_dir.is_some() {
      save_path = project_dir.unwrap().data_dir().into();
    } else {
      save_path = std::env::current_dir().unwrap_or_default();
    }
    save_path.push("stored.json");
    save_path
  }

  fn empty_json() -> String {
    "{\n  \"bookmarks\": {}\n}".to_string()
  }

  pub async fn load() -> Result<Stored, StorageError> {
    let save_path = Storage::path();
    //todo: find some way to make this not mutable idk
    let mut save_file: File;
    let save_path_parent = save_path.parent().unwrap();
    if !save_path_parent.is_dir() {
      create_dir_all(save_path_parent).await.map_err(|_| StorageError::CreateError)?;
    }
    if !save_path.is_file() {
      //create file
      save_file = File::create(save_path.clone()).await.map_err(|_| StorageError::CreateError)?;
      save_file.write_all(
        Storage::empty_json().as_bytes()
      ).await.map_err(|_| StorageError::WriteError)?;
    }
    //we must open the file even if we just created it because of permission
    save_file = File::open(save_path).await.map_err(|_| StorageError::OpenError)?;
    let mut contents: String = String::new();
    if let Err(error) = save_file.read_to_string(&mut contents).await {
      println!("{:?}", error);
    }
    save_file.read_to_string(&mut contents).await.map_err(|_| StorageError::ReadError)?;
    Ok(serde_json::from_str(&contents).unwrap())
  }

  /*
  pub async fn save_async(&self) -> Result<(), StorageError> {
    let mut save_file: File = File::create(Storage::path()).await.map_err(|_| StorageError::OpenError)?;
    if self.stored.is_none() {
      save_file.write_all(Storage::empty_json().as_bytes()).await.map_err(|_| StorageError::WriteError)?;
    } else {
      save_file.write_all(serde_json::to_string_pretty(&self.stored.as_ref().unwrap()).unwrap().as_bytes()).await.map_err(|_| StorageError::WriteError)?;
    }
    Ok(())
  }
  */

  //would be nice to change to &Stored or some kind of pointer
  pub async fn save_async_separate(stored: Stored) -> Result<(), StorageError> {
    let mut save_file: File = File::create(Storage::path()).await.map_err(|_| StorageError::OpenError)?;
    save_file.write_all(serde_json::to_string_pretty(&stored).unwrap().as_bytes()).await.map_err(|_| StorageError::WriteError)?;
    Ok(())
  }

  //also do this for edit bookmark
  pub fn add_bookmark(&mut self, bookmark: Bookmark) {
    self.stored.as_mut().unwrap().bookmarks.insert((&bookmark.uuid).to_string(), bookmark);
  }

  pub fn remove_bookmark(&mut self, uuid: String) {
    self.stored.as_mut().unwrap().bookmarks.remove(&uuid);
  }

  //would be nice to change to &Stored or some kind of pointer
  pub async fn export(stored: Stored) -> Result<(), StorageError> {
    if let Some(user_dirs) = directories_next::UserDirs::new() {
      if let Some(download_path) = user_dirs.download_dir() {
        let mut save_path: PathBuf = download_path.into();
        save_path.push(format!("reservoir_info_{}.json", get_timestamp().to_string()));
        let mut save_file: File = File::create(save_path).await.map_err(|_| StorageError::OpenError)?;
        save_file.write_all(serde_json::to_string_pretty(&stored).unwrap().as_bytes()).await.map_err(|_| StorageError::WriteError)?;
      } else {
        return Err(StorageError::OpenError);
      }
    } else {
      return Err(StorageError::OpenError);
    }
    Ok(())
  }
}
