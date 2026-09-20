use cosmic::iced::font::{Family, Font, Weight};
use std::borrow::Cow;

pub const ROBOTO_REGULAR_BYTES: &[u8] = include_bytes!("../../resources/fonts/Roboto-Regular.ttf");
pub const ROBOTO_BOLD_BYTES: &[u8] = include_bytes!("../../resources/fonts/Roboto-Bold.ttf");
pub const JETBRAINS_REGULAR_BYTES: &[u8] = include_bytes!("../../resources/fonts/JetBrainsMono-Regular.ttf");
pub const JETBRAINS_BOLD_BYTES: &[u8] = include_bytes!("../../resources/fonts/JetBrainsMono-Bold.ttf");

pub const FONT_ROBOTO_REGULAR: Font = Font {
    family: Family::Name("Roboto"),
    weight: Weight::Normal,
    ..Font::DEFAULT
};

pub const FONT_ROBOTO_BOLD: Font = Font {
    family: Family::Name("Roboto"),
    weight: Weight::Bold,
    ..Font::DEFAULT
};

pub const FONT_MONO_REGULAR: Font = Font {
    family: Family::Name("JetBrains Mono"),
    weight: Weight::Normal,
    ..Font::DEFAULT
};

pub const FONT_MONO_BOLD: Font = Font {
    family: Family::Name("JetBrains Mono"),
    weight: Weight::Bold,
    ..Font::DEFAULT
};

/// Returns all embedded fonts to be loaded into Iced at startup
pub fn embedded_fonts() -> Vec<Cow<'static, [u8]>> {
    vec![
        Cow::Borrowed(ROBOTO_REGULAR_BYTES),
        Cow::Borrowed(ROBOTO_BOLD_BYTES),
        Cow::Borrowed(JETBRAINS_REGULAR_BYTES),
        Cow::Borrowed(JETBRAINS_BOLD_BYTES),
    ]
}
