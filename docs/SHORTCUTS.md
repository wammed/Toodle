# 🖱️ Toodle Controls & Interaction Guide (SHORTCUTS)

<p align="center">
  <strong>English</strong> | <a href="SHORTCUTS.ja.md">日本語</a> | <a href="PORTAL.md">📚 Documentation Portal</a> | <a href="../README.md">← Root README</a>
</p>

---

This reference guide provides a complete overview of the input model, mouse and keyboard interactions, popup surface navigation, and the dual-surface `Edit Layout` workflow in **Toodle** and its companion application (`toodle-settings`).

---

## 🧭 Controls Quick Reference

| Target | Input / Action | Resulting Behavior |
| :--- | :--- | :--- |
| **Main Widget** | **Right-Click** (inside clock/weather bounds) | Open **Context Menu** (`Calendar`, `Weekly Forecast`, `Edit Layout`, `Settings`, `Quit`) |
| **Main Widget** | **Left-Click** (outside clock/weather bounds) | **Click-Through** directly to background/desktop (Minimal input region design) |
| **Popups** | **Click Outside** or press **`Esc`** | Dismiss currently open popup (Menu, Calendar, Weekly Forecast) |
| **Calendar** | **Click `<` or `>` buttons** | Smoothly navigate to previous / next month |
| **Calendar** | **Click `Today` button** | Instantly return to current month with today's cell clearly highlighted |
| **Edit Layout Panel** | **9 Grid Zone Buttons** (TopLeft to BottomRight) | Reposition the widget on your desktop in real-time |
| **Edit Layout Panel** | **Stage 1–10 Size Buttons / Slider** | Dynamically morph widget dimensions and typography in place |
| **Edit Layout Panel** | **Save Button** | Persist selected position and size atomically to `config.toml` and exit edit mode |
| **Edit Layout Panel** | **Cancel Button** | Revert to initial layout prior to editing and dismiss edit panel |
| **Settings App** | **Adjust any setting** | Instantly auto-saved to `config.toml` and applied to live widget within ~25–50ms |

---

## 1. Widget Input Architecture & Mouse Handling

The main Toodle clock widget resides on `Layer::Bottom`, immediately above desktop wallpapers.

- **Localized Input Region (Click-Through Transparency)**:
  - Toodle never creates an invisible full-screen input blocker. Instead, it dynamically declares only the tight bounding box around the active clock, date, and weather elements (`content_bounds`) to the Wayland compositor.
  - Clicking on any empty desktop area around the widget passes through directly to the underlying desktop shell, allowing desktop icons and compositor menus to function without interference.
- **Right-Click Activation**:
  - Right-clicking within the widget's active content rectangle opens the dedicated context menu (340 × 380 px) on `Layer::Top`.

---

## 2. Popup Surfaces (`Layer::Top`)

Popups opened from the context menu are styled with a solid dark background, ensuring high contrast and legibility across all desktop wallpapers:

### ① Monthly Calendar
- **Dimensions**: 680 × 720 px
- **Interaction**:
  - Click `<` and `>` at the top to traverse months.
  - Click `Today` to return immediately to the current date, which is highlighted with high-contrast accent styling.
  - Dismiss by clicking outside or right-clicking.

### ② 7-Day Weather Forecast
- **Dimensions**: 680 × 720 px
- **Information Rendered**:
  - Date and day of week for the upcoming 7 days.
  - Vivid color vector weather icons (Bas Milius Meteocons).
  - High and low temperature bars with visual range indicator.
  - Precipitation probability percentages.
- **Interaction**:
  - Information-only view. Click outside or press `Esc` to close.

---

## 3. Dual-Surface Edit Layout Mode

Selecting **`Edit Layout`** from the context menu initiates Toodle's unique dual-surface configuration mode:

```text
┌────────────────────────────────────────────────────────┐
│  Desktop (Wayland)                                     │
│                                                        │
│  ┌───────────────────────┐                             │
│  │   Edit Layout Panel   │ (Layer::Top Floating Panel) │
│  │   [ 3x3 Grid Buttons] │                             │
│  │   [ 1-10 Size Slider] │                             │
│  │   [ Save ] [ Cancel ] │                             │
│  └───────────┬───────────┘                             │
│              │ In-place Layer Commands (Zero Flicker)  │
│              ▼                                         │
│  ┌───────────────────────┐                             │
│  │  Toodle Widget (Live) │ (Layer::Bottom Live Widget) │
│  └───────────────────────┘                             │
└────────────────────────────────────────────────────────┘
```

1. **Zero-Flicker In-Place Transformation**:
   - The floating `Edit Layout Panel` appears on `Layer::Top`.
   - Clicking any of the 9 grid positions (TopLeft, TopCenter, TopRight, MiddleLeft, Center, MiddleRight, BottomLeft, BottomCenter, BottomRight) moves the widget immediately.
   - Adjusting size stages (Stage 1: 280px to Stage 10: 2060px) rescales fonts, spacing, and icons without destroying or recreating the Wayland surface.
2. **Commit and Rollback**:
   - **Save**: Atomically writes changes to `~/.config/toodle/config.toml` and dismisses the panel.
   - **Cancel**: Restores original layout settings and exits edit mode cleanly.

---

## 4. Companion Settings Application (`toodle-settings`)

`toodle-settings` provides a full XDG Toplevel window (720 × 780 px) for comprehensive customization:

- **Appearance Tab**:
  - 10 curated themes (Modern, Classic, Digital Mono, Cyberpunk, Nord, Warm Sunset, Forest, Slate, Rose Gold, Minimal).
  - 16-color palette picker and text drop-shadow toggle.
- **Layout Tab**:
  - 9-zone position grid and 10-stage size slider.
- **Weather Tab**:
  - Global city preset selector (Tokyo, New York, London, Paris, Sydney, etc.).
  - Manual latitude/longitude coordinate entry.
  - Temperature unit toggle (Celsius ℃ / Fahrenheit ℉).
- **Display Tab**:
  - Multi-monitor target selector (e.g. `"DP-1"`, `"HDMI-A-1"`).

> 💡 **Real-Time Hot Reload**:
> Changes made in `toodle-settings` are saved and pushed to the running widget via inotify within ~25–50ms, eliminating the need to restart the daemon.

---

<p align="center">
  <a href="PORTAL.md">📚 Documentation Portal</a> | <a href="../README.md">← Root README</a> | <a href="SHORTCUTS.ja.md">日本語ガイドはこちら →</a>
</p>
