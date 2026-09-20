# Toodle

> **Modern Digital Clock Widget for the System76 COSMIC Desktop Environment**

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-2021%20Edition-orange.svg)](https://www.rust-lang.org/)
[![COSMIC](https://img.shields.io/badge/Desktop-COSMIC-purple.svg)](https://github.com/pop-os/cosmic-epoch)
[![Wayland](https://img.shields.io/badge/Protocol-wlr--layer--shell-green.svg)](https://wayland.freedesktop.org/)

[日本語ドキュメント (Japanese)](README.ja.md) | [Architecture](docs/ARCHITECTURE.md) | [Features](docs/FEATURES.md) | [Handover Guide](SESSION_HANDOVER.md)

---

## Highlights

- **Wayland Native**: Built specifically for COSMIC Desktop with `libcosmic` on `wlr-layer-shell`.
- **Zero-Flicker Surface Management**: Always pinned to `Layer::Bottom` behind regular windows, with non-rectangular input zones that let background clicks pass through seamlessly to the desktop wallpaper and icons.
- **Accurate Sub-Second Clock**: High-precision async tick stream synchronized to the exact microsecond boundary of each upcoming second.
- **2-Tier Cached Weather**: Live weather and 7-day forecast powered by Open-Meteo REST API, with 30-minute / 3-hour local persistent caching and offline fallback.
- **Interactive Popup Surfaces (`Layer::Top`)**:
  - Right-click Context Menu (340 x 380) with centered 18px actions.
  - Full Monthly Calendar (680 x 720) with smooth month navigation and today highlight.
  - 7-Day Weekly Forecast (680 x 720) with condition icons, high/low temperatures, and precipitation probabilities.
- **Interactive Edit Layout Mode**: Right-click to enter layout edit mode directly on the desktop to adjust margins, dimensions, and font scale with live visual boundaries.
- **Dedicated Settings App (`toodle-settings`)**: Native XDG Toplevel application (720 x 780) with 4 tabs (Appearance, Layout, Weather, Display), real-time slider updates, 10 theme presets, 16 curated colors, and city presets.
- **Live Inotify Hot-Reloading**: Automatically updates running widgets in real-time when `~/.config/toodle/config.toml` changes (25ms response).
- **100% Embedded Fonts**: Includes Roboto Sans, JetBrains Mono, DejaVu Serif, and Open Sans statically embedded into the binary via `include_bytes!`. No external font dependencies.

---

## Quick Start

### Prerequisites
- Arch Linux (or any modern Linux distribution with Rust & Cargo installed).
- COSMIC Desktop Environment (`cosmic-comp`, `libcosmic` dependencies: `wayland`, `libxkbcommon`).

### Building

```bash
# Clone repository
git clone https://github.com/wammed/Toodle.git
cd Toodle

# Build both toodle widget and toodle-settings
cargo build --release
```

### Running

```bash
# Launch the desktop clock widget (runs on desktop background Layer::Bottom)
./target/release/toodle &

# Launch the settings application
./target/release/toodle-settings
```

---

## Architecture at a Glance

```text
┌─────────────────────────────────────────────────────────────┐
│                       COSMIC Desktop                        │
│                                                             │
│   Layer::Top Popups:                                        │
│   ┌───────────────┐ ┌───────────────┐ ┌──────────────────┐  │
│   │ Context Menu  │ │   Calendar    │ │ Weekly Forecast  │  │
│   │   (340x380)   │ │   (680x720)   │ │    (680x720)     │  │
│   └───────▲───────┘ └───────▲───────┘ └────────▲─────────┘  │
│           │                 │                  │            │
│   Layer::Bottom Widget:     │                  │            │
│   ┌─────────────────────────┴──────────────────┴─────────┐  │
│   │ Toodle Main Widget                                   │  │
│   │ [12:34:56]  [Monday, Sep 20, 2026]  [Sunny 22°C]     │  │
│   │ (Non-rectangular input zone: passthrough background) │  │
│   └─────────────────────────▲────────────────────────────┘  │
│                             │ inotify watch (~25ms)         │
│               ~/.config/toodle/config.toml                  │
│                             ▲                               │
│   XDG Toplevel Window:      │ atomic write (.tmp -> rename) │
│   ┌─────────────────────────┴────────────────────────────┐  │
│   │ toodle-settings (Appearance, Layout, Weather, Disp)  │  │
│   └──────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

---

## Theme Presets & Fonts

| Theme | Font Family | Default Color | Style Description |
|---|---|---|---|
| **Modern** | Roboto Sans Bold / Regular | `#FFFFFF` | Contemporary clean aesthetic |
| **Classic** | DejaVu Serif Bold / Regular | `#E2E8F0` | Sophisticated timeless timepiece look |
| **Digital Mono** | JetBrains Mono Bold / Regular | `#38BDF8` | Sleek developer monospace terminal style |
| **Minimal** | Open Sans Bold / Regular | `#94A3B8` | Understated, clean humanist design |
| **Cyberpunk** | JetBrains Mono Bold / Regular | `#EAB308` | Futuristic neon high-contrast look |
| **Nord** | Open Sans Bold / Regular | `#38BDF8` | Cool arctic blue and slate tones |
| **Warm Sunset** | DejaVu Serif Bold / Regular | `#F59E0B` | Warm literary amber and evening glow |
| **Forest** | Roboto Sans Bold / Regular | `#10B981` | Natural fresh emerald green theme |
| **Slate** | JetBrains Mono Bold / Regular | `#94A3B8` | Industrial muted dark metallic slate |
| **Rose Gold** | DejaVu Serif Bold / Regular | `#EC4899` | Elegant pastel rose and violet accents |

---

## Configuration

Settings are saved in `~/.config/toodle/config.toml`:

```toml
[display]
output = "" # Empty for primary/active output, or specify e.g. "DP-1"

[layout]
anchor = "TopRight" # TopLeft, TopRight, BottomLeft, BottomRight
margin_x = 40
margin_y = 60
width = 280
height = 140

[appearance]
theme = "Modern"
color = "#FFFFFF"
font_scale = 1.0
text_shadow = true

[weather]
location_name = "Tokyo, Japan"
latitude = 35.6895
longitude = 139.6917
temperature_unit = "Celsius" # Celsius, Fahrenheit
```

---

## Key Interactions

- **Left Click**: Focus or trigger interactive elements.
- **Right Click**: Opens the Context Menu popup (`Calendar`, `Weekly Forecast`, `Edit Layout`, `Settings`, `Quit`).
- **Edit Layout Mode**: Drag margins/dimensions with the on-screen slider, then click `Save` or `Cancel`.
- **Settings Application**: Changes to sliders, themes, colors, and city presets are applied in real-time to the active widget.

---

## License

This project is licensed under the [MIT License](LICENSE).
