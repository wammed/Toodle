# Toodle Feature Specification

This document details the functional capabilities and features of **Toodle**, the COSMIC Desktop Clock Widget.

---

## 1. Digital Clock & Date Display

- **High-Precision Clock**:
  - Displays hours, minutes, and seconds in `HH:MM:SS` format.
  - Sub-second synchronization stream guarantees precise alignment to wall-clock second boundaries without CPU spin-loops.
- **Full Date Display**:
  - Displays day of the week, full month name, day of the month, and year (e.g., `Monday, September 20, 2026`).
- **Dynamic Font Scaling**:
  - Scales cleanly from 30% (`0.3x`) to 1000% (`10.0x`) without pixelation or clipping.
- **Text Drop Shadow**:
  - Optional soft ambient text shadow (`Color::from_rgba(0, 0, 0, 0.65)`, offset: `(1.0, 2.0)`, blur: `6.0px`) for optimal readability over light or vibrant wallpapers.

---

## 2. Weather & Forecast Subsystem

- **Current Weather**:
  - Displays weather condition text (e.g., `Clear sky`, `Partly cloudy`, `Rain`) and current temperature.
  - Temperature unit toggleable between Celsius (`°C`) and Fahrenheit (`°F`).
- **7-Day Weekly Forecast Popup (`680 x 720`)**:
  - Accessible via the right-click context menu.
  - Displays cards for all 7 days with formatted dates (e.g., `Mon, Sep 21`).
  - Highlights `(Today)` in the current day card.
  - Shows minimum and maximum temperatures, weather conditions, and high-contrast styling.
- **Two-Tier Persistent Caching**:
  - Current weather is cached locally for 30 minutes.
  - Forecast data is cached locally for 3 hours.
  - Validates geographical coordinates (`is_location_match`) so changing cities immediately fetches fresh data.
  - Graceful offline fallback: if network requests fail, the last valid cached forecast is displayed automatically.

---

## 3. Popups & Menus (`Layer::Top`)

### 3.1 Context Menu (`340 x 380`)
- Opens on right-click anywhere on the clock widget.
- Elevated to `Layer::Top` with a solid dark background (`Color::from_rgb(0.13, 0.14, 0.18)`), border, and drop shadow.
- 18px font size, centered items:
  - `Calendar`: Opens the monthly calendar popup.
  - `Weekly Forecast`: Opens the 7-day forecast popup.
  - `Edit Layout`: Enters the desktop layout edit mode.
  - `Settings`: Launches the `toodle-settings` application.
  - `Quit`: Gracefully terminates the widget daemon.

### 3.2 Monthly Calendar Popup (`680 x 720`)
- Displays current month, year, and a clean 7-column calendar grid.
- Weekday headers: `Mon`, `Tue`, `Wed`, `Thu`, `Fri`, `Sat`, `Sun`.
- Interactive month navigation buttons:
  - `<` (Previous Month)
  - `Today` (Jump back to current month/day)
  - `>` (Next Month)
- Today's date highlighted in high-visibility suggested styling.

---

## 4. Desktop Edit Layout Mode

- Allows adjusting widget placement and dimensions directly on the desktop.
- Features:
  - Blue dashed/solid visual bounding box indicating the widget canvas.
  - `Save` button: Commits new layout and font scale to `config.toml` and returns to normal mode.
  - `Cancel` button: Reverts any in-progress changes to the saved configuration.
  - Zero screen flicker when entering or exiting edit mode.

---

## 5. Standalone Settings Application (`toodle-settings`)

A native XDG Toplevel application (`720 x 780`) providing 4 categorized sections:

### 5.1 Appearance Tab
- **Theme Presets Grid**:
  - 10 Presets with font family labels: `Modern`, `Classic`, `Digital Mono`, `Minimal`, `Cyberpunk`, `Nord`, `Warm Sunset`, `Forest`, `Slate`, `Rose Gold`.
  - Clicking a preset immediately updates both the font and default accent color on the active widget.
- **16-Color Palette Grid**:
  - Interactive swatches with checkmarks (`✓`) and high-contrast borders.
  - Curated colors: Pure White, Soft Silver, Cool Slate, Sky Blue, COSMIC Blue, Indigo, Purple, Rose Pink, Crimson, Coral Red, Orange, Amber Gold, Sun Yellow, Lime, Emerald Green, Teal Cyan.
- **Font Scale Slider**: Range from `30%` to `1000%` in `5%` increments.
- **Text Shadow Toggle**: Instantly toggles ambient text shadow.

### 5.2 Layout Tab
- **Desktop Anchor**: Four anchor buttons: `TopLeft`, `TopRight`, `BottomLeft`, `BottomRight`.
- **Margin X & Margin Y Sliders**:
  - Range: `0..=2560 px` (X) and `0..=1440 px` (Y) with 2px precision.
  - Minimum clamped to `0` to prevent widgets from sliding off-screen into negative coordinates.
- **Width & Height Sliders**:
  - Width: `150..=1200 px` in 5px steps.
  - Height: `60..=800 px` in 5px steps.
- **Live Updates**: Moving any slider immediately writes to `config.toml`, moving the widget on the desktop in real time.

### 5.3 Weather Tab
- **Location Inputs**: `Location Name`, `Latitude`, and `Longitude` text fields.
- **Quick City Presets**:
  - One-click buttons: `Tokyo`, `Gifu`, `London`, `New York`, `Paris`.
  - Active city highlighted with a `✓` badge.
  - Immediately persists to config, bypasses cache, and refreshes weather live.
- **Apply Location & Refresh Weather Button**: Commits manually entered coordinates immediately.
- **Temperature Unit**: Toggle between `Celsius (°C)` and `Fahrenheit (°F)`.

### 5.4 Display Tab
- Allows specifying target Wayland output (e.g. `DP-1`, `HDMI-A-1`).
- Empty string defaults to primary/active output.

---

## 6. Embedded Fonts (Design Doc Sec 10)

All fonts are statically compiled into the binary via `include_bytes!`:
1. **Roboto Sans** (Regular & Bold) — Modern & Forest themes.
2. **JetBrains Mono** (Regular & Bold) — Digital Mono, Cyberpunk, Slate themes.
3. **DejaVu Serif** (Regular & Bold) — Classic, Warm Sunset, Rose Gold themes.
4. **Open Sans** (Regular & Bold) — Minimal, Nord themes.
