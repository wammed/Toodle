pub mod theme;

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tracing::{info, warn};

pub use theme::{COLOR_PALETTE_16, THEME_PRESETS, parse_hex_color};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct DisplayConfig {
    /// Target output display name (e.g., "DP-1", "HDMI-A-1", or empty for primary/default)
    #[serde(default)]
    pub output: Option<String>,
}

/// Legacy 4-corner anchor (retained for legacy config migration compatibility only)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum Anchor {
    TopLeft,
    #[default]
    TopRight,
    BottomLeft,
    BottomRight,
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
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum GridPosition {
    // Row 0: Top
    TopLeft,
    TopCenter,
    #[default]
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
    SizeStageInfo {
        stage: 1,
        width: 280,
        height: 130,
        time_size: 38,
        date_size: 14,
        spacing: 4,
        padding: 8,
        font_scale: 0.70,
        label: "Compact",
    },
    SizeStageInfo {
        stage: 2,
        width: 380,
        height: 175,
        time_size: 52,
        date_size: 19,
        spacing: 6,
        padding: 10,
        font_scale: 1.00,
        label: "Default",
    },
    SizeStageInfo {
        stage: 3,
        width: 490,
        height: 225,
        time_size: 68,
        date_size: 25,
        spacing: 7,
        padding: 13,
        font_scale: 1.35,
        label: "Medium",
    },
    SizeStageInfo {
        stage: 4,
        width: 620,
        height: 285,
        time_size: 88,
        date_size: 32,
        spacing: 9,
        padding: 16,
        font_scale: 1.70,
        label: "Standard",
    },
    SizeStageInfo {
        stage: 5,
        width: 780,
        height: 355,
        time_size: 112,
        date_size: 40,
        spacing: 11,
        padding: 19,
        font_scale: 2.20,
        label: "Large",
    },
    SizeStageInfo {
        stage: 6,
        width: 960,
        height: 440,
        time_size: 140,
        date_size: 50,
        spacing: 13,
        padding: 23,
        font_scale: 2.75,
        label: "X-Large",
    },
    SizeStageInfo {
        stage: 7,
        width: 1180,
        height: 540,
        time_size: 174,
        date_size: 62,
        spacing: 16,
        padding: 27,
        font_scale: 3.35,
        label: "2X-Large",
    },
    SizeStageInfo {
        stage: 8,
        width: 1440,
        height: 650,
        time_size: 215,
        date_size: 76,
        spacing: 19,
        padding: 32,
        font_scale: 4.15,
        label: "Huge",
    },
    SizeStageInfo {
        stage: 9,
        width: 1740,
        height: 780,
        time_size: 260,
        date_size: 92,
        spacing: 22,
        padding: 38,
        font_scale: 4.90,
        label: "Giant",
    },
    SizeStageInfo {
        stage: 10,
        width: 2060,
        height: 920,
        time_size: 310,
        date_size: 110,
        spacing: 25,
        padding: 44,
        font_scale: 5.60,
        label: "Max WQHD",
    },
];

pub fn get_size_stage(stage: u8) -> SizeStageInfo {
    let idx = (stage.clamp(1, 10) - 1) as usize;
    SIZE_STAGES[idx]
}

/// Official finite-state layout configuration model.
/// Layout is fully defined by 9 screen zones and 10 discrete size stages.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct LayoutConfig {
    pub grid_position: GridPosition,
    pub size_stage: u8,
}

pub fn default_size_stage() -> u8 {
    2
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self {
            grid_position: GridPosition::default(),
            size_stage: default_size_stage(),
        }
    }
}

/// Raw deserialization representation to support explicit, deterministic migration
/// from legacy layout fields (anchor, margin_x, margin_y, width, height).
#[derive(Deserialize)]
struct RawLayoutConfig {
    grid_position: Option<GridPosition>,
    size_stage: Option<u8>,

    // Legacy fields for backward compatibility and migration
    anchor: Option<Anchor>,
    margin_x: Option<i32>,
    margin_y: Option<i32>,
    width: Option<u32>,
    height: Option<u32>,
}

impl<'de> serde::Deserialize<'de> for LayoutConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw = RawLayoutConfig::deserialize(deserializer)?;
        Ok(raw.into_layout_config())
    }
}

impl RawLayoutConfig {
    pub(crate) fn is_legacy(&self) -> bool {
        self.grid_position.is_none()
            && self.size_stage.is_none()
            && (self.anchor.is_some()
                || self.margin_x.is_some()
                || self.margin_y.is_some()
                || self.width.is_some()
                || self.height.is_some())
    }

    fn into_layout_config(self) -> LayoutConfig {
        // 1. Both modern fields specified: use directly
        if let (Some(grid_position), Some(size_stage)) = (self.grid_position, self.size_stage) {
            return LayoutConfig {
                grid_position,
                size_stage: size_stage.clamp(1, 10),
            };
        }

        // 2. At least one modern field specified: fill missing modern field with default
        if self.grid_position.is_some() || self.size_stage.is_some() {
            return LayoutConfig {
                grid_position: self.grid_position.unwrap_or_default(),
                size_stage: self
                    .size_stage
                    .unwrap_or_else(default_size_stage)
                    .clamp(1, 10),
            };
        }

        // 3. Legacy fields present: perform explicit, deterministic migration
        if self.anchor.is_some()
            || self.margin_x.is_some()
            || self.margin_y.is_some()
            || self.width.is_some()
            || self.height.is_some()
        {
            return migrate_legacy_layout(
                self.anchor,
                self.margin_x,
                self.margin_y,
                self.width,
                self.height,
            );
        }

        // 4. Complete fallback to default
        LayoutConfig::default()
    }
}

/// Deterministic migration from legacy layout fields (anchor, margin_x, margin_y, width, height)
/// to the finite-state layout model (GridPosition + SizeStage).
pub fn migrate_legacy_layout(
    anchor: Option<Anchor>,
    margin_x: Option<i32>,
    margin_y: Option<i32>,
    width: Option<u32>,
    height: Option<u32>,
) -> LayoutConfig {
    let legacy_anchor = anchor.unwrap_or(Anchor::TopRight);
    let mx = margin_x.unwrap_or(40);
    let my = margin_y.unwrap_or(60);
    let w = width.unwrap_or(480);
    let h = height.unwrap_or(270);

    let grid_position = migrate_position(legacy_anchor, mx, my, w, h);
    let size_stage = migrate_size_stage(w, h);

    info!(
        "Migrated legacy layout config (anchor={:?}, margin_x={}, margin_y={}, width={}, height={}) -> (grid_position={:?}, size_stage={})",
        legacy_anchor, mx, my, w, h, grid_position, size_stage
    );

    LayoutConfig {
        grid_position,
        size_stage,
    }
}

/// Deterministically map legacy anchor + margins to the closest 3x3 GridPosition.
/// Analyzes margin offsets relative to the anchor edge:
/// - Low horizontal margin (<350px): near the anchored edge (Left or Right).
/// - Medium horizontal margin (350..=1100px): centered horizontally (TopCenter/Center/BottomCenter).
/// - High horizontal margin (>1100px): shifted to the opposite horizontal edge.
/// - Low vertical margin (<250px): near the anchored edge (Top or Bottom).
/// - Medium vertical margin (250..=700px): centered vertically (MiddleLeft/Center/MiddleRight).
/// - High vertical margin (>700px): shifted to the opposite vertical edge.
pub fn migrate_position(
    anchor: Anchor,
    margin_x: i32,
    margin_y: i32,
    _width: u32,
    _height: u32,
) -> GridPosition {
    let col = match anchor {
        Anchor::TopLeft | Anchor::BottomLeft => {
            if margin_x < 350 {
                0 // Left
            } else if margin_x <= 1100 {
                1 // Center
            } else {
                2 // Right
            }
        }
        Anchor::TopRight | Anchor::BottomRight => {
            if margin_x < 350 {
                2 // Right
            } else if margin_x <= 1100 {
                1 // Center
            } else {
                0 // Left
            }
        }
    };

    let row = match anchor {
        Anchor::TopLeft | Anchor::TopRight => {
            if margin_y < 250 {
                0 // Top
            } else if margin_y <= 700 {
                1 // Middle
            } else {
                2 // Bottom
            }
        }
        Anchor::BottomLeft | Anchor::BottomRight => {
            if margin_y < 250 {
                2 // Bottom
            } else if margin_y <= 700 {
                1 // Middle
            } else {
                0 // Top
            }
        }
    };

    match (row, col) {
        (0, 0) => GridPosition::TopLeft,
        (0, 1) => GridPosition::TopCenter,
        (0, 2) => GridPosition::TopRight,
        (1, 0) => GridPosition::MiddleLeft,
        (1, 1) => GridPosition::Center,
        (1, 2) => GridPosition::MiddleRight,
        (2, 0) => GridPosition::BottomLeft,
        (2, 1) => GridPosition::BottomCenter,
        (2, 2) => GridPosition::BottomRight,
        _ => GridPosition::TopRight,
    }
}

/// Find the closest SizeStage in SIZE_STAGES based on Euclidean distance to (width, height).
pub fn migrate_size_stage(width: u32, height: u32) -> u8 {
    let mut best_stage = 2;
    let mut min_dist_sq = i64::MAX;

    for info in SIZE_STAGES.iter() {
        let dw = width as i64 - info.width as i64;
        let dh = height as i64 - info.height as i64;
        let dist_sq = dw * dw + dh * dh;
        if dist_sq < min_dist_sq {
            min_dist_sq = dist_sq;
            best_stage = info.stage;
        }
    }

    best_stage
}

pub type LayerGeometry = (
    cosmic::iced::platform_specific::shell::commands::layer_surface::Anchor,
    (i32, i32, i32, i32),
    (u32, u32),
    f32,
);

impl LayoutConfig {
    /// Calculate LayerSurface geometry for current grid_position and size_stage
    pub fn calculate_geometry(&self, screen_w: u32, screen_h: u32) -> LayerGeometry {
        Self::geometry_for(self.grid_position, self.size_stage, screen_w, screen_h)
    }

    /// Calculate LayerSurface geometry for given position and size_stage
    pub fn geometry_for(
        position: GridPosition,
        stage: u8,
        screen_w: u32,
        screen_h: u32,
    ) -> LayerGeometry {
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
    #[serde(default = "default_text_shadow")]
    pub text_shadow: bool,
}

fn default_theme() -> String {
    "Modern".to_string()
}
fn default_color() -> String {
    "#FFFFFF".to_string()
}
fn default_text_shadow() -> bool {
    true
}

impl Default for AppearanceConfig {
    fn default() -> Self {
        Self {
            theme: default_theme(),
            color: default_color(),
            text_shadow: default_text_shadow(),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum TemperatureUnit {
    #[default]
    Celsius,
    Fahrenheit,
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

#[derive(Deserialize)]
struct RawConfig {
    #[serde(default)]
    display: DisplayConfig,
    #[serde(default)]
    layout: Option<RawLayoutConfig>,
    #[serde(default)]
    appearance: AppearanceConfig,
    #[serde(default)]
    weather: WeatherConfig,
}

impl Config {
    pub fn config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from(".config"))
            .join("toodle")
            .join("config.toml")
    }

    /// Parse configuration from a TOML string, detecting if legacy migration occurred.
    pub fn load_from_str(content: &str) -> Result<(Config, bool), toml::de::Error> {
        let raw: RawConfig = toml::from_str(content)?;
        let mut migrated = false;
        let layout = if let Some(raw_layout) = raw.layout {
            if raw_layout.is_legacy() {
                migrated = true;
            }
            raw_layout.into_layout_config()
        } else {
            LayoutConfig::default()
        };

        Ok((
            Config {
                display: raw.display,
                layout,
                appearance: raw.appearance,
                weather: raw.weather,
            },
            migrated,
        ))
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            match fs::read_to_string(&path) {
                Ok(content) => match Self::load_from_str(&content) {
                    Ok((cfg, migrated)) => {
                        info!("Loaded configuration from {:?}", path);
                        if migrated {
                            info!(
                                "Legacy configuration detected at {:?}. Rewriting with modern format.",
                                path
                            );
                            if let Err(e) = cfg.save() {
                                warn!("Failed to save migrated configuration to {:?}: {}", path, e);
                            }
                        }
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
            if let Err(e) = default_cfg.save() {
                warn!("Failed to save default config to {:?}: {}", path, e);
            }
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
            .map_err(std::io::Error::other)?;

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
                while let Ok(Ok(event)) = std_rx.recv() {
                    if event
                        .paths
                        .iter()
                        .any(|p| p.file_name() == path.file_name())
                    {
                        // Debounce / coalesce rapid sequential filesystem events
                        std::thread::sleep(std::time::Duration::from_millis(50));
                        while let Ok(coalesced) = std_rx.try_recv() {
                            let _ = coalesced;
                        }
                        let cfg = Config::load();
                        if tx.unbounded_send(cfg).is_err() {
                            break;
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
    fn test_size_stage_monotonicity_and_integrity() {
        for i in 0..SIZE_STAGES.len() {
            let cur = &SIZE_STAGES[i];
            assert_eq!(cur.stage, (i + 1) as u8);
            assert!(cur.width > 0);
            assert!(cur.height > 0);
            assert!(cur.time_size > 0);
            assert!(cur.date_size > 0);
            assert!(cur.padding > 0);
            assert!(cur.font_scale > 0.0);

            if i + 1 < SIZE_STAGES.len() {
                let next = &SIZE_STAGES[i + 1];
                assert!(
                    next.width > cur.width,
                    "Stage {} width ({}) not greater than Stage {} width ({})",
                    next.stage,
                    next.width,
                    cur.stage,
                    cur.width
                );
                assert!(
                    next.height > cur.height,
                    "Stage {} height ({}) not greater than Stage {} height ({})",
                    next.stage,
                    next.height,
                    cur.stage,
                    cur.height
                );
                assert!(
                    next.time_size > cur.time_size,
                    "Stage {} time_size not monotonic",
                    next.stage
                );
                assert!(
                    next.date_size > cur.date_size,
                    "Stage {} date_size not monotonic",
                    next.stage
                );
                assert!(
                    next.font_scale > cur.font_scale,
                    "Stage {} font_scale not monotonic",
                    next.stage
                );
                assert!(
                    next.spacing >= cur.spacing,
                    "Stage {} spacing decreased",
                    next.stage
                );
                assert!(
                    next.padding >= cur.padding,
                    "Stage {} padding decreased",
                    next.stage
                );
            }
        }
    }

    #[test]
    fn test_legacy_config_migration() {
        // 1. Default legacy configuration: TopRight with default margins and dimensions
        let legacy_toml = r##"
[display]
output = ""

[layout]
anchor = "TopRight"
margin_x = 40
margin_y = 60
width = 480
height = 270

[appearance]
theme = "Modern"
color = "#FFFFFF"
font_scale = 1.0
text_shadow = true
"##;
        let cfg: Config = toml::from_str(legacy_toml).expect("parse legacy config");
        assert_eq!(cfg.layout.grid_position, GridPosition::TopRight);
        assert_eq!(cfg.layout.size_stage, 3); // 480x270 is closest to Stage 3 (490x225)

        // 2. Legacy TopLeft with small margins
        let toml_top_left = r#"
anchor = "TopLeft"
margin_x = 32
margin_y = 32
width = 280
height = 130
"#;
        let cfg_tl: LayoutConfig = toml::from_str(toml_top_left).expect("parse top-left");
        assert_eq!(cfg_tl.grid_position, GridPosition::TopLeft);
        assert_eq!(cfg_tl.size_stage, 1);

        // 3. Legacy TopLeft pushed horizontally to center
        let toml_top_center = r#"
anchor = "TopLeft"
margin_x = 720
margin_y = 32
width = 380
height = 175
"#;
        let cfg_tc: LayoutConfig = toml::from_str(toml_top_center).expect("parse top-center");
        assert_eq!(cfg_tc.grid_position, GridPosition::TopCenter);
        assert_eq!(cfg_tc.size_stage, 2);

        // 4. Legacy pushed both horizontally and vertically to center
        let toml_center = r#"
anchor = "TopLeft"
margin_x = 720
margin_y = 400
width = 780
height = 355
"#;
        let cfg_c: LayoutConfig = toml::from_str(toml_center).expect("parse center");
        assert_eq!(cfg_c.grid_position, GridPosition::Center);
        assert_eq!(cfg_c.size_stage, 5);

        // 5. Legacy BottomLeft
        let toml_bl = r#"
anchor = "BottomLeft"
margin_x = 32
margin_y = 32
width = 280
height = 130
"#;
        let cfg_bl: LayoutConfig = toml::from_str(toml_bl).expect("parse bottom-left");
        assert_eq!(cfg_bl.grid_position, GridPosition::BottomLeft);
        assert_eq!(cfg_bl.size_stage, 1);

        // 6. Legacy BottomRight
        let toml_br = r#"
anchor = "BottomRight"
margin_x = 40
margin_y = 40
width = 2060
height = 920
"#;
        let cfg_br: LayoutConfig = toml::from_str(toml_br).expect("parse bottom-right");
        assert_eq!(cfg_br.grid_position, GridPosition::BottomRight);
        assert_eq!(cfg_br.size_stage, 10);

        // 7. Legacy MiddleLeft
        let toml_ml = r#"
anchor = "TopLeft"
margin_x = 32
margin_y = 400
width = 380
height = 175
"#;
        let cfg_ml: LayoutConfig = toml::from_str(toml_ml).expect("parse mid-left");
        assert_eq!(cfg_ml.grid_position, GridPosition::MiddleLeft);

        // 8. Legacy MiddleRight
        let toml_mr = r#"
anchor = "TopRight"
margin_x = 32
margin_y = 400
width = 380
height = 175
"#;
        let cfg_mr: LayoutConfig = toml::from_str(toml_mr).expect("parse mid-right");
        assert_eq!(cfg_mr.grid_position, GridPosition::MiddleRight);

        // 9. Legacy BottomCenter
        let toml_bc = r#"
anchor = "BottomLeft"
margin_x = 720
margin_y = 32
width = 380
height = 175
"#;
        let cfg_bc: LayoutConfig = toml::from_str(toml_bc).expect("parse bot-center");
        assert_eq!(cfg_bc.grid_position, GridPosition::BottomCenter);

        // 10. Reserialization writes strictly the new format without legacy fields
        let serialized = toml::to_string_pretty(&cfg).expect("serialize migrated");
        assert!(!serialized.contains("anchor"));
        assert!(!serialized.contains("margin_x"));
        assert!(!serialized.contains("margin_y"));
        assert!(!serialized.contains("width"));
        assert!(!serialized.contains("height"));
        assert!(!serialized.contains("font_scale"));
        assert!(serialized.contains("grid_position"));
        assert!(serialized.contains("size_stage"));
    }

    #[test]
    fn test_geometry_all_9_grid_positions_multiple_resolutions() {
        use cosmic::iced::platform_specific::shell::commands::layer_surface::Anchor as LayerAnchor;

        let all_positions = [
            (GridPosition::TopLeft, LayerAnchor::TOP | LayerAnchor::LEFT),
            (
                GridPosition::TopCenter,
                LayerAnchor::TOP | LayerAnchor::LEFT,
            ),
            (
                GridPosition::TopRight,
                LayerAnchor::TOP | LayerAnchor::RIGHT,
            ),
            (
                GridPosition::MiddleLeft,
                LayerAnchor::TOP | LayerAnchor::LEFT,
            ),
            (GridPosition::Center, LayerAnchor::TOP | LayerAnchor::LEFT),
            (
                GridPosition::MiddleRight,
                LayerAnchor::TOP | LayerAnchor::RIGHT,
            ),
            (
                GridPosition::BottomLeft,
                LayerAnchor::BOTTOM | LayerAnchor::LEFT,
            ),
            (
                GridPosition::BottomCenter,
                LayerAnchor::BOTTOM | LayerAnchor::LEFT,
            ),
            (
                GridPosition::BottomRight,
                LayerAnchor::BOTTOM | LayerAnchor::RIGHT,
            ),
        ];

        let test_resolutions = [
            (640, 360),   // Minimum supported
            (1280, 720),  // HD
            (1920, 1080), // FHD
            (2560, 1440), // WQHD
            (3840, 2160), // 4K
        ];

        let test_stages = [1u8, 2, 5, 8, 10];

        for (screen_w, screen_h) in test_resolutions {
            for stage in test_stages {
                let stage_info = get_size_stage(stage);
                let exp_w = stage_info.width.min(screen_w);
                let exp_h = stage_info.height.min(screen_h);
                let gutter = 32i32;

                for &(pos, expected_layer_anchor) in &all_positions {
                    let cfg = LayoutConfig {
                        grid_position: pos,
                        size_stage: stage,
                    };

                    let (anchor, (top, right, bottom, left), (w, h), scale) =
                        cfg.calculate_geometry(screen_w, screen_h);

                    assert_eq!(
                        anchor, expected_layer_anchor,
                        "Anchor mismatch for {:?} on {}x{}",
                        pos, screen_w, screen_h
                    );
                    assert_eq!(w, exp_w);
                    assert_eq!(h, exp_h);
                    assert_eq!(scale, stage_info.font_scale);
                    assert!(top >= 0 && right >= 0 && bottom >= 0 && left >= 0);

                    // Position specific boundary assertions
                    match pos {
                        GridPosition::TopLeft => {
                            assert_eq!((top, right, bottom, left), (gutter, 0, 0, gutter));
                        }
                        GridPosition::TopCenter => {
                            let exp_mid_left = ((screen_w as i32 - exp_w as i32) / 2).max(gutter);
                            assert_eq!((top, right, bottom, left), (gutter, 0, 0, exp_mid_left));
                        }
                        GridPosition::TopRight => {
                            assert_eq!((top, right, bottom, left), (gutter, gutter, 0, 0));
                        }
                        GridPosition::MiddleLeft => {
                            let exp_mid_top = ((screen_h as i32 - exp_h as i32) / 2).max(gutter);
                            assert_eq!((top, right, bottom, left), (exp_mid_top, 0, 0, gutter));
                        }
                        GridPosition::Center => {
                            let exp_mid_left = ((screen_w as i32 - exp_w as i32) / 2).max(gutter);
                            let exp_mid_top = ((screen_h as i32 - exp_h as i32) / 2).max(gutter);
                            assert_eq!(
                                (top, right, bottom, left),
                                (exp_mid_top, 0, 0, exp_mid_left)
                            );
                        }
                        GridPosition::MiddleRight => {
                            let exp_mid_top = ((screen_h as i32 - exp_h as i32) / 2).max(gutter);
                            assert_eq!((top, right, bottom, left), (exp_mid_top, gutter, 0, 0));
                        }
                        GridPosition::BottomLeft => {
                            assert_eq!((top, right, bottom, left), (0, 0, gutter, gutter));
                        }
                        GridPosition::BottomCenter => {
                            let exp_mid_left = ((screen_w as i32 - exp_w as i32) / 2).max(gutter);
                            assert_eq!((top, right, bottom, left), (0, 0, gutter, exp_mid_left));
                        }
                        GridPosition::BottomRight => {
                            assert_eq!((top, right, bottom, left), (0, gutter, gutter, 0));
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn test_geometry_determinism() {
        let (screen_w, screen_h) = (1920, 1080);
        let cfg = LayoutConfig {
            grid_position: GridPosition::Center,
            size_stage: 4,
        };

        let result1 = cfg.calculate_geometry(screen_w, screen_h);
        let result2 = cfg.calculate_geometry(screen_w, screen_h);
        let result3 = LayoutConfig::geometry_for(GridPosition::Center, 4, screen_w, screen_h);

        assert_eq!(result1, result2);
        assert_eq!(result1, result3);
    }

    #[test]
    fn test_load_from_str_migration_detection() {
        // 1. Legacy config: has anchor, margin_x, etc.
        let legacy_toml = r#"
[layout]
anchor = "TopRight"
margin_x = 20
margin_y = 20
width = 400
height = 200
"#;
        let (cfg, migrated) = Config::load_from_str(legacy_toml).expect("Should parse legacy config");
        assert!(migrated, "Legacy layout fields must trigger migration flag");
        assert_eq!(cfg.layout.grid_position, GridPosition::TopRight);

        // 2. Modern config: has grid_position and size_stage
        let modern_toml = r#"
[layout]
grid_position = "Center"
size_stage = 3
"#;
        let (cfg2, migrated2) = Config::load_from_str(modern_toml).expect("Should parse modern config");
        assert!(!migrated2, "Modern layout fields must NOT trigger migration flag");
        assert_eq!(cfg2.layout.grid_position, GridPosition::Center);
        assert_eq!(cfg2.layout.size_stage, 3);
    }
}
