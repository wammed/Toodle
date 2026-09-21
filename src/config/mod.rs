pub mod theme;

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tracing::{info, warn};

pub use theme::{COLOR_PALETTE_16, THEME_PRESETS, parse_hex_color};

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
    pub fn to_layer_anchor(
        self,
    ) -> cosmic::iced::platform_specific::shell::commands::layer_surface::Anchor {
        use cosmic::iced::platform_specific::shell::commands::layer_surface::Anchor as LayerAnchor;
        match self {
            Anchor::TopLeft => LayerAnchor::TOP | LayerAnchor::LEFT,
            Anchor::TopRight => LayerAnchor::TOP | LayerAnchor::RIGHT,
            Anchor::BottomLeft => LayerAnchor::BOTTOM | LayerAnchor::LEFT,
            Anchor::BottomRight => LayerAnchor::BOTTOM | LayerAnchor::RIGHT,
        }
    }
}

/// 9 display zones (3 columns × 3 rows: 3x3 grid)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum GridPosition {
    // Row 0: Top
    TopLeft,
    TopCenter,
    TopRight,

    // Row 1: Middle
    MiddleLeft,
    Center,
    MiddleRight,

    // Row 2: Bottom
    BottomLeft,
    BottomCenter,
    BottomRight,
}

impl Default for GridPosition {
    fn default() -> Self {
        GridPosition::TopRight
    }
}

impl GridPosition {
    pub fn label(self) -> &'static str {
        match self {
            GridPosition::TopLeft => "Top-Left",
            GridPosition::TopCenter => "Top-Center",
            GridPosition::TopRight => "Top-Right",
            GridPosition::MiddleLeft => "Mid-Left",
            GridPosition::Center => "Center",
            GridPosition::MiddleRight => "Mid-Right",
            GridPosition::BottomLeft => "Bot-Left",
            GridPosition::BottomCenter => "Bot-Center",
            GridPosition::BottomRight => "Bot-Right",
        }
    }

    pub fn to_legacy_anchor(self) -> Anchor {
        match self {
            GridPosition::TopLeft | GridPosition::TopCenter => Anchor::TopLeft,
            GridPosition::TopRight => Anchor::TopRight,
            GridPosition::MiddleLeft
            | GridPosition::Center
            | GridPosition::BottomLeft
            | GridPosition::BottomCenter => Anchor::BottomLeft,
            GridPosition::MiddleRight | GridPosition::BottomRight => Anchor::BottomRight,
        }
    }
}

/// Information for one of the 10 size stages (up to 2560x1440 WQHD)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SizeStageInfo {
    pub stage: u8,
    pub width: u32,
    pub height: u32,
    pub time_size: u16,
    pub date_size: u16,
    pub spacing: u16,
    pub padding: u16,
    pub font_scale: f32,
    pub label: &'static str,
}

pub const SIZE_STAGES: [SizeStageInfo; 10] = [
    SizeStageInfo { stage: 1,  width: 280,  height: 130, time_size: 38,  date_size: 14, spacing: 4,  padding: 8,  font_scale: 0.70, label: "Compact" },
    SizeStageInfo { stage: 2,  width: 380,  height: 175, time_size: 52,  date_size: 19, spacing: 6,  padding: 10, font_scale: 1.00, label: "Default" },
    SizeStageInfo { stage: 3,  width: 490,  height: 225, time_size: 68,  date_size: 25, spacing: 7,  padding: 13, font_scale: 1.35, label: "Medium" },
    SizeStageInfo { stage: 4,  width: 620,  height: 285, time_size: 88,  date_size: 32, spacing: 9,  padding: 16, font_scale: 1.70, label: "Standard" },
    SizeStageInfo { stage: 5,  width: 780,  height: 355, time_size: 112, date_size: 40, spacing: 11, padding: 19, font_scale: 2.20, label: "Large" },
    SizeStageInfo { stage: 6,  width: 960,  height: 440, time_size: 140, date_size: 50, spacing: 13, padding: 23, font_scale: 2.75, label: "X-Large" },
    SizeStageInfo { stage: 7,  width: 1180, height: 540, time_size: 174, date_size: 62, spacing: 16, padding: 27, font_scale: 3.35, label: "2X-Large" },
    SizeStageInfo { stage: 8,  width: 1440, height: 650, time_size: 215, date_size: 76, spacing: 19, padding: 32, font_scale: 4.15, label: "Huge" },
    SizeStageInfo { stage: 9,  width: 1740, height: 780, time_size: 260, date_size: 92, spacing: 22, padding: 38, font_scale: 4.90, label: "Giant" },
    SizeStageInfo { stage: 10, width: 2060, height: 920, time_size: 310, date_size: 110, spacing: 25, padding: 44, font_scale: 5.60, label: "Max WQHD" },
];

pub fn get_size_stage(stage: u8) -> SizeStageInfo {
    let idx = (stage.clamp(1, 10) - 1) as usize;
    SIZE_STAGES[idx]
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LayoutConfig {
    #[serde(default)]
    pub grid_position: GridPosition,
    #[serde(default = "default_size_stage")]
    pub size_stage: u8,

    // Legacy fields preserved for backward compatibility
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

fn default_size_stage() -> u8 {
    2
}

fn default_margin_x() -> i32 {
    40
}
fn default_margin_y() -> i32 {
    60
}
fn default_width() -> u32 {
    480
}
fn default_height() -> u32 {
    270
}

impl Default for LayoutConfig {
    fn default() -> Self {
        let stage_info = get_size_stage(default_size_stage());
        Self {
            grid_position: GridPosition::default(),
            size_stage: default_size_stage(),
            anchor: Anchor::default(),
            margin_x: default_margin_x(),
            margin_y: default_margin_y(),
            width: stage_info.width,
            height: stage_info.height,
        }
    }
}

impl LayoutConfig {
    /// Returns (top, right, bottom, left) margins for LayerSurface (legacy fallback)
    pub fn margins(&self) -> (i32, i32, i32, i32) {
        match self.anchor {
            Anchor::TopLeft => (self.margin_y, 0, 0, self.margin_x),
            Anchor::TopRight => (self.margin_y, self.margin_x, 0, 0),
            Anchor::BottomLeft => (0, 0, self.margin_y, self.margin_x),
            Anchor::BottomRight => (0, self.margin_x, self.margin_y, 0),
        }
    }

    /// Calculate LayerSurface geometry for current grid_position and size_stage
    pub fn calculate_geometry(
        &self,
        screen_w: u32,
        screen_h: u32,
    ) -> (
        cosmic::iced::platform_specific::shell::commands::layer_surface::Anchor,
        (i32, i32, i32, i32),
        (u32, u32),
        f32,
    ) {
        Self::geometry_for(self.grid_position, self.size_stage, screen_w, screen_h)
    }

    /// Calculate LayerSurface geometry for given position and size_stage
    pub fn geometry_for(
        position: GridPosition,
        stage: u8,
        screen_w: u32,
        screen_h: u32,
    ) -> (
        cosmic::iced::platform_specific::shell::commands::layer_surface::Anchor,
        (i32, i32, i32, i32),
        (u32, u32),
        f32,
    ) {
        use cosmic::iced::platform_specific::shell::commands::layer_surface::Anchor as LayerAnchor;
        let info = get_size_stage(stage);
        let screen_w = screen_w.max(640);
        let screen_h = screen_h.max(360);
        let w = info.width.min(screen_w);
        let h = info.height.min(screen_h);
        let font_scale = info.font_scale;

        let gutter = 32i32;

        // Horizontal center (Column 1)
        let mid_left = ((screen_w as i32 - w as i32) / 2).max(gutter);

        // Vertical center (Row 1)
        let mid_top = ((screen_h as i32 - h as i32) / 2).max(gutter);

        match position {
            GridPosition::TopLeft => (
                LayerAnchor::TOP | LayerAnchor::LEFT,
                (gutter, 0, 0, gutter),
                (w, h),
                font_scale,
            ),
            GridPosition::TopCenter => (
                LayerAnchor::TOP | LayerAnchor::LEFT,
                (gutter, 0, 0, mid_left),
                (w, h),
                font_scale,
            ),
            GridPosition::TopRight => (
                LayerAnchor::TOP | LayerAnchor::RIGHT,
                (gutter, gutter, 0, 0),
                (w, h),
                font_scale,
            ),

            GridPosition::MiddleLeft => (
                LayerAnchor::TOP | LayerAnchor::LEFT,
                (mid_top, 0, 0, gutter),
                (w, h),
                font_scale,
            ),
            GridPosition::Center => (
                LayerAnchor::TOP | LayerAnchor::LEFT,
                (mid_top, 0, 0, mid_left),
                (w, h),
                font_scale,
            ),
            GridPosition::MiddleRight => (
                LayerAnchor::TOP | LayerAnchor::RIGHT,
                (mid_top, gutter, 0, 0),
                (w, h),
                font_scale,
            ),

            GridPosition::BottomLeft => (
                LayerAnchor::BOTTOM | LayerAnchor::LEFT,
                (0, 0, gutter, gutter),
                (w, h),
                font_scale,
            ),
            GridPosition::BottomCenter => (
                LayerAnchor::BOTTOM | LayerAnchor::LEFT,
                (0, 0, gutter, mid_left),
                (w, h),
                font_scale,
            ),
            GridPosition::BottomRight => (
                LayerAnchor::BOTTOM | LayerAnchor::RIGHT,
                (0, gutter, gutter, 0),
                (w, h),
                font_scale,
            ),
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
                        warn!(
                            "Failed to parse config at {:?}: {}. Using default.",
                            path, e
                        );
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
                        if event
                            .paths
                            .iter()
                            .any(|p| p.file_name() == path.file_name())
                        {
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

    #[test]
    fn test_size_stages() {
        assert_eq!(SIZE_STAGES.len(), 10);
        let s1 = get_size_stage(1);
        assert_eq!(s1.width, 280);
        assert_eq!(s1.height, 130);
        assert_eq!(s1.time_size, 38);
        assert_eq!(s1.stage, 1);

        let s10 = get_size_stage(10);
        assert_eq!(s10.width, 2060);
        assert_eq!(s10.height, 920);
        assert_eq!(s10.time_size, 310);
        assert_eq!(s10.stage, 10);
        assert_eq!(s10.label, "Max WQHD");

        // Clamp tests
        assert_eq!(get_size_stage(0).stage, 1);
        assert_eq!(get_size_stage(11).stage, 10);
    }

    #[test]
    fn test_grid_position_geometry() {
        let (screen_w, screen_h) = (2560, 1440);
        let cfg = LayoutConfig {
            grid_position: GridPosition::TopRight,
            size_stage: 2,
            ..Default::default()
        };
        let (_anchor, (top, right, _bottom, _left), (w, h), scale) =
            cfg.calculate_geometry(screen_w, screen_h);
        assert_eq!(w, 380);
        assert_eq!(h, 175);
        assert_eq!(scale, 1.0);
        assert_eq!(top, 32);
        assert_eq!(right, 32);

        // Center
        let cfg_c = LayoutConfig {
            grid_position: GridPosition::Center,
            size_stage: 5,
            ..Default::default()
        };
        let (_anchor_c, (t, _r, _b, l), (w_c, h_c), _s) =
            cfg_c.calculate_geometry(screen_w, screen_h);
        assert_eq!(w_c, 780);
        assert_eq!(h_c, 355);
        assert_eq!(l, (2560 - 780) / 2);
        assert_eq!(t, (1440 - 355) / 2);
    }
}

