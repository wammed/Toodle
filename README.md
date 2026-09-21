# Toodle

> **Modern Digital Clock Widget for the System76 COSMIC Desktop Environment**

[![License: GPL-3.0-or-later](https://img.shields.io/badge/License-GPL--3.0--or--later-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-2021%20Edition-orange.svg)](https://www.rust-lang.org/)
[![COSMIC](https://img.shields.io/badge/Desktop-COSMIC-purple.svg)](https://github.com/pop-os/cosmic-epoch)
[![Wayland](https://img.shields.io/badge/Protocol-wlr--layer--shell-green.svg)](https://wayland.freedesktop.org/)

[日本語ドキュメント (Japanese)](README.ja.md) | [Architecture](docs/ARCHITECTURE.md) | [Features](docs/FEATURES.md) | [Design Doc v0.3](docs/Drafts/Toodle-Design-Docs-v0.3.md) | [Handover Guide](SESSION_HANDOVER.md)

---

## Highlights

- **Wayland Native**: Built specifically for COSMIC Desktop with `libcosmic` on `wlr-layer-shell`.
- **Zero-Flicker Surface Management**: Pinned to `Layer::Bottom` behind regular windows, with a rectangular `content_bounds` input zone that lets background clicks pass through seamlessly to desktop wallpaper and icons.
- **Accurate Sub-Second Clock**: High-precision async tick stream synchronized to the exact microsecond boundary of each upcoming second.
- **2-Tier Cached Weather**: Live weather and 7-day forecast powered by Open-Meteo REST API, with 30-minute / 3-hour local persistent caching, coordinate verification, and offline fallback.
- **Independent Popup Surfaces (`Layer::Top`)**:
  - Right-click Context Menu (340 x 380) with centered 18px actions.
  - Full Monthly Calendar (680 x 720) with smooth month navigation and today highlight.
  - 7-Day Weekly Forecast (680 x 720) with condition icons, high/low temperatures, and precipitation probabilities.
  - Solid dark styling for consistent visibility across any desktop wallpaper.
- **9-Zone Display Grid & 10 Discrete Size Stages (WQHD-Ready)**: Intuitive 3×3 screen placement (`TopLeft` through `BottomRight`) paired with 10 size stages (280px to 2060px) tuned for screens up to WQHD (2560×1440). Font sizes (time, date, weather) synchronize 1:1 with window dimensions, with ample vertical headroom preventing any bottom clipping of text or weather glyphs.
- **Two-Surface Edit Layout Mode**: An independent `Layer::Top` Edit Layout Panel (featuring 9-zone position buttons and 10 size selector buttons) works alongside the running `Layer::Bottom` widget, allowing instant in-place layout tuning without surface recreation or redraw flicker.
- **Dedicated Settings App (`toodle-settings`)**: Native XDG Toplevel application (720 x 780, solid dark background) with 4 tabs (Appearance, Layout, Weather, Display), real-time atomic auto-save, 10 theme presets, 16 curated colors, 9-zone layout and 10-stage size selectors, and quick city presets.
- **Decoupled Build & Local Install Structure**: `cargo build --release` strictly compiles the Rust binaries without touching the user's home directory. Local installation to `$HOME/.local/bin` is handled via explicit execution of `./tools/install-local.sh`, establishing a clean architecture for future distribution packaging (.deb, RPM, AUR).
- **Live Inotify Hot-Reloading**: Automatically updates running widgets in real-time when `~/.config/toodle/config.toml` changes (25ms response).
- **100% Embedded Fonts**: Includes Roboto Sans, JetBrains Mono, DejaVu Serif, Open Sans, and Weather Icons statically embedded into the binary via `include_bytes!`. Weather condition icons are rendered using the embedded Weather Icons font (`resources/fonts/WeatherIcons.ttf`, SIL Open Font License 1.1) rather than Unicode emojis, ensuring identical rendering across all environments without external font dependencies.

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

# Build both toodle widget and toodle-settings (builds only, does not install)
cargo build --release
```

### Local Installation

To install the release binaries (`toodle` and `toodle-settings`) to `$HOME/.local/bin`:

```bash
./tools/install-local.sh
```

### Running

```bash
# Launch the desktop clock widget (runs on desktop background Layer::Bottom)
./target/release/toodle &
# or from $HOME/.local/bin if installed:
# toodle &

# Launch the settings application
./target/release/toodle-settings
# or from $HOME/.local/bin if installed:
# toodle-settings
```

---

## Architecture at a Glance

```text
┌─────────────────────────────────────────────────────────────┐
│                       COSMIC Desktop                        │
│                                                             │
│   Layer::Top Independent Surfaces:                          │
│   ┌───────────────┐ ┌───────────────┐ ┌──────────────────┐  │
│   │ Context Menu  │ │   Calendar    │ │ Weekly Forecast  │  │
│   │   (340x380)   │ │   (680x720)   │ │    (680x720)     │  │
│   └───────▲───────┘ └───────▲───────┘ └────────▲─────────┘  │
│           │                 │                  │            │
│           └─────────────────┼──────────────────┘            │
│                             │ Edit Layout Panel (Layer::Top)│
│                             ▼                               │
│   Layer::Bottom Widget:                                     │
│   ┌──────────────────────────────────────────────────────┐  │
│   │ Toodle Main Widget                                   │  │
│   │ [12:34:56]  [Monday, Sep 20, 2026]  [Sunny 22°C]     │  │
│   │ (content_bounds input zone: transparent passthrough) │  │
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
output = "" # Target Wayland output (e.g. "DP-1", empty defaults to active output)

[layout]
grid_position = "TopRight" # 9 zones: TopLeft, TopCenter, TopRight, MiddleLeft, Center, MiddleRight, BottomLeft, BottomCenter, BottomRight
size_stage = 2            # 10 stages: 1 (Compact 280x130) to 10 (Max WQHD 2060x920)

[appearance]
theme = "Modern"
color = "#FFFFFF"
text_shadow = true

[weather]
location_name = "Tokyo, Japan"
latitude = 35.6895
longitude = 139.6917
temperature_unit = "Celsius" # Celsius, Fahrenheit
```

> **Automatic Config Migration**:
> If an older `config.toml` containing legacy layout fields (`anchor`, `margin_x`, `margin_y`, `width`, `height`) or `font_scale` is loaded, Toodle automatically and deterministically migrates it to the closest `grid_position` and `size_stage`, preserving your preferred position and sizing before saving the clean modern structure.

> **Display & Multi-Monitor Note**:
> Single-display geometry across standard resolutions (from 640×360 up to WQHD 2560×1440 and 4K 3840×2160) is verified through comprehensive automated geometry unit testing. However, multi-monitor configurations—especially those with differing mixed resolutions or per-monitor fractional scaling—have not yet been sufficiently validated on physical hardware and remain an unverified item until dedicated test hardware is available.

---

## Key Interactions

- **Left Click**: Focus or trigger interactive elements (e.g. month navigation in Calendar).
- **Right Click**: Opens the Context Menu popup (`Calendar`, `Weekly Forecast`, `Edit Layout`, `Settings`, `Quit`).
- **Edit Layout Mode**: Adjust position using the 3×3 zone grid and select one of 10 discrete size stages via the dedicated `Edit Layout Panel` with instantaneous in-place Wayland layer surface updates, then click `Save` or `Cancel`.
- **Settings Application**: Changes to themes, 16-color swatches, and city presets are applied in real-time to the active widget.

---

## Development

Toodle is developed using AI-assisted Vibe Coding. AI is actively used for architecture exploration, implementation, refactoring, testing, documentation, and code review, with the resulting code and behavior reviewed and validated throughout development.

---

## Documentation & Design Baseline

- **[Toodle Design Document v0.3](docs/Drafts/Toodle-Design-Docs-v0.3.md)**: Current official design baseline reflecting real-world COSMIC verification results.
- **[Architecture Document](docs/ARCHITECTURE.md)**: Detailed surface hierarchy, tick stream, and cache specifications.
- **[Feature Specification](docs/FEATURES.md)**: Detailed UI/UX feature guide.
- **[Session Handover Guide](SESSION_HANDOVER.md)**: Full project history, technical decisions, known gaps, and roadmap.

---

## License

This project is licensed under the [GPL-3.0-or-later](LICENSE) license.
The embedded Weather Icons font by Erik Flowers is licensed under the [SIL Open Font License 1.1](THIRD_PARTY_LICENSES/WEATHER_ICONS_LICENSE.txt). See [THIRD_PARTY_LICENSES/README.md](THIRD_PARTY_LICENSES/README.md) for details.
