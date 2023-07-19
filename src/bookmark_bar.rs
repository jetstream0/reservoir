use std::collections::HashMap;

use iced::Element;
use iced::{ alignment, Length };
use iced::widget::{ button, pick_list, container, text_input, column, row };

use crate::storage::{ Storage, Bookmark };
use crate::utils::normalize_link;

//contains bookmark search and adding

#[derive(Clone, Debug, PartialEq)]
pub enum BarMessage {
  ShowSearch,
  ShowAdd,
  Hide,
  SearchOptionChange(SearchOptions),
  SortOptionChange(SortOptions),
  InputSet(String, String),
  AddBookmark,
  //
}

impl BarMessage {
  pub fn is_save_after(message: BarMessage) -> bool {
    if message == BarMessage::AddBookmark {
      true
    } else {
      false
    }
  }

  pub fn is_search_update(message: BarMessage) -> bool {
    let _s: String = "search".to_string();
    if matches!(message, BarMessage::SearchOptionChange(_)) || matches!(message, BarMessage::SortOptionChange(_)) || matches!(message, BarMessage::InputSet(_s, _)) {
      true
    } else {
      false
    }
  }
}

#[derive(PartialEq)]
enum DisplayEnum {
  Search,
  Add,
  Neither,
}

pub struct BookmarkBar {
  display: DisplayEnum,
  bookmark_add: BookmarkAdd,
  pub bookmark_search: BookmarkSearch,
  pub input_values: HashMap<String, String>,
}

impl BookmarkBar {
  pub fn new() -> BookmarkBar {
    BookmarkBar {
      display: DisplayEnum::Neither,
      bookmark_add: BookmarkAdd::new(),
      bookmark_search: BookmarkSearch::new(),
      input_values: HashMap::new(),
    }
  }

  pub fn update(&mut self, message: BarMessage, storage: &mut Storage) {
    match message {
      BarMessage::ShowSearch => {
        self.display = DisplayEnum::Search;
      },
      BarMessage::ShowAdd => {
        self.display = DisplayEnum::Add;
      },
      BarMessage::Hide => {
        self.display = DisplayEnum::Neither;
        //reset
        self.input_values = HashMap::new();
      },
      BarMessage::InputSet(input_name, value) => {
        self.input_values.insert(input_name, value);
      },
      BarMessage::AddBookmark => {
        //required
        let empty_string: String = "".to_string();
        let title: String = self.input_values.get("title").unwrap_or(&empty_string).to_owned();
        let link: String = normalize_link(self.input_values.get("link").unwrap_or(&empty_string).to_owned());
        let already_exists = storage.stored.as_ref().unwrap().bookmarks.values().any(|bookmark| {
          bookmark.link == link
        });
        if already_exists {
          return;
        }
        if title == "".to_string() || link == "".to_string() {
          return;
        }
        //optional
        let mut note: Option<String> = self.input_values.get("note").cloned();
        if note.is_some() {
          if note.as_ref().unwrap() == "" {
            note = None;
          }
        }
        let tags_value: &String = self.input_values.get("tags").unwrap_or(&empty_string);
        let tags: Vec<String>;
        if tags_value.trim() == &empty_string {
          tags = Vec::new();
        } else {
          tags = tags_value.split(",").map(|item| item.to_string()).collect();
        }
        storage.add_bookmark(Bookmark::new(title, link, note, tags, None));
        //reset
        self.input_values = HashMap::new();
      },
      BarMessage::SearchOptionChange(new_search_option) => {
        self.bookmark_search.search_option = new_search_option;
      },
      BarMessage::SortOptionChange(new_sort_option) => {
        self.bookmark_search.sort_option = new_sort_option;
      },
    }
  }

  pub fn view(&self) -> Element<BarMessage> {
    if self.display == DisplayEnum::Add {
      column![
        container(row![
          button("Show Search").on_press(BarMessage::ShowSearch),
          button("Hide New Bookmark").on_press(BarMessage::Hide),
        ].spacing(5)).width(Length::Fill).align_x(alignment::Horizontal::Center),
        self.bookmark_add.view(&self.input_values),
      ].spacing(8).padding([10, 20]).into()
    } else if self.display == DisplayEnum::Search {
      column![
        container(row![
          button("Hide Search").on_press(BarMessage::Hide),
          button("Show New Bookmark").on_press(BarMessage::ShowAdd),
        ].spacing(5)).width(Length::Fill).align_x(alignment::Horizontal::Center),
        self.bookmark_search.view(&self.input_values),
      ].spacing(8).padding([10, 20]).into()
    } else {
      //
      container(row![
        button("Show Search").on_press(BarMessage::ShowSearch),
        button("Show New Bookmark").on_press(BarMessage::ShowAdd),
      ].spacing(5).padding([10, 20])).width(Length::Fill).align_x(alignment::Horizontal::Center).into()
    }
  }
}

trait Options where Self: Sized {
  fn all() -> Vec<Self>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SearchOptions {
  All,
  Title,
  Link,
  Tags,
}

impl Options for SearchOptions {
  fn all() -> Vec<SearchOptions> {
    vec![SearchOptions::All, SearchOptions::Title, SearchOptions::Link, SearchOptions::Tags]
  }
}

impl std::fmt::Display for SearchOptions {
  fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let content: String = match self {
      SearchOptions::All => "Search All".to_string(),
      SearchOptions::Title => "Search Title".to_string(),
      SearchOptions::Link => "Search Link".to_string(),
      SearchOptions::Tags => "Search Tags".to_string(),
    };
    write!(formatter, "{}", content)
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SortOptions {
  Relevant,
  Newest,
  Oldest
}

impl Options for SortOptions {
  fn all() -> Vec<SortOptions> {
    vec![SortOptions::Relevant, SortOptions::Newest, SortOptions::Oldest]
  }
}

impl std::fmt::Display for SortOptions {
  fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let content: String = match self {
      SortOptions::Relevant => "Sort Relevant".to_string(),
      SortOptions::Newest => "Sort Newest".to_string(),
      SortOptions::Oldest => "Sort Oldest".to_string(),
    };
    write!(formatter, "{}", content)
  }
}

pub struct BookmarkSearch {
  pub search_option: SearchOptions,
  pub sort_option: SortOptions,
}

impl BookmarkSearch {
  pub fn new() -> BookmarkSearch {
    BookmarkSearch {
      search_option: SearchOptions::All,
      sort_option: SortOptions::Relevant,
    }
  }

  pub fn view(&self, input_values: &HashMap<String, String>) -> Element<BarMessage> {
    row![
      pick_list(SearchOptions::all(), Some(self.search_option), BarMessage::SearchOptionChange),
      pick_list(SortOptions::all(), Some(self.sort_option), BarMessage::SortOptionChange),
      text_input("Search Query", input_values.get("search").unwrap_or(&"".to_string())).on_input(|value| BarMessage::InputSet("search".to_string(), value)),
      //button("Search"),
    ].spacing(5).into()
  }
}

//this is the hidden bookmark add thing that only pops up after button is clicked
pub struct BookmarkAdd;

impl BookmarkAdd {
  pub fn new() -> BookmarkAdd {
    BookmarkAdd
  }

  pub fn view(&self, input_values: &HashMap<String, String>) -> Element<BarMessage> {
    row![
      text_input("Title", input_values.get("title").unwrap_or(&"".to_string())).on_input(|value| BarMessage::InputSet("title".to_string(), value)),
      text_input("Link", input_values.get("link").unwrap_or(&"".to_string())).on_input(|value| BarMessage::InputSet("link".to_string(), value)),
      text_input("Note", input_values.get("note").unwrap_or(&"".to_string())).on_input(|value| BarMessage::InputSet("note".to_string(), value)),
      text_input("Tags (CSV)", input_values.get("tags").unwrap_or(&"".to_string())).on_input(|value| BarMessage::InputSet("tags".to_string(), value)),
      button("Add").on_press(BarMessage::AddBookmark),
    ].spacing(5).into()
  }
}
