use std::collections::HashMap;

use iced::Element;
use iced::{ alignment, Length, theme };
use iced::widget::{ button, container, text, text_input, scrollable, row, column, Column };

use webbrowser;

use crate::storage::{ Bookmark, Storage };
use crate::bookmark_bar::Filter;
use crate::utils::timestamp_to_string;
use crate::styles;

#[derive(Clone, Debug)]
pub enum ListMessage {
  ExpandBookmark(String),
  UnexpandBookmark(String),
  EditBookmark(String),
  IgnoreEditBookmark(String),
  SaveEditBookmark(String, Bookmark),
  DeleteBookmark(String),
  OpenLink(String),
  InputSet(String, String),
  UpdateFilter(Option<Filter>),
}

impl ListMessage {
  pub fn is_save_after(message: ListMessage) -> bool {
    if matches!(message, ListMessage::SaveEditBookmark(_, _)) || matches!(message, ListMessage::DeleteBookmark(_)) {
      true
    } else {
      false
    }
  }
}

pub struct BookmarkList {
  filter: Option<Filter>,
  expand_uuids: Vec<String>,
  edit_uuids: Vec<String>,
  input_values: HashMap<String, String>,
}

impl BookmarkList {
  const ITEM_PADDING: [u16; 2] = [15, 15];

  pub fn new() -> BookmarkList {
    BookmarkList {
      filter: None,
      expand_uuids: Vec::new(),
      edit_uuids: Vec::new(),
      input_values: HashMap::new(),
    }
  }

  pub fn update(&mut self, message: ListMessage, storage: &mut Storage) {
    match message {
      ListMessage::ExpandBookmark(uuid_value) => {
        if !self.expand_uuids.contains(&uuid_value) {
          self.expand_uuids.push(uuid_value);
        }
      },
      ListMessage::UnexpandBookmark(uuid_value) => {
        self.expand_uuids.retain(|value| value != &uuid_value);
      },
      ListMessage::EditBookmark(uuid_value) => {
        //remove from expand list, because you can click edit button from expanded state
        if self.expand_uuids.contains(&uuid_value) {
          self.expand_uuids.retain(|value| value != &uuid_value);
        }
        //add to edit
        if !self.edit_uuids.contains(&uuid_value) {
          self.edit_uuids.push(uuid_value.clone());
        }
        //reset edit fields or something idk
        self.input_values.retain(|key, _| {
          !key.starts_with(&format!("{}-", uuid_value))
        });
      },
      ListMessage::IgnoreEditBookmark(uuid_value) => {
        self.edit_uuids.retain(|value| value != &uuid_value);
        //reset edit fields
        self.input_values.retain(|key, _| {
          !key.starts_with(&format!("{}-", uuid_value))
        });
      },
      ListMessage::SaveEditBookmark(uuid_value, bookmark) => {
        let mut bookmark = bookmark.clone();
        //title
        let title_key: String = format!("{}-title", &bookmark.uuid);
        if self.input_values.get(&title_key).is_some() {
          let temp_title: String = self.input_values.get(&title_key).unwrap().to_string();
          if temp_title != "".to_string() {
            bookmark.title = temp_title;
          }
        }
        //link
        let link_key: String = format!("{}-link", &bookmark.uuid);
        if self.input_values.get(&link_key).is_some() {
          let temp_link: String = self.input_values.get(&link_key).unwrap().to_string();
          if temp_link != "".to_string() {
            bookmark.title = temp_link;
          }
        }
        //tags
        let tags_key: String = format!("{}-tags", &uuid_value);
        if self.input_values.get(&tags_key).is_some() {
          bookmark.tags = self.input_values.get(&tags_key).unwrap().split(",").map(|item| item.to_string()).collect();
        }
        //note
        let note_key: String = format!("{}-note", &uuid_value);
        if self.input_values.get(&note_key).is_none() && bookmark.note.is_none() {
          bookmark.note = None;
        } else if self.input_values.get(&note_key).is_some() {
          let temp_note: String = self.input_values.get(&note_key).unwrap().to_string();
          if temp_note == "".to_string() {
            bookmark.note = None;
          } else {
            bookmark.note = Some(temp_note);
          }
        } else { //bookmark.note.is_some()
          bookmark.note = Some(bookmark.note.unwrap());
        }
        //
        storage.add_bookmark(bookmark);
        self.edit_uuids.retain(|value| value != &uuid_value);
      },
      ListMessage::DeleteBookmark(uuid_value) => {
        storage.remove_bookmark(uuid_value);
      },
      ListMessage::OpenLink(link) => {
        let with_https: String = format!("https://{}", link);
        let open_result = webbrowser::open(if !link.starts_with("https://") && !link.starts_with("http://") { &with_https } else { &link });
        match open_result {
          Ok(()) => {},
          Err(error) => {
            dbg!("{:?}", error);
          },
        }
      },
      ListMessage::InputSet(input_name, value) => {
        self.input_values.insert(input_name, value);
      },
      ListMessage::UpdateFilter(new_filter) => {
        self.filter = new_filter;
      },
    }
  }

  pub fn view(&self, bookmarks: &HashMap<String, Bookmark>) -> Element<ListMessage> {
    let bookmarks_show = bookmarks.values();
    if self.filter.is_some() {
      //filter stuff
      //
    }
    //now display
    let mut bookmark_elements: Vec<Element<ListMessage>> = Vec::new();
    for bookmark in bookmarks_show {
      if self.expand_uuids.contains(&bookmark.uuid) {
        bookmark_elements.push(
          container(
            column![
              row![
                row![
                  text(&bookmark.title),
                  text(&bookmark.link),
                ].width(Length::FillPortion(4)).spacing(5),
                container(row![
                  button("Unexpand").on_press(ListMessage::UnexpandBookmark(bookmark.uuid.clone())),
                  button("Edit").on_press(ListMessage::EditBookmark(bookmark.uuid.clone())),
                  button("Open").on_press(ListMessage::OpenLink(bookmark.link.clone())),
                ].width(Length::FillPortion(1)).spacing(5)).align_x(alignment::Horizontal::Right),
              ],
              row![
                text(&timestamp_to_string(bookmark.timestamp)).style(styles::BOOKMARK_TIMESTAMP_STYLE),
                text(&bookmark.tags.join(", ")),
              ].spacing(5),
              row![
                if bookmark.note.is_some() { text(&bookmark.note.as_ref().unwrap()) } else { text("No note") },
              ]
            ]
          ).padding(BookmarkList::ITEM_PADDING).style(theme::Container::Custom(Box::new(styles::BookmarkContainer))).into()
        );
      } else if self.edit_uuids.contains(&bookmark.uuid) {
        let title_key: String = format!("{}-title", &bookmark.uuid);
        let link_key: String = format!("{}-link", &bookmark.uuid);
        let tags_key: String = format!("{}-tags", &bookmark.uuid);
        let note_key: String = format!("{}-note", &bookmark.uuid);
        //
        bookmark_elements.push(
          container(
            column![
              row![
                row![
                  text_input("Title", self.input_values.get(&title_key).unwrap_or(&bookmark.title)).on_input(move |value| ListMessage::InputSet(title_key.clone(), value)),
                  text_input("Link", self.input_values.get(&link_key).unwrap_or(&bookmark.link)).on_input(move |value| ListMessage::InputSet(link_key.clone(), value)),
                ].width(Length::FillPortion(4)).spacing(5),
                container(row![
                  button("Cancel Edit").on_press(ListMessage::IgnoreEditBookmark(bookmark.uuid.clone())),
                  button("Save").on_press(ListMessage::SaveEditBookmark(bookmark.uuid.clone(), bookmark.clone())),
                  button("Delete").on_press(ListMessage::DeleteBookmark(bookmark.uuid.clone())).style(theme::Button::Destructive),
                ].width(Length::FillPortion(1)).spacing(5)).align_x(alignment::Horizontal::Right),
              ],
              row![
                text(&timestamp_to_string(bookmark.timestamp)).style(styles::BOOKMARK_TIMESTAMP_STYLE),
                text_input("Tags", self.input_values.get(&tags_key).unwrap_or(&bookmark.tags.join(","))).on_input(move |value| ListMessage::InputSet(tags_key.clone(), value)),
              ].spacing(5),
              row![
                text_input("Note", self.input_values.get(&note_key).unwrap_or(&bookmark.note.as_ref().unwrap_or(&"".to_string()))).on_input(move |value| ListMessage::InputSet(note_key.clone(), value)),
              ]
            ].spacing(5)
          ).padding(BookmarkList::ITEM_PADDING).style(theme::Container::Custom(Box::new(styles::BookmarkContainer))).into()
        );
        //
      } else {
        bookmark_elements.push(
          container(
            row![
              row![
                text(&bookmark.title),
                text(&bookmark.link),
              ].width(Length::FillPortion(4)).spacing(5),
              container(row![
                button("Expand").on_press(ListMessage::ExpandBookmark(bookmark.uuid.clone())),
                button("Edit").on_press(ListMessage::EditBookmark(bookmark.uuid.clone())),
                button("Open").on_press(ListMessage::OpenLink(bookmark.link.clone())),
              ].width(Length::FillPortion(1)).spacing(5)).align_x(alignment::Horizontal::Right),
            ]
          ).padding(BookmarkList::ITEM_PADDING).style(theme::Container::Custom(Box::new(styles::BookmarkContainer))).into()
        );
      }
    }
    scrollable(container(Column::with_children(bookmark_elements).spacing(10)).padding([10, 20])).into()
  }
}
