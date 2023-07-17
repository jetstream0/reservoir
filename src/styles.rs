use iced::{ Background, Color, Theme, theme };
use iced::widget::{ button, container };

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

pub const BOOKMARK_TIMESTAMP_STYLE: theme::Text = theme::Text::Color(Color::from_rgb(00.5, 0.5, 0.5));
