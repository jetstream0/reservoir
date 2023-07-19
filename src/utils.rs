use std::time::{ SystemTime, UNIX_EPOCH, Duration };

use uuid::Uuid;
use chrono::prelude::DateTime;
use chrono::Local;

pub fn gen_uuid() -> String {
  let random_uuid: Uuid = Uuid::new_v4(); 
  return random_uuid.hyphenated().encode_lower(&mut Uuid::encode_buffer()).to_string();
}

pub fn get_timestamp() -> u64 {
  SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
}

pub fn timestamp_to_string(timestamp: u64) -> String {
  //if add overflows, default to unix epoch - hey, better than crashing!
  let timestamp_time: SystemTime = UNIX_EPOCH.checked_add(Duration::from_secs(timestamp)).unwrap_or(UNIX_EPOCH);
  let local_datetime: DateTime<Local> = DateTime::<Local>::from(timestamp_time);
  local_datetime.format("%d/%m/%Y %H:%M").to_string()
}

pub fn normalize_link(link: String) -> String {
  let mut link = link;
  if link.starts_with("https://") {
    link = link.replacen("https://", "", 1);
  }
  link
}

pub fn truncate_with_ellipses(input: &str, max_length: usize) -> String {
  if input.len() > max_length {
    format!("{}...", &input[..max_length])
  } else {
    input.to_string()
  }
}
