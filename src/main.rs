#![windows_subsystem = "windows"]

use std::time::Duration;

//use iced::futures::FutureExt;
use iced::{ Application, Element };
use iced::{ alignment, Command, Length, Settings, subscription, Subscription, window };
use iced::theme::Theme;
use iced::widget::{ container, column, text };
use image::ImageFormat;

use async_std::task;

mod utils;

mod styles;

mod storage;
use storage::{ Stored, StorageError, Storage };

mod bookmark_bar;
use bookmark_bar::{ BarMessage, BookmarkBar, SearchOptions };

mod bookmark_list;
use bookmark_list::{ ListMessage, BookmarkList };

fn main() -> iced::Result {
  App::run(Settings {
    window: window::Settings {
      size: (920, 600),
      min_size: Some((575, 250)),
      icon: Some(window::icon::from_file_data(include_bytes!("icon.png"), Some(ImageFormat::Png)).unwrap()),
      ..window::Settings::default()
    },
    ..Settings::default()
  })
}

pub struct WindowSize {
  pub width: u32,
  pub height: u32,
}

struct App {
  pub storage: Storage,
  loaded: bool,
  bookmark_list: BookmarkList,
  bookmark_bar: BookmarkBar,
  window_size: WindowSize,
  save_message_count: u16,
  save_message: bool,
}

#[derive(Clone, Debug)]
enum AppMessage {
  Loaded(Result<Stored, StorageError>),
  BarMessage(BarMessage),
  ListMessage(ListMessage),
  SaveDone(Result<(), StorageError>),
  ExportDone(Result<(), StorageError>),
  HideExportDone(u16),
  SizeChange(u32, u32),
}

//all a big placeholder for now
impl Application for App {
  type Executor = iced::executor::Default;
  type Message = AppMessage;
  type Theme = Theme;
  type Flags = ();
  
  fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
    (
      App {
        storage: Storage::new(),
        loaded: false,
        bookmark_list: BookmarkList::new(),
        bookmark_bar: BookmarkBar::new(),
        window_size: WindowSize {
          width: 920,
          height: 600,
        },
        save_message: false,
        save_message_count: 0,
      },
      Command::perform(Storage::load(), Self::Message::Loaded),
    )
  }

  fn title(&self) -> String {
    "reservoir".to_string()
  }

  fn theme(&self) -> Theme {
    Self::Theme::Dark
  }

  fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
    match message {
      Self::Message::Loaded(Ok(stored)) => {
        self.storage.stored = Some(stored);
        self.loaded = true;
        Command::none()
      },
      Self::Message::BarMessage(message) => {
        self.bookmark_bar.update(message.clone(), &mut self.storage);
        if BarMessage::is_save_after(message.clone()) {
          //self.storage.save_sync();
          Command::perform(Storage::save_async_separate(self.storage.stored.as_ref().unwrap().to_owned()), AppMessage::SaveDone)
        } else if message == BarMessage::ExportAll {
          Command::perform(Storage::export(self.storage.stored.as_ref().unwrap().to_owned()), AppMessage::ExportDone)
        } else {
          if message == BarMessage::ExpandAll {
            self.bookmark_list.update(ListMessage::ExpandAll, &mut self.storage);
          } else if message == BarMessage::ShrinkAll {
            self.bookmark_list.update(ListMessage::ShrinkAll, &mut self.storage);
          } else if BarMessage::is_search_update(message) {
            self.bookmark_list.update(ListMessage::UpdateSearch(self.bookmark_bar.bookmark_search.search_option, self.bookmark_bar.bookmark_search.sort_option, self.bookmark_bar.input_values.get("search").cloned()), &mut self.storage);
          }
          Command::none()
        }
      },
      Self::Message::ListMessage(message) => {
        self.bookmark_list.update(message.clone(), &mut self.storage);
        if ListMessage::is_save_after(message.clone()) {
          //self.storage.save_sync();
          Command::perform(Storage::save_async_separate(self.storage.stored.as_ref().unwrap().to_owned()), AppMessage::SaveDone)
        } else {
          if let ListMessage::TagPress(tag) = message {
            self.bookmark_bar.update(BarMessage::ShowSearch, &mut self.storage);
            self.bookmark_bar.update(BarMessage::SearchOptionChange(SearchOptions::Tags), &mut self.storage);
            self.bookmark_bar.update(BarMessage::InputSet("search".to_string(), tag), &mut self.storage);
            self.bookmark_list.update(ListMessage::UpdateSearch(self.bookmark_bar.bookmark_search.search_option, self.bookmark_bar.bookmark_search.sort_option, self.bookmark_bar.input_values.get("search").cloned()), &mut self.storage);
          }
          Command::none()
        }
      },
      Self::Message::SaveDone(Err(error)) => {
        println!("{:?}", error);
        Command::none()
      },
      Self::Message::ExportDone(Ok(_)) => {
        self.save_message = true;
        self.save_message_count += 1;
        let save_message_count: u16 = self.save_message_count;
        Command::perform(task::sleep(Duration::from_secs(2)), move |_| Self::Message::HideExportDone(save_message_count))
      },
      Self::Message::HideExportDone(save_message_count) => {
        if save_message_count == self.save_message_count {
          self.save_message = false;
        }
        Command::none()
      },
      Self::Message::SizeChange(width, height) => {
        self.window_size = WindowSize { 
          width,
          height
        };
        Command::none()
      },
      _ => {
        Command::none()
      },
    }
  }

  //"view called when state is modified"
  fn view(&self) -> Element<'_, Self::Message> {
    //println!("Rerendering");
    if self.loaded {
      //something something DRY. don't care right now
      if self.save_message {
        column![
          self.bookmark_bar.view().map(move |message| {
            Self::Message::BarMessage(message)
          }),
          container(
            text("Exported!").horizontal_alignment(alignment::Horizontal::Center)
          ).width(Length::Fill).align_x(alignment::Horizontal::Center),
          self.bookmark_list.view(&self.storage.stored.as_ref().unwrap().bookmarks, &self.window_size).map(move |message| {
            Self::Message::ListMessage(message)
          }),
        ].into()
      } else {
        column![
          self.bookmark_bar.view().map(move |message| {
            Self::Message::BarMessage(message)
          }),
          self.bookmark_list.view(&self.storage.stored.as_ref().unwrap().bookmarks, &self.window_size).map(move |message| {
            Self::Message::ListMessage(message)
          }),
        ].into()
      }
    } else {
      container("Loading...").padding(5).into()
    }
  }

  fn subscription(&self) -> Subscription<Self::Message> {
    subscription::events_with(|event, _status| {
      match event {
        iced::Event::Window(window::Event::Resized { width, height }) => {
          Some(AppMessage::SizeChange(width, height))
        },
        _ => None,
      }
    })
  }
}
