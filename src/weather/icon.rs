/// Semantic weather icon categories abstracting weather conditions from specific fonts/glyphs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WeatherIcon {
    Clear,
    PartlyCloudy,
    Cloudy,
    Fog,
    Drizzle,
    FreezingDrizzle,
    Rain,
    FreezingRain,
    Snow,
    SnowShower,
    RainShower,
    Thunderstorm,
}

impl WeatherIcon {
    /// Maps a WMO weather interpretation code to a semantic `WeatherIcon`.
    pub fn from_wmo_code(code: u8) -> Self {
        match code {
            0 => Self::Clear,
            1 | 2 => Self::PartlyCloudy,
            3 => Self::Cloudy,
            45 | 48 => Self::Fog,
            51 | 53 | 55 => Self::Drizzle,
            56 | 57 => Self::FreezingDrizzle,
            61 | 63 | 65 => Self::Rain,
            66 | 67 => Self::FreezingRain,
            71 | 73 | 75 | 77 => Self::Snow,
            80 | 81 | 82 => Self::RainShower,
            85 | 86 => Self::SnowShower,
            95 | 96 | 99 => Self::Thunderstorm,
            _ => Self::Cloudy,
        }
    }

    /// Converts the semantic weather icon into a Weather Icons font glyph character.
    pub fn glyph(self) -> char {
        weather_icon_glyph(self)
    }
}

/// Maps a semantic `WeatherIcon` to its corresponding Weather Icons font glyph codepoint.
pub fn weather_icon_glyph(icon: WeatherIcon) -> char {
    match icon {
        WeatherIcon::Clear => '\u{f00d}',           // wi-day-sunny
        WeatherIcon::PartlyCloudy => '\u{f002}',    // wi-day-cloudy
        WeatherIcon::Cloudy => '\u{f013}',          // wi-cloudy
        WeatherIcon::Fog => '\u{f014}',             // wi-fog
        WeatherIcon::Drizzle => '\u{f01c}',         // wi-sprinkle
        WeatherIcon::FreezingDrizzle => '\u{f0b5}', // wi-sleet
        WeatherIcon::Rain => '\u{f019}',            // wi-rain
        WeatherIcon::FreezingRain => '\u{f017}',    // wi-rain-mix
        WeatherIcon::Snow => '\u{f01b}',            // wi-snow
        WeatherIcon::SnowShower => '\u{f064}',      // wi-snow-wind
        WeatherIcon::RainShower => '\u{f01a}',      // wi-showers
        WeatherIcon::Thunderstorm => '\u{f01e}',    // wi-thunderstorm
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wmo_code_to_weather_icon_classification() {
        assert_eq!(WeatherIcon::from_wmo_code(0), WeatherIcon::Clear);

        assert_eq!(WeatherIcon::from_wmo_code(1), WeatherIcon::PartlyCloudy);
        assert_eq!(WeatherIcon::from_wmo_code(2), WeatherIcon::PartlyCloudy);

        assert_eq!(WeatherIcon::from_wmo_code(3), WeatherIcon::Cloudy);

        assert_eq!(WeatherIcon::from_wmo_code(45), WeatherIcon::Fog);
        assert_eq!(WeatherIcon::from_wmo_code(48), WeatherIcon::Fog);

        assert_eq!(WeatherIcon::from_wmo_code(51), WeatherIcon::Drizzle);
        assert_eq!(WeatherIcon::from_wmo_code(53), WeatherIcon::Drizzle);
        assert_eq!(WeatherIcon::from_wmo_code(55), WeatherIcon::Drizzle);

        // Explicitly verify WMO 56 & 57 are mapped to FreezingDrizzle
        assert_eq!(WeatherIcon::from_wmo_code(56), WeatherIcon::FreezingDrizzle);
        assert_eq!(WeatherIcon::from_wmo_code(57), WeatherIcon::FreezingDrizzle);

        assert_eq!(WeatherIcon::from_wmo_code(61), WeatherIcon::Rain);
        assert_eq!(WeatherIcon::from_wmo_code(63), WeatherIcon::Rain);
        assert_eq!(WeatherIcon::from_wmo_code(65), WeatherIcon::Rain);

        assert_eq!(WeatherIcon::from_wmo_code(66), WeatherIcon::FreezingRain);
        assert_eq!(WeatherIcon::from_wmo_code(67), WeatherIcon::FreezingRain);

        assert_eq!(WeatherIcon::from_wmo_code(71), WeatherIcon::Snow);
        assert_eq!(WeatherIcon::from_wmo_code(73), WeatherIcon::Snow);
        assert_eq!(WeatherIcon::from_wmo_code(75), WeatherIcon::Snow);
        assert_eq!(WeatherIcon::from_wmo_code(77), WeatherIcon::Snow);

        assert_eq!(WeatherIcon::from_wmo_code(80), WeatherIcon::RainShower);
        assert_eq!(WeatherIcon::from_wmo_code(81), WeatherIcon::RainShower);
        assert_eq!(WeatherIcon::from_wmo_code(82), WeatherIcon::RainShower);

        assert_eq!(WeatherIcon::from_wmo_code(85), WeatherIcon::SnowShower);
        assert_eq!(WeatherIcon::from_wmo_code(86), WeatherIcon::SnowShower);

        assert_eq!(WeatherIcon::from_wmo_code(95), WeatherIcon::Thunderstorm);
        assert_eq!(WeatherIcon::from_wmo_code(96), WeatherIcon::Thunderstorm);
        assert_eq!(WeatherIcon::from_wmo_code(99), WeatherIcon::Thunderstorm);

        // Fallback for unknown codes
        assert_eq!(WeatherIcon::from_wmo_code(255), WeatherIcon::Cloudy);
    }

    #[test]
    fn test_weather_icon_to_glyph_mapping() {
        let all_icons = [
            (WeatherIcon::Clear, '\u{f00d}'),
            (WeatherIcon::PartlyCloudy, '\u{f002}'),
            (WeatherIcon::Cloudy, '\u{f013}'),
            (WeatherIcon::Fog, '\u{f014}'),
            (WeatherIcon::Drizzle, '\u{f01c}'),
            (WeatherIcon::FreezingDrizzle, '\u{f0b5}'),
            (WeatherIcon::Rain, '\u{f019}'),
            (WeatherIcon::FreezingRain, '\u{f017}'),
            (WeatherIcon::Snow, '\u{f01b}'),
            (WeatherIcon::SnowShower, '\u{f064}'),
            (WeatherIcon::RainShower, '\u{f01a}'),
            (WeatherIcon::Thunderstorm, '\u{f01e}'),
        ];

        for (icon, expected_glyph) in all_icons {
            assert_eq!(weather_icon_glyph(icon), expected_glyph);
            assert_eq!(icon.glyph(), expected_glyph);
            // Verify glyph is in Weather Icons Private Use Area
            let cp = expected_glyph as u32;
            assert!(
                (0xf000..=0xf2ff).contains(&cp),
                "Glyph {expected_glyph:?} (0x{cp:x}) is outside expected PUA range"
            );
        }
    }
}
