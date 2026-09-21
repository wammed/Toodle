use cosmic::iced::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeFontKind {
    RobotoSans,
    JetBrainsMono,
    DejaVuSerif,
    OpenSans,
}

impl ThemeFontKind {
    pub fn label(&self) -> &'static str {
        match self {
            Self::RobotoSans => "Roboto Sans",
            Self::JetBrainsMono => "JetBrains Mono",
            Self::DejaVuSerif => "DejaVu Serif",
            Self::OpenSans => "Open Sans",
        }
    }

    pub fn fonts(&self) -> (cosmic::iced::Font, cosmic::iced::Font) {
        match self {
            Self::RobotoSans => (
                crate::clock::fonts::FONT_ROBOTO_BOLD,
                crate::clock::fonts::FONT_ROBOTO_REGULAR,
            ),
            Self::JetBrainsMono => (
                crate::clock::fonts::FONT_MONO_BOLD,
                crate::clock::fonts::FONT_MONO_REGULAR,
            ),
            Self::DejaVuSerif => (
                crate::clock::fonts::FONT_SERIF_BOLD,
                crate::clock::fonts::FONT_SERIF_REGULAR,
            ),
            Self::OpenSans => (
                crate::clock::fonts::FONT_OPENSANS_BOLD,
                crate::clock::fonts::FONT_OPENSANS_REGULAR,
            ),
        }
    }
}

pub struct ThemePreset {
    pub name: &'static str,
    pub description: &'static str,
    pub default_color: &'static str,
    pub font_kind: ThemeFontKind,
    pub is_mono: bool,
}

pub const THEME_PRESETS: &[ThemePreset] = &[
    ThemePreset {
        name: "Modern",
        description: "Contemporary aesthetic with Roboto Sans",
        default_color: "#FFFFFF",
        font_kind: ThemeFontKind::RobotoSans,
        is_mono: false,
    },
    ThemePreset {
        name: "Classic",
        description: "Sophisticated timeless look with DejaVu Serif",
        default_color: "#E2E8F0",
        font_kind: ThemeFontKind::DejaVuSerif,
        is_mono: false,
    },
    ThemePreset {
        name: "Digital Mono",
        description: "Sleek terminal style with JetBrains Mono",
        default_color: "#38BDF8",
        font_kind: ThemeFontKind::JetBrainsMono,
        is_mono: true,
    },
    ThemePreset {
        name: "Minimal",
        description: "Clean understated design with Open Sans",
        default_color: "#94A3B8",
        font_kind: ThemeFontKind::OpenSans,
        is_mono: false,
    },
    ThemePreset {
        name: "Cyberpunk",
        description: "Futuristic high-tech look with JetBrains Mono",
        default_color: "#EAB308",
        font_kind: ThemeFontKind::JetBrainsMono,
        is_mono: true,
    },
    ThemePreset {
        name: "Nord",
        description: "Cool arctic blue tones with Open Sans",
        default_color: "#38BDF8",
        font_kind: ThemeFontKind::OpenSans,
        is_mono: false,
    },
    ThemePreset {
        name: "Warm Sunset",
        description: "Warm amber glow with DejaVu Serif",
        default_color: "#F59E0B",
        font_kind: ThemeFontKind::DejaVuSerif,
        is_mono: false,
    },
    ThemePreset {
        name: "Forest",
        description: "Natural emerald green with Roboto Sans",
        default_color: "#10B981",
        font_kind: ThemeFontKind::RobotoSans,
        is_mono: false,
    },
    ThemePreset {
        name: "Slate",
        description: "Muted metallic slate with JetBrains Mono",
        default_color: "#94A3B8",
        font_kind: ThemeFontKind::JetBrainsMono,
        is_mono: true,
    },
    ThemePreset {
        name: "Rose Gold",
        description: "Elegant pastel rose with DejaVu Serif",
        default_color: "#EC4899",
        font_kind: ThemeFontKind::DejaVuSerif,
        is_mono: false,
    },
];

pub fn get_theme_preset(name: &str) -> Option<&'static ThemePreset> {
    THEME_PRESETS
        .iter()
        .find(|p| p.name.eq_ignore_ascii_case(name))
}

pub fn get_font_pair_for_theme(theme_name: &str) -> (cosmic::iced::Font, cosmic::iced::Font) {
    if let Some(preset) = get_theme_preset(theme_name) {
        preset.font_kind.fonts()
    } else if theme_name.to_lowercase().contains("mono")
        || theme_name.to_lowercase().contains("digital")
    {
        (
            crate::clock::fonts::FONT_MONO_BOLD,
            crate::clock::fonts::FONT_MONO_REGULAR,
        )
    } else if theme_name.to_lowercase().contains("serif") {
        (
            crate::clock::fonts::FONT_SERIF_BOLD,
            crate::clock::fonts::FONT_SERIF_REGULAR,
        )
    } else {
        (
            crate::clock::fonts::FONT_ROBOTO_BOLD,
            crate::clock::fonts::FONT_ROBOTO_REGULAR,
        )
    }
}

pub struct PaletteColor {
    pub name: &'static str,
    pub hex: &'static str,
}

pub const COLOR_PALETTE_16: &[PaletteColor] = &[
    PaletteColor {
        name: "Pure White",
        hex: "#FFFFFF",
    },
    PaletteColor {
        name: "Soft Silver",
        hex: "#E2E8F0",
    },
    PaletteColor {
        name: "Cool Slate",
        hex: "#94A3B8",
    },
    PaletteColor {
        name: "Sky Blue",
        hex: "#38BDF8",
    },
    PaletteColor {
        name: "COSMIC Blue",
        hex: "#3B82F6",
    },
    PaletteColor {
        name: "Indigo",
        hex: "#6366F1",
    },
    PaletteColor {
        name: "Purple",
        hex: "#8B5CF6",
    },
    PaletteColor {
        name: "Rose Pink",
        hex: "#EC4899",
    },
    PaletteColor {
        name: "Crimson",
        hex: "#F43F5E",
    },
    PaletteColor {
        name: "Coral Red",
        hex: "#EF4444",
    },
    PaletteColor {
        name: "Orange",
        hex: "#F97316",
    },
    PaletteColor {
        name: "Amber Gold",
        hex: "#F59E0B",
    },
    PaletteColor {
        name: "Sun Yellow",
        hex: "#EAB308",
    },
    PaletteColor {
        name: "Lime",
        hex: "#84CC16",
    },
    PaletteColor {
        name: "Emerald Green",
        hex: "#10B981",
    },
    PaletteColor {
        name: "Teal Cyan",
        hex: "#06B6D4",
    },
];

pub fn parse_hex_color(hex: &str) -> Option<Color> {
    let clean = hex.trim().trim_start_matches('#');
    if clean.len() == 6 {
        let r = u8::from_str_radix(&clean[0..2], 16).ok()? as f32 / 255.0;
        let g = u8::from_str_radix(&clean[2..4], 16).ok()? as f32 / 255.0;
        let b = u8::from_str_radix(&clean[4..6], 16).ok()? as f32 / 255.0;
        Some(Color::from_rgb(r, g, b))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hex_color() {
        assert_eq!(parse_hex_color("#FFFFFF"), Some(Color::WHITE));
        assert_eq!(parse_hex_color("#000000"), Some(Color::BLACK));
        assert!(parse_hex_color("invalid").is_none());
    }

    #[test]
    fn test_presets_and_palette_counts() {
        assert_eq!(THEME_PRESETS.len(), 10);
        assert_eq!(COLOR_PALETTE_16.len(), 16);
    }
}
