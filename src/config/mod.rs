pub mod theme;

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tracing::{info, warn};

pub use theme::{parse_hex_color, COLOR_PALETTE_16, THEME_PRESETS};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Config {
    #[serde(default)]
    pub display: DisplayConfig,
    #[serde(default)]
    pub layout: LayoutConfig,
    #[serde(default)]
    pub appearance: AppearanceConfig,
    #[serde(default)]
    pub weather: WeatherConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            display: DisplayConfig::default(),
            layout: LayoutConfig::default(),
            appearance: AppearanceConfig::default(),
            weather: WeatherConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DisplayConfig {
    /// Target output display name (e.g., "DP-1", "HDMI-A-1", or empty for primary/default)
    #[serde(default)]
    pub output: Option<String>,
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self { output: None }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Anchor {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

impl Default for Anchor {
    fn default() -> Self {
        Anchor::TopRight
    }
}

impl Anchor {
    pub fn to_layer_anchor(self) -> cosmic::iced::platform_specific::shell::commands::layer_surface::Anchor {
        use cosmic::iced::platform_specific::shell::commands::layer_surface::Anchor as LayerAnchor;
        match self {
            Anchor::TopLeft => LayerAnchor::TOP | LayerAnchor::LEFT,
            Anchor::TopRight => LayerAnchor::TOP | LayerAnchor::RIGHT,
            Anchor::BottomLeft => LayerAnchor::BOTTOM | LayerAnchor::LEFT,
            Anchor::BottomRight => LayerAnchor::BOTTOM | LayerAnchor::RIGHT,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LayoutConfig {
    #[serde(default)]
    pub anchor: Anchor,
    #[serde(default = "default_margin_x")]
    pub margin_x: i32,
    #[serde(default = "default_margin_y")]
    pub margin_y: i32,
    #[serde(default = "default_width")]
    pub width: u32,
    #[serde(default = "default_height")]
    pub height: u32,
}

fn default_margin_x() -> i32 {
    40
}
fn default_margin_y() -> i32 {
    60
}
fn default_width() -> u32 {
    320
}
fn default_height() -> u32 {
    160
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self {
            anchor: Anchor::default(),
            margin_x: default_margin_x(),
            margin_y: default_margin_y(),
            width: default_width(),
            height: default_height(),
        }
    }
}

impl LayoutConfig {
    /// Returns (top, right, bottom, left) margins for LayerSurface
    pub fn margins(&self) -> (i32, i32, i32, i32) {
        match self.anchor {
            Anchor::TopLeft => (self.margin_y, 0, 0, self.margin_x),
            Anchor::TopRight => (self.margin_y, self.margin_x, 0, 0),
            Anchor::BottomLeft => (0, 0, self.margin_y, self.margin_x),
            Anchor::BottomRight => (0, self.margin_x, self.margin_y, 0),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppearanceConfig {
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default = "default_color")]
    pub color: String,
    #[serde(default = "default_font_scale")]
    pub font_scale: f32,
    #[serde(default = "default_text_shadow")]
    pub text_shadow: bool,
}

fn default_theme() -> String {
    "Modern".to_string()
}
fn default_color() -> String {
    "#FFFFFF".to_string()
}
fn default_font_scale() -> f32 {
    1.0
}
fn default_text_shadow() -> bool {
    true
}

impl Default for AppearanceConfig {
    fn default() -> Self {
        Self {
            theme: default_theme(),
            color: default_color(),
            font_scale: default_font_scale(),
            text_shadow: default_text_shadow(),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TemperatureUnit {
    Celsius,
    Fahrenheit,
}

impl Default for TemperatureUnit {
    fn default() -> Self {
        TemperatureUnit::Celsius
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WeatherConfig {
    #[serde(default = "default_location_name")]
    pub location_name: String,
    #[serde(default = "default_latitude")]
    pub latitude: f64,
    #[serde(default = "default_longitude")]
    pub longitude: f64,
    #[serde(default)]
    pub temperature_unit: TemperatureUnit,
}

fn default_location_name() -> String {
    "Tokyo, Japan".to_string()
}
fn default_latitude() -> f64 {
    35.6895
}
fn default_longitude() -> f64 {
    139.6917
}

impl Default for WeatherConfig {
    fn default() -> Self {
        Self {
            location_name: default_location_name(),
            latitude: default_latitude(),
            longitude: default_longitude(),
            temperature_unit: TemperatureUnit::default(),
        }
    }
}

impl Config {
    pub fn config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from(".config"))
            .join("toodle")
            .join("config.toml")
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            match fs::read_to_string(&path) {
                Ok(content) => match toml::from_str::<Config>(&content) {
                    Ok(cfg) => {
                        info!("Loaded configuration from {:?}", path);
                        return cfg;
                    }
                    Err(e) => {
                        warn!("Failed to parse config at {:?}: {}. Using default.", path, e);
                    }
                },
                Err(e) => {
                    warn!("Failed to read config at {:?}: {}. Using default.", path, e);
                }
            }
        } else {
            let default_cfg = Config::default();
            let _ = default_cfg.save();
            return default_cfg;
        }
        Config::default()
    }

    pub fn save(&self) -> Result<(), std::io::Error> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let serialized = toml::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        // Atomic write via temporary file
        let tmp_path = path.with_extension("tmp");
        fs::write(&tmp_path, serialized)?;
        fs::rename(&tmp_path, &path)?;
        info!("Saved configuration to {:?}", path);
        Ok(())
    }

    /// Creates an asynchronous stream watching the config file and emitting updated Config instances.
    pub fn watch() -> impl futures::Stream<Item = Config> {
        let (tx, rx) = futures::channel::mpsc::unbounded();
        let path = Config::config_path();
        let parent = path.parent().unwrap_or(&path).to_path_buf();

        std::thread::spawn(move || {
            use notify::{RecursiveMode, Watcher};
            let (std_tx, std_rx) = std::sync::mpsc::channel();
            if let Ok(mut watcher) = notify::recommended_watcher(std_tx) {
                let _ = watcher.watch(&parent, RecursiveMode::NonRecursive);
                while let Ok(event) = std_rx.recv() {
                    if let Ok(event) = event {
                        if event.paths.iter().any(|p| p.file_name() == path.file_name()) {
                            // Small delay to ensure atomic rename write is flushed
                            std::thread::sleep(std::time::Duration::from_millis(25));
                            let cfg = Config::load();
                            if tx.unbounded_send(cfg).is_err() {
                                break;
                            }
                        }
                    }
                }
            }
        });

        rx
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_serialization() {
        let cfg = Config::default();
        let s = toml::to_string(&cfg).expect("serialize");
        let des: Config = toml::from_str(&s).expect("deserialize");
        assert_eq!(cfg, des);
    }
}
