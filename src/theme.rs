use bevy::prelude::*;
use bevy::text::LineHeight;

pub mod font {
    pub const BASIC_SIZE: f32 = 14.0;
    pub const TITLE_SIZE: f32 = BASIC_SIZE + 2.0;
    pub const SMALL_SIZE: f32 = BASIC_SIZE - 2.0;
    pub const LARGE_SIZE: f32 = BASIC_SIZE + 6.0;
}

pub mod color {
    use bevy::prelude::*;
    pub const TEXT: Color = Color::WHITE;
    pub const BACKGROUND: Color = Color::BLACK;
}

pub fn title_font() -> TextFont {
    TextFont::default()
        .with_font_size(font::TITLE_SIZE)
        .with_line_height(LineHeight::RelativeToFont(2.0))
}

pub fn basic_font() -> TextFont {
    TextFont::default().with_font_size(font::BASIC_SIZE)
}

pub fn background_color() -> BackgroundColor {
    BackgroundColor(color::BACKGROUND.with_alpha(0.7))
}

pub fn text_color() -> TextColor {
    TextColor(color::TEXT)
}
