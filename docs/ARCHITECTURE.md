# Toodle Architecture & Technical Design Document

<p align="center">
  <strong>English</strong> | <a href="ARCHITECTURE.ja.md">日本語</a> | <a href="PORTAL.md">📚 Documentation Portal</a> | <a href="../README.md">← Root README</a>
</p>

---

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
  let info = crate::config::get_size_stage(config.layout.size_stage);
  vec![Rectangle {
      x: 0.0,
      y: 0.0,
      width: info.width as f32,
      height: info.height as f32,
  }]
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
This guarantees instantaneous 0ms visual updates with zero compositor redraw flicker during discrete layout or size stage selection, settings reload, or mode transitions.

### 2.4 Edit Layout Architecture (Two-Surface Model, Design Doc Sec 7)
Rather than morphing the main widget into an editing interface, Toodle separates presentation from interaction:
- **Bottom Surface**: The actual running `Toodle` widget on `Layer::Bottom`.
- **Top Surface**: An independent `Edit Layout Panel` on `Layer::Top` providing a 3×3 grid position button matrix, a 10-stage size selection matrix (Stages 1..=5 and 6..=10), Save, and Cancel.
- Moving controls in the top panel sends live layer surface commands to the bottom surface.
- **Save**: Commits values to `config.toml`, closes the panel, and restores normal input bounds.
- **Cancel**: Discards edits, reverts the bottom surface via layer commands, and closes the panel.

### 2.5 9-Zone Display Layout & 10 Discrete Size Stages (WQHD Support)
To eliminate redraw flicker and ghosting/blur artifacts caused by continuous slider resizing, Toodle uses a structured 9-zone display layout and discrete size stages:
- **9-Zone Display Grid (`GridPosition`)**:
  - `TopLeft`, `TopCenter`, `TopRight`, `MiddleLeft`, `Center`, `MiddleRight`, `BottomLeft`, `BottomCenter`, `BottomRight`.
  - Outer screen gutter is fixed at `32px`. Center alignment mathematically positions the window at `((screen_w - w) / 2).max(gutter)` and `((screen_h - h) / 2).max(gutter)`.
  - Content horizontal alignment (`align_x`) adapts dynamically: left positions align left (`Alignment::Start`), center positions center horizontally (`Alignment::Center`), and right positions align right (`Alignment::End`).
  - Container vertical alignment (`align_y`) is unified to `Alignment::Start` (top padding), preventing downward sagging when anchored to the screen bottom.
- **10 Discrete Size Stages (`SIZE_STAGES`)**:
  - Predefined dimensions tailored for displays up to WQHD 2560×1440 (Stage 1: 280×130 up to Stage 10: 2060×920).
  - Window heights are sized with generous headroom to fully enclose oversized typography (clock size: 38..310px, date/weather size: 14..110px) and weather icon bounding boxes, preventing any bottom clipping.
  - Window dimensions and font sizes synchronize 1:1, eliminating the need for a separate font scale slider.

### 2.6 Multi-Monitor & Output State Machine (`OutputManager`)
Toodle manages Wayland outputs dynamically through `OutputManager`:
- Tracks detected `WlOutput` instances and extracts clean connector names (`DP-1`, `HDMI-A-1`) directly from Wayland compositor metadata (`OutputEvent::Created`, `OutputEvent::InfoUpdate`). `cosmic-randr` is utilized for system display enumeration, user-facing display settings labels, and fallback startup resolution detection.
- Dynamically resolves `DisplayConfig::output`:
  - If a specific output name is configured and connected, Toodle targets that exact output.
  - If the configured output is disconnected or unset, Toodle seamlessly falls back to the active/primary or first available output.
  - When the configured output reconnects (hot-plug), Toodle automatically rebinds.
- Target dimensions (`target_logical_size`) are tracked per output, completely isolating geometry calculations across multi-monitor setups without global screen dimension contamination. Surface recreation is deferred until confirmed geometry is received via `InfoUpdate`, avoiding transient double-recreation.

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

### 4.1 Strict Response Parsing, Generation Tracking & Coordinate Validation
To ensure robustness against upstream API changes, stale asynchronous responses, and invalid user input:
- **Strict Parsing**: JSON responses are rigorously validated. Missing `temperature_2m` or `weather_code` returns `WeatherError::Parse` rather than silently defaulting to `0.0°C` / `Clear sky`. Daily forecasts require a present `daily` object with matching array lengths (`time`, `weather_code`, `temperature_2m_max`, `temperature_2m_min`) covering at least 7 forecast days.
- **Request Generation Tracking (`WeatherStateManager`)**: Every outgoing weather fetch increments a monotonic `weather_generation` sequence ID. When an asynchronous response arrives, any response whose generation does not match the active generation is dropped, completely preventing stale responses from previous locations or outdated retries from overwriting current state.
- **Startup Cache Location & Freshness Check**: During startup initialization, cached data is validated against the active configured coordinates via `is_location_match()` and freshness via `is_current_valid()`, ensuring outdated or mismatched locations are never rendered on launch.
- **Concurrent Atomic Cache Writes**: Cache saving writes to a unique temporary file (`.pid.nanos.tmp`) before atomic renaming to prevent concurrent write corruption, and logs descriptive diagnostic warnings on persistence failures.
- **Coordinate Bounds**: `latitude` (`-90.0..=90.0`) and `longitude` (`-180.0..=180.0`) are validated for finite numeric bounds before initiating network requests.
- **Clock-Skew Protection**: Cache freshness validation rejects negative age elapsed (`now < cached_at`), preventing stale cache freezing on system clock adjustments.
- **Location Matching**: `CachedWeather` records `latitude`, `longitude`, `cached_at`, and `data`. `is_location_match(lat, lon)` validates coordinates within `0.02` degrees (~2 km). Switching coordinates immediately bypasses outdated cache.

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
   - Coalesces rapid sequential events with a 50ms event coalescing delay and event queue drain.
   - Loads new `Config`, automatically migrates legacy configuration fields (`anchor`, `margin_x`, etc.) to modern `grid_position` & `size_stage`, and rewrites the clean modern TOML structure back to disk.
   - Compares against current in-memory config and applies updates:
     - Margin / Anchor / Size adjustments -> Immediate Wayland layer updates.
     - Theme / Color / Font Scale / Shadow -> Widget visual repaint.
     - Location change -> Weather display reset to loading `"..."`, generation incremented, and fresh data fetched.
     - Edit mode protection -> If user is actively in interactive Edit mode, active edit coordinates are preserved to prevent edit cancellation.

---

## 6. Embedded Typography System

Per Section 11 of the design specifications, external system fonts are strictly avoided to ensure identical metrics across any Linux distribution:

```rust
pub const ROBOTO_REGULAR_BYTES: &[u8] = include_bytes!("../../resources/fonts/Roboto-Regular.ttf");
pub const JETBRAINS_REGULAR_BYTES: &[u8] = include_bytes!("../../resources/fonts/JetBrainsMono-Regular.ttf");
pub const DEJAVUSERIF_REGULAR_BYTES: &[u8] = include_bytes!("../../resources/fonts/DejaVuSerif-Regular.ttf");
pub const OPENSANS_REGULAR_BYTES: &[u8] = include_bytes!("../../resources/fonts/OpenSans-Regular.ttf");
```

All text font variants are injected into `embedded_fonts()` during application startup. Weather condition icons are rendered via embedded Bas Milius Meteocons color SVG assets (`resources/icons/meteocons/`, MIT License).

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
   svg_bytes() (&'static [u8])
       │
       ▼
cosmic::iced::widget::svg (resvg vector renderer)
       │
       ▼
   COSMIC UI (vibrant color SVG scaled to layout stage)
```

Normal text (time, date, temperature, condition text) uses the theme's selected font, while the weather icon is rendered via `svg` using embedded Meteocons color SVGs, ensuring identical, vibrant display across all environments and size stages.

---

## 7. Known Implementation Gaps (Design Doc Sec 25)

The following areas are explicitly documented as known gaps between the v0.3 design and current code:
1. **Display Output Binding**: **Resolved**. Implemented via `OutputManager` state machine; supports specific connector matching, automatic fallback to active/primary output, hot-plug re-attachment, and per-output geometry calculation. Surface recreation is deferred until confirmed geometry arrives via `InfoUpdate`.
2. **Decoupled Weather TTL**: Current weather (30m) and forecast (3h) cache validity are defined, but fetch operations currently run bundled.
3. **HTTP Exponential Backoff**: Automatic retry backoff for HTTP 429/5xx is not yet implemented.
4. **Geocoding**: City presets and manual coordinates are supported; automatic name-to-coordinate lookup is planned for Phase 5.
5. **Popup Dismissal**: Outside-click and focus-loss dismiss behavior are awaiting compositor event verification and formalization.
6. **Multi-Monitor Verification Status**:
   - **Hardware Verified**: Dual identical 2560×1440 monitors (`DP-1` and `DP-2`) under Wayland have been verified on physical hardware for target switching, correct display position, and correct display size.
   - **Unverified (Future Robustness)**: Displays with differing resolutions (e.g. 1080p + 4K), mixed-DPI multi-monitor environments, and fractional scaling have not been tested on physical hardware and remain future robustness targets.

---

## 8. Build & Local Install Separation (`tools/install-local.sh`)

Toodle enforces a clear separation between compilation and local system modifications:
- **`cargo build --release`**:
  - Compiles the Rust binaries only.
  - Does not alter the user environment (no auto-copy to `$HOME/.local/bin`).
- **`./tools/install-local.sh`**:
  - Explicitly executed by the developer to build and copy release binaries into `$HOME/.local/bin`.
- **Future Packaging**:
  - Provides a clean architecture ready for distribution packaging (`.deb`, RPM, Arch PKGBUILD) as decoupled pipelines.

---

<p align="center">
  <a href="PORTAL.md">📚 Documentation Portal</a> | <a href="../README.md">← Root README</a> | <a href="ARCHITECTURE.ja.md">日本語設計書はこちら →</a>
</p>

