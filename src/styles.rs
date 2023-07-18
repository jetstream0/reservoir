use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use iced::{ Background, Color, Theme, theme };
use iced::widget::{ button, container };
//use iced::overlay::Element;

#[derive(Default)]
pub struct BookmarkContainer;

impl container::StyleSheet for BookmarkContainer {
  type Style = Theme;

  fn appearance(&self, _style: &Self::Style) -> container::Appearance {
    container::Appearance {
      border_radius: 15.0,
      border_color: Color::from_rgb8(255, 255, 255),
      border_width: 2.0,
      ..container::Appearance::default()
    }
  }
}

pub struct TagButton {
  pub text: String,
}

impl TagButton {
  fn text_to_color(&self) -> [u8; 3] {
    let mut hasher = DefaultHasher::new();
    self.text.hash(&mut hasher);
    let bytes: [u8; 8] = hasher.finish().to_le_bytes();
    const THRESHOLD: u8 = 90;
    let mut return_rgb: [u8; 3];
    if bytes[0] < THRESHOLD && bytes[1] < THRESHOLD && bytes[2] < THRESHOLD {
      //too dark
      const ADD: u8 = 255 - THRESHOLD;
      if bytes[0] > 60 {
        return_rgb = [bytes[0]+ADD, bytes[1], bytes[2]];
      } else if bytes[0] > 30 {
        return_rgb = [bytes[0], bytes[1]+ADD, bytes[2]];
      } else {
        return_rgb = [bytes[0], bytes[1], bytes[2]+ADD];
      }
    } else {
      return_rgb = [bytes[0], bytes[1], bytes[2]];
    }
    if return_rgb[2] > 100 && return_rgb[0] < 45 && return_rgb[1] < 45 {
      return_rgb = [return_rgb[0]+40, return_rgb[1]+40, return_rgb[2]-30]
    }
    return_rgb
  }
}

impl button::StyleSheet for TagButton {
  type Style = Theme;

  fn active(&self, _style: &Self::Style) -> button::Appearance {
    let rgb = self.text_to_color();
    button::Appearance {
      border_radius: 15.0,
      border_width: 1.0,
      border_color: Color::from_rgb8(rgb[0], rgb[1], rgb[2]),
      background: Some(Background::Color(Color::from_rgba8(rgb[0], rgb[1], rgb[2], 0.3))),
      text_color: Color::from_rgb8(rgb[0], rgb[1], rgb[2]),
      ..button::Appearance::default()
    }
  }
}

pub const BOOKMARK_TIMESTAMP_STYLE: theme::Text = theme::Text::Color(Color::from_rgb(00.5, 0.5, 0.5));
