# Toodle Architecture & Technical Design Document

This document provides a comprehensive technical overview of the **Toodle** COSMIC Desktop Clock Widget system, detailing its surface management, Wayland layer-shell protocol usage, asynchronous tick streams, weather caching, inotify configuration synchronization, and embedded font infrastructure.

---

## 1. System Overview

Toodle is built specifically for the System76 COSMIC desktop environment (Wayland compositor `cosmic-comp`) using the official `libcosmic` framework (which extends `iced` with Wayland layer-shell capabilities).

The architecture is split into two primary components:
1. **`toodle` (Widget Daemon & Layer Shell Popups)**:
   - A persistent, lightweight desktop daemon creating a `Layer::Bottom` surface on the Wayland desktop background.
   - Dynamically renders `Layer::Top` popups: Context Menu, Calendar, Weekly Forecast, and Edit Layout mode.
   - Watches `~/.config/toodle/config.toml` via `inotify` and dynamically reloads configuration with zero screen flicker.
2. **`toodle-settings` (XDG Toplevel Configuration App)**:
   - A standalone graphical settings application launched independently or from the right-click menu.
   - Provides tabbed configuration for Appearance, Layout, Weather, and Display Output.
   - Saves settings atomically, immediately triggering live updates in the active widget.

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
│   - Toodle Context Menu (340 x 380)                           │
│   - Toodle Monthly Calendar (680 x 720)                       │
│   - Toodle Weekly Forecast (680 x 720)                        │
│   - Toodle Edit Layout Mode Overlay Controls                  │
│                                                               │
│ [ Regular Windows (XDG Toplevel) ]                            │
│   - Applications, Browsers, Terminals                         │
│   - toodle-settings (720 x 780)                               │
│                                                               │
│ [ Layer::Bottom ]                                             │
│   - Toodle Clock Widget Surface (280 x 140)                   │
│     * Anchor: TopRight (configurable)                         │
│     * Non-rectangular input zone (click passthrough)          │
│                                                               │
│ [ Layer::Background ]                                         │
│   - COSMIC Wallpaper                                          │
└───────────────────────────────────────────────────────────────┘
```

### 2.1 Layer Hierarchy
- **Widget Placement (`Layer::Bottom`)**: Ensures normal application windows (browsers, terminals, file managers) cover the widget when maximized or dragged over it, keeping the desktop tidy while remaining visible on an empty workspace.
- **Popups Placement (`Layer::Top`)**: Popups (ContextMenu, Calendar, Forecast, and the Edit Layout overlay) are elevated above standard application windows so users can interact with them without obstruction.
- **Exclusive Zone**: The widget explicitly sets `exclusive_zone(0)`, preventing it from pushing desktop panels or resizing application work areas.

### 2.2 Non-Rectangular Input Region & Click Passthrough
A common issue with desktop widgets on Wayland is that transparent window areas block user clicks from reaching desktop icons and the desktop context menu.

Toodle solves this using `cosmic::iced::wayland::layer_cmd::set_input_zone`:
- Calculates `content_bounds`:
  ```rust
  let padding = 12.0;
  let estimated_w = (180.0 * config.appearance.font_scale + padding * 2.0).min(config.layout.width as f32);
  let estimated_h = (90.0 * config.appearance.font_scale + padding * 2.0).min(config.layout.height as f32);
  Rectangle { x: 0.0, y: 0.0, width: estimated_w, height: estimated_h }
  ```
- Transparent padding outside of the active text area is excluded from the input region, allowing desktop clicks to pass through freely.

### 2.3 Zero-Flicker In-Place Surface Manipulation
Rather than destroying and recreating the Wayland surface during configuration reload or mode transitions (which induces visual flash / stutter), Toodle issues incremental Wayland protocol requests:
```rust
tasks.push(layer_cmd::set_anchor(surface_id, anchor));
tasks.push(layer_cmd::set_margin(surface_id, top, right, bottom, left));
tasks.push(layer_cmd::set_size(surface_id, Some(width), Some(height)));
tasks.push(layer_cmd::set_input_zone(surface_id, Some(bounds)));
```
This guarantees 0ms visual transitions with zero compositor redraw stutter.

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
- If coordinates do not match, the cache is bypassed immediately, triggering a fresh fetch for the newly requested city.

### 4.2 Cache Retention Policy (Design Doc Sec 13)
- **Current Weather**: 30-minute validity window.
- **7-Day Forecast**: 3-hour validity window.
- **Offline Fallback**: If the network is unreachable, the system gracefully falls back to the most recent cached data for the current coordinates before returning an error state.

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

Per Section 10 of the design specifications, external system fonts are strictly avoided to ensure identical metrics across any Linux distribution:

```rust
pub const ROBOTO_REGULAR_BYTES: &[u8] = include_bytes!("../../resources/fonts/Roboto-Regular.ttf");
pub const JETBRAINS_REGULAR_BYTES: &[u8] = include_bytes!("../../resources/fonts/JetBrainsMono-Regular.ttf");
pub const DEJAVUSERIF_REGULAR_BYTES: &[u8] = include_bytes!("../../resources/fonts/DejaVuSerif-Regular.ttf");
pub const OPENSANS_REGULAR_BYTES: &[u8] = include_bytes!("../../resources/fonts/OpenSans-Regular.ttf");
```

All 8 font variants are injected into `embedded_fonts()` during application startup and resolved dynamically through `crate::config::theme::get_font_pair_for_theme()`.
