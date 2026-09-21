# Toodle Architecture & Technical Design Document

This document provides a comprehensive technical overview of the **Toodle** COSMIC Desktop Clock Widget system, detailing its surface management, Wayland layer-shell protocol usage, asynchronous tick streams, weather caching, inotify configuration synchronization, and embedded font infrastructure.

> **Baseline Design Reference**: This architecture aligns with [`docs/Drafts/Toodle-Design-Docs-v0.3.md`](file:///home/susie/GitHUB/wammed/Toodle/docs/Drafts/Toodle-Design-Docs-v0.3.md) (v0.3 Phase 0 Real-World Verification Baseline). Real-world verification on COSMIC compositor (`cosmic-comp`) takes precedence over earlier draft assumptions.

---

## 1. System Overview

Toodle is built specifically for the System76 COSMIC desktop environment (Wayland compositor `cosmic-comp`) using the official `libcosmic` framework (which extends `iced` with Wayland layer-shell capabilities).

The architecture is split into two independent binaries:
1. **`toodle` (Widget Daemon & Independent Layer Shell Popups)**:
   - A persistent, lightweight desktop daemon creating a `Layer::Bottom` surface on the Wayland desktop wallpaper.
   - Dynamically manages independent `Layer::Top` surfaces: Context Menu, Calendar, Weekly Forecast, and Edit Layout Panel (`xdg_popup` is explicitly not used).
   - Watches `~/.config/toodle/config.toml` via `inotify` and dynamically reloads configuration with zero screen flicker via in-place Wayland layer protocol commands.
2. **`toodle-settings` (XDG Toplevel Configuration App)**:
   - A standalone graphical settings application launched independently or from the right-click menu.
   - Provides tabbed configuration for Appearance, Layout, Weather, and Display Output.
   - Saves settings atomically (`.tmp` write followed by `rename`), immediately triggering live updates in the active widget.

---

## 2. Wayland Surface Architecture (`wlr-layer-shell`)

```text
┌───────────────────────────────────────────────────────────────┐
│ Wayland Output (e.g., DP-1: 2560 x 1440)                      │
│                                                               │
│ [ Layer::Overlay ]                                            │
│   (Lock screens, screen capture overlays)                     │
│                                                               │
│ [ Layer::Top ]                                                │
│   - Context Menu (340 x 380, solid background)                │
│   - Monthly Calendar (680 x 720, solid background)            │
│   - Weekly Forecast (680 x 720, solid background)             │
│   - Edit Layout Panel (independent top surface)               │
│                                                               │
│ [ Regular Windows (XDG Toplevel) ]                            │
│   - Applications, Browsers, Terminals                         │
│   - toodle-settings (720 x 780, solid background)             │
│                                                               │
│ [ Layer::Bottom ]                                             │
│   - Toodle Clock Widget Surface (e.g., 280 x 140)             │
│     * Transparent background                                  │
│     * Anchor: TopRight (configurable)                         │
│     * Input Zone: content_bounds() (Normal) / full (Edit)     │
│                                                               │
│ [ Layer::Background ]                                         │
│   - COSMIC Wallpaper                                          │
└───────────────────────────────────────────────────────────────┘
```

### 2.1 Layer Hierarchy
- **Widget Placement (`Layer::Bottom`)**: Placed on top of the wallpaper but underneath normal application windows (browsers, terminals, file managers). Maximizing or moving standard windows naturally covers the widget.
- **Top Surfaces (`Layer::Top`)**: Context Menu, Calendar, Weekly Forecast, and the Edit Layout Panel are created on demand as independent `Layer::Top` surfaces to sit above regular applications.
- **Exclusive Zone**: The widget explicitly sets `exclusive_zone(0)`, preventing it from pushing desktop panels or resizing application work areas.
- **Solid Styling**: All top popups and `toodle-settings` use an opaque dark solid background (`Color::from_rgb(0.12, 0.13, 0.17)`) to ensure high contrast and readability regardless of user wallpaper.

### 2.2 Input Region & Click Passthrough (Design Doc Sec 8)
Wayland desktop widgets with transparent window areas can inadvertently intercept user clicks destined for desktop icons or wallpaper menus. Toodle controls the input zone via `cosmic::iced::wayland::layer_cmd::set_input_zone`:
- **Normal Mode**:
  Calculates `content_bounds()` representing the active widget layout bounding box:
  ```rust
  let padding = 12.0;
  let estimated_w = (180.0 * config.appearance.font_scale + padding * 2.0).min(config.layout.width as f32);
  let estimated_h = (90.0 * config.appearance.font_scale + padding * 2.0).min(config.layout.height as f32);
  Rectangle { x: 0.0, y: 0.0, width: estimated_w, height: estimated_h }
  ```
  Transparent margins outside the active widget layout are excluded from the input region, allowing desktop clicks to pass through freely.
  *(Note: Per v0.3 design Section 8, non-rectangular glyph-level hit masks are not required; rectangular `content_bounds()` is the official specification).*
- **Edit Mode**:
  Expands the widget surface input region to the entire surface (`None`) so user interactions cannot miss the frame, returning to `content_bounds()` upon exit.

### 2.3 Zero-Flicker In-Place Surface Updates
Destroying and recreating Wayland surfaces causes visual stutter and screen flicker on compositors. Toodle exclusively issues in-place incremental Wayland protocol requests:
```rust
tasks.push(layer_cmd::set_anchor(surface_id, anchor));
tasks.push(layer_cmd::set_margin(surface_id, top, right, bottom, left));
tasks.push(layer_cmd::set_size(surface_id, Some(width), Some(height)));
tasks.push(layer_cmd::set_input_zone(surface_id, Some(bounds)));
```
This guarantees instantaneous 0ms visual updates with zero compositor redraw flicker during live slider dragging, settings reload, or mode transitions.

### 2.4 Edit Layout Architecture (Two-Surface Model, Design Doc Sec 7)
Rather than morphing the main widget into an editing interface, Toodle separates presentation from interaction:
- **Bottom Surface**: The actual running `Toodle` widget on `Layer::Bottom`.
- **Top Surface**: An independent `Edit Layout Panel` on `Layer::Top` providing controls for Anchor, Margin X/Y, Width, Height, Font Scale, Save, and Cancel.
- Moving controls in the top panel sends live layer surface commands to the bottom surface.
- **Save**: Commits values to `config.toml`, closes the panel, and restores normal input bounds.
- **Cancel**: Discards edits, reverts the bottom surface via layer commands, and closes the panel.

---

## 3. Clock Synchronizer Pipeline

To avoid second-skipping or erratic timer drift, Toodle implements a microsecond-precision clock subscription:

```rust
pub fn clock_tick_stream() -> impl futures::Stream<Item = DateTime<Local>> {
    futures::stream::unfold((), |_| async {
        let now = Local::now();
        let dur = duration_until_next_second(now);
        tokio::time::sleep(dur).await;
        let next_now = Local::now();
        Some((next_now, ()))
    })
}
```

Where `duration_until_next_second` calculates:
```rust
let nanos = now.timestamp_subsec_nanos();
let remaining_nanos = 1_000_000_000u32 - nanos;
Duration::from_nanos(remaining_nanos as u64)
```
This ensures each update tick fires precisely at `XX:XX:XX.000000`, matching the hardware wall clock without CPU spin-polling.

---

## 4. Weather Subsystem & Two-Tier Caching

```text
┌─────────────────┐       Cache Hit (<30m)
│  WeatherService ├──────────────────────────────► [ Return Cached Data ]
└────────┬────────┘
         │
         │ Cache Miss / Expired / Location Changed
         ▼
┌──────────────────────┐  HTTP 200 (JSON)   ┌───────────────────────────┐
│  OpenMeteoProvider   ├───────────────────►│ Open-Meteo REST API       │
│  (reqwest + tokio)   │                    │ (api.open-meteo.com)      │
└────────┬─────────────┘                    └───────────────────────────┘
         │
         │ Network Failure / Offline
         ▼
┌───────────────────────────┐
│ Stale Cache Fallback      │
│ (Return last valid cache) │
└───────────────────────────┘
```

### 4.1 Strict Location Coordinate Validation
To prevent returning stale cache data when a user switches locations (e.g. from Tokyo to London):
- `CachedWeather` records `latitude`, `longitude`, `cached_at`, and `data`.
- `is_location_match(lat, lon)` validates coordinates within `0.02` degrees (~2 km):
  ```rust
  pub fn is_location_match(&self, lat: f64, lon: f64) -> bool {
      (self.latitude - lat).abs() < 0.02 && (self.longitude - lon).abs() < 0.02
  }
  ```
- If coordinates do not match, the cache is bypassed immediately, triggering a fresh fetch for the newly requested coordinates.

### 4.2 Cache Retention Policy (Design Doc Sec 14)
- **Current Weather**: 30-minute validity window.
- **7-Day Forecast**: 3-hour validity window.
- **Offline Fallback**: If the network is unreachable, the system gracefully falls back to the most recent cached data for the active coordinates.
- *(Implementation Gap Note: While the TTL parameters are defined, decoupling fetch cycles between current weather and weekly forecast remains a known roadmap item).*

---

## 5. Inotify IPC Synchronization & Hot-Reload

Communication between `toodle-settings` and `toodle` is decoupled via `~/.config/toodle/config.toml`:

1. **Atomic Write in `toodle-settings`**:
   ```rust
   let tmp_path = path.with_extension("tmp");
   fs::write(&tmp_path, serialized_toml)?;
   fs::rename(&tmp_path, &path)?;
   ```
2. **Inotify Event in `toodle` Daemon**:
   - `notify::recommended_watcher` receives a rename / create event on `config.toml`.
   - Flushes after a short 25ms delay to ensure file write integrity.
   - Loads new `Config`, compares against current in-memory config, and applies updates:
     - Margin / Anchor / Size adjustments -> Immediate Wayland layer updates.
     - Theme / Color / Font Scale / Shadow -> Widget visual repaint.
     - Location change -> Weather display reset to loading `"..."` and fresh data fetched.

---

## 6. Embedded Typography System

Per Section 11 of the design specifications, external system fonts are strictly avoided to ensure identical metrics across any Linux distribution:

```rust
pub const ROBOTO_REGULAR_BYTES: &[u8] = include_bytes!("../../resources/fonts/Roboto-Regular.ttf");
pub const JETBRAINS_REGULAR_BYTES: &[u8] = include_bytes!("../../resources/fonts/JetBrainsMono-Regular.ttf");
pub const DEJAVUSERIF_REGULAR_BYTES: &[u8] = include_bytes!("../../resources/fonts/DejaVuSerif-Regular.ttf");
pub const OPENSANS_REGULAR_BYTES: &[u8] = include_bytes!("../../resources/fonts/OpenSans-Regular.ttf");
pub const WEATHER_ICONS_BYTES: &[u8] = include_bytes!("../../resources/fonts/WeatherIcons.ttf");
```

All text font variants and the Weather Icons font (`resources/fonts/WeatherIcons.ttf`, SIL Open Font License 1.1) are injected into `embedded_fonts()` during application startup.

### Weather Icon Pipeline

Weather state representation completely avoids Unicode emoji fallback, following a dedicated abstraction pipeline:

```text
WMO weather code
       │
       ▼
  WeatherIcon (semantic weather category)
       │
       ├────────────────► condition text (theme font)
       │
       ▼
Weather Icons glyph (char)
       │
       ▼
 FONT_WEATHER_ICONS (Weather Icons font)
       │
       ▼
   COSMIC UI (themed foreground color)
```

Normal text (time, date, temperature, condition text) uses the theme's selected font, while the weather icon glyph is rendered exclusively via `FONT_WEATHER_ICONS` (`Weather Icons`), ensuring identical glyph display across all environments.

---

## 7. Known Implementation Gaps (Design Doc Sec 25)

The following areas are explicitly documented as known gaps between the v0.3 design and current code:
1. **Display Output Binding**: `DisplayConfig::output` is stored in configuration, but `toodle` currently binds to `IcedOutput::Active`.
2. **Decoupled Weather TTL**: Current weather (30m) and forecast (3h) cache validity are defined, but fetch operations currently run bundled.
3. **HTTP Exponential Backoff**: Automatic retry backoff for HTTP 429/5xx is not yet implemented.
4. **Geocoding**: City presets and manual coordinates are supported; automatic name-to-coordinate lookup is planned for Phase 5.
5. **Popup Dismissal**: Outside-click and focus-loss dismiss behavior are awaiting compositor event verification and formalization.
