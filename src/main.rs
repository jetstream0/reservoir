#![windows_subsystem = "windows"]

//use iced::futures::FutureExt;
use iced::{ Application, Element };
use iced::{ Command, Settings, window, subscription, Subscription };
use iced::theme::Theme;
use iced::widget::{ container, column };
use image::ImageFormat;

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
}

#[derive(Clone, Debug)]
enum AppMessage {
  Loaded(Result<Stored, StorageError>),
  BarMessage(BarMessage),
  ListMessage(ListMessage),
  SaveDone(Result<(), StorageError>),
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
        }
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
        } else {
          if BarMessage::is_search_update(message) {
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
      //
      AppMessage::SizeChange(width, height) => {
        self.window_size = WindowSize { 
          width,
          height
        };
        Command::none()
      },
      _ => Command::none(),
    }
  }

  //"view called when state is modified"
  fn view(&self) -> Element<'_, Self::Message> {
    println!("Rerendering");
    if self.loaded {
      column![
        self.bookmark_bar.view().map(move |message| {
          Self::Message::BarMessage(message)
        }),
        self.bookmark_list.view(&self.storage.stored.as_ref().unwrap().bookmarks, &self.window_size).map(move |message| {
          Self::Message::ListMessage(message)
        }),
      ].into()
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
