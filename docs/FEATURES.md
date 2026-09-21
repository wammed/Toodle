# Toodle Feature Specification

This document details the functional capabilities and features of **Toodle**, the COSMIC Desktop Clock Widget.

> **Specification Baseline**: Conforms to [`docs/Drafts/Toodle-Design-Docs-v0.3.md`](file:///home/susie/GitHUB/wammed/Toodle/docs/Drafts/Toodle-Design-Docs-v0.3.md) (v0.3 Phase 0 Real-World Verification Baseline).

---

## 1. Digital Clock & Date Display

- **High-Precision Clock**:
  - Displays hours, minutes, and seconds in 24-hour `HH:MM:SS` format.
  - Sub-second synchronization stream guarantees precise alignment to wall-clock second boundaries without CPU spin-loops.
- **Full Date Display**:
  - Displays day of the week, full month name, day of the month, and year (e.g., `Monday, September 20, 2026`).
  - Formatted in English (US).
- **Unified Typography Scaling**:
  - Automatically synchronizes time (38..310px), date (14..110px), spacing, and padding across 10 discrete size stages without pixelation, blur, or bottom clipping.
- **Text Drop Shadow**:
  - Optional soft ambient text shadow (`Color::from_rgba(0, 0, 0, 0.65)`, offset: `(1.0, 2.0)`, blur: `6.0px`) for optimal readability over light or vibrant wallpapers.

---

## 2. Weather & Forecast Subsystem

- **Current Weather**:
  - Displays weather condition text (e.g., `Clear sky`, `Partly cloudy`, `Rain`) and current temperature.
  - Temperature unit toggleable between Celsius (`°C`) and Fahrenheit (`°F`).
  - Temporary `"..."` loading indicator during location transition.
- **7-Day Weekly Forecast Popup (`680 x 720`)**:
  - Accessible via the right-click context menu.
  - Displays cards for 7 days with formatted dates (e.g., `Mon, Sep 21`).
  - Highlights `(Today)` in the current day card.
  - Shows minimum and maximum temperatures, weather conditions, precipitation probability, and weather icons.
  - Solid dark styling for consistent visibility.
- **Two-Tier Persistent Caching**:
  - Current weather cache policy: 30 minutes validity.
  - Forecast cache policy: 3 hours validity.
  - Geographical coordinate validation (`is_location_match`) ensures that switching cities immediately bypasses outdated cache.
  - Graceful offline fallback: if network requests fail, the last valid cached data for the current coordinates is displayed automatically.

---

## 3. Popups & Menus (`Layer::Top`)

All popups are independent `Layer::Top` surfaces with solid dark styling (`Color::from_rgb(0.12, 0.13, 0.17)`), crisp borders, and ambient drop shadows.

### 3.1 Context Menu (`340 x 380`)
- Opens on right-click anywhere on the clock widget.
- Clean centered 18px actions:
  - `Calendar`: Opens the monthly calendar popup.
  - `Weekly Forecast`: Opens the 7-day forecast popup.
  - `Edit Layout`: Enters desktop layout edit mode.
  - `Settings`: Launches the `toodle-settings` application.
  - `Quit`: Gracefully terminates the widget daemon.

### 3.2 Monthly Calendar Popup (`680 x 720`)
- Displays current month, year, and a clean 7-column calendar grid.
- Weekday headers: `Mon`, `Tue`, `Wed`, `Thu`, `Fri`, `Sat`, `Sun`.
- Interactive month navigation buttons:
  - `<` (Previous Month)
  - `Today` (Jump back to current month/day)
  - `>` (Next Month)
- Today's date highlighted with a suggested accent badge.

---

## 4. Desktop Edit Layout Mode (Two-Surface Architecture)

Rather than morphing the widget itself into an edit UI, Toodle employs a two-surface system:
- **Architecture**:
  - **Bottom Surface**: The actual `Toodle` widget on `Layer::Bottom`.
  - **Top Surface**: An independent `Edit Layout Panel` on `Layer::Top`.
- **Controls Available on Panel**:
  - **9-Zone Display Grid**: 3×3 matrix buttons (`TopLeft`, `TopCenter`, `TopRight`, `MiddleLeft`, `Center`, `MiddleRight`, `BottomLeft`, `BottomCenter`, `BottomRight`).
  - **10 Discrete Size Stages (WQHD-ready)**: 2 rows of size preset buttons (`1: 280`, `2: 380`, `3: 490`, `4: 620`, `5: 780` / `6: 960`, `7: 1180`, `8: 1440`, `9: 1740`, `10: 2060`).
  - `Save` button: Commits changes to `config.toml`, closes panel, restores input bounds.
  - `Cancel` button: Discards edits, restores saved state via in-place layer commands, closes panel.
- **Input Region Handling**:
  - During edit mode, the widget surface expands its input region to the full surface (`None`).
  - On exit, input region returns to the `content_bounds()` bounding box.
- **Zero Flicker**:
  - Button selections apply immediately to the bottom widget using in-place layer commands (`set_margin`, `set_size`, `set_anchor`) without destroying or recreating surfaces.

---

## 5. Standalone Settings Application (`toodle-settings`)

A native XDG Toplevel application (`720 x 780`, solid dark background) providing 4 categorized sections:

### 5.1 Appearance Tab
- **Theme Presets Grid**:
  - 10 Presets with font family labels: `Modern` (Roboto), `Classic` (DejaVu Serif), `Digital Mono` (JetBrains Mono), `Minimal` (Open Sans), `Cyberpunk` (JetBrains Mono), `Nord` (Open Sans), `Warm Sunset` (DejaVu Serif), `Forest` (Roboto), `Slate` (JetBrains Mono), `Rose Gold` (DejaVu Serif).
  - Clicking a preset immediately updates both the font and default accent color on the active widget.
- **16-Color Palette Grid**:
  - Interactive swatches with checkmarks (`✓`) and high-contrast borders.
  - Curated colors: Pure White, Soft Silver, Cool Slate, Sky Blue, COSMIC Blue, Indigo, Purple, Rose Pink, Crimson, Coral Red, Orange, Amber Gold, Sun Yellow, Lime, Emerald Green, Teal Cyan.
- **Text Shadow Toggle**: Instantly toggles ambient text shadow.

### 5.2 Layout Tab
- **9-Zone Display Layout**: 3×3 grid buttons (`TopLeft` through `BottomRight`).
- **10-Stage Size Selector**: 10 preset dimensions optimized for desktop screens. Font sizes (time, date, weather) automatically and proportionally synchronize with window size.
- **Live In-Place Updates**: Clicking buttons writes atomically to `config.toml`, updating the desktop widget in real time via inotify.

### 5.3 Weather Tab
- **Location Inputs**: `Location Name`, `Latitude`, and `Longitude` text fields.
- **Quick City Presets**: One-click buttons: `Tokyo`, `Gifu`, `London`, `New York`, `Paris`.
- **Temperature Unit**: Toggle between `Celsius (°C)` and `Fahrenheit (°F)`.
- **Apply & Refresh Button**: Instantly forces a fresh weather fetch.

### 5.4 Display Tab
- **Target Display Output**: Wayland output name (e.g. `"DP-1"`).
- *(Note: Stored in config; output binding is a roadmap item; widget currently attaches to `Active Output`)*.

---

## 6. Build & Local Installation (`tools/install-local.sh`)

- **`cargo build --release`**: Only builds the Rust binaries without touching the user's home directory or system paths.
- **`./tools/install-local.sh`**: Explicitly executed by developers to build and install the release binaries into `$HOME/.local/bin`.
