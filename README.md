# Toodle

> **Modern Digital Clock Widget for the System76 COSMIC Desktop Environment**

![Banner](images/toodle-banner.svg)

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-2021%20Edition-orange.svg)](https://www.rust-lang.org/)
[![COSMIC](https://img.shields.io/badge/Desktop-COSMIC-purple.svg)](https://github.com/pop-os/cosmic-epoch)
[![Wayland](https://img.shields.io/badge/Protocol-wlr--layer--shell-green.svg)](https://wayland.freedesktop.org/)

[日本語ドキュメント (Japanese)](README.ja.md) | [Licenses](LICENSES.md) | [Architecture](docs/ARCHITECTURE.md) | [Features](docs/FEATURES.md) | [Design Doc v0.3](docs/Drafts/Toodle-Design-Docs-v0.3.md) | [Handover Guide](SESSION_HANDOVER.md)

---

## Highlights

- **Wayland Native**: Built specifically for COSMIC Desktop with `libcosmic` on `wlr-layer-shell`.
- **Dynamic Multi-Monitor Management (`OutputManager`)**: Automatically discovers connected Wayland outputs from `cosmic-randr`, seamlessly binding to a configured display (e.g. `"DP-1"`) or safely falling back to the primary/first available output. Isolates screen dimensions per display and handles hot-plug reconnection dynamically.
- **Accurate Sub-Second Clock**: High-precision async tick stream synchronized to the exact microsecond boundary of each upcoming second.
- **Robust 2-Tier Cached Weather**: Live weather and 7-day forecast powered by Open-Meteo REST API, with 30-minute / 3-hour local persistent caching, strict JSON response validation, coordinate bounds checking, clock-skew protection, and offline fallback.
- **Independent Popup Surfaces (`Layer::Top`)**:
  - Right-click Context Menu (340 x 380) with centered 18px actions.
  - Full Monthly Calendar (680 x 720) with smooth month navigation and today highlight.
  - 7-Day Weekly Forecast (680 x 720) with condition icons, high/low temperatures, and precipitation probabilities.
  - Solid dark styling for consistent visibility across any desktop wallpaper.
- **9-Zone Display Grid & 10 Discrete Size Stages (WQHD-Ready)**: Intuitive 3×3 screen placement (`TopLeft` through `BottomRight`) paired with 10 size stages (280px to 2060px) tuned for screens up to WQHD (2560×1440). Font sizes (time, date) and vector weather icons synchronize 1:1 with window dimensions, with ample vertical headroom preventing any bottom clipping of text or weather icons.
- **Two-Surface Edit Layout Mode**: An independent `Layer::Top` Edit Layout Panel (featuring 9-zone position buttons and 10 size selector buttons) works alongside the running `Layer::Bottom` widget, allowing instant in-place layout tuning without surface recreation or redraw flicker.
- **Dedicated Settings App (`toodle-settings`)**: Native XDG Toplevel application (720 x 780, solid dark background) with 4 tabs (Appearance, Layout, Weather, Display), real-time atomic auto-save with error notifications, 10 theme presets, 16 curated colors, 9-zone layout and 10-stage size selectors, and quick city presets.
- **Decoupled Build & Local Install Structure**: `cargo build --release` strictly compiles the Rust binaries without touching the user's home directory. Local installation to `$HOME/.local/bin` is handled via explicit execution of `./tools/install-local.sh`, establishing a clean architecture for future distribution packaging (.deb, RPM, AUR).
- **Live Inotify Hot-Reloading & Migration**: Automatically updates running widgets in real-time when `~/.config/toodle/config.toml` changes (50ms debounced response). Older configuration files are automatically migrated and rewritten to disk in modern clean format. Active in-progress edits are protected from unexpected config reloads.
- **100% Embedded Fonts & Color Vector Weather Icons**: Includes Roboto, JetBrains Mono, DejaVu Serif, and Open Sans statically embedded into the binary via `include_bytes!`. Weather condition icons are rendered using embedded Bas Milius Meteocons color SVG assets (`resources/icons/meteocons/`, MIT License) rather than monochrome font glyphs or Unicode emojis, ensuring vibrant, crisp, modern visual representation across all display scales without external dependencies.

---

## Quick Start

### Prerequisites
- Arch Linux (or any modern Linux distribution with Rust & Cargo installed).
- COSMIC Desktop Environment (`cosmic-comp`, `libcosmic` dependencies: `wayland`, `libxkbcommon`).

### Building

Toodle is distributed as source code; pre-compiled binaries are not distributed via GitHub releases. You can build the binaries locally from source:

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

#### Autostart & Troubleshooting (COSMIC Desktop)

When registering Toodle to launch automatically at login (e.g. via COSMIC Settings > Autostart or `$HOME/.config/autostart/*.desktop`):

Depending on the initialization timing between the COSMIC login session and Toodle's Layer Surface / Input Region, right-clicks may not respond immediately after autostart. While the exact root cause remains unconfirmed, potential timing competition with desktop background initialization has been hypothesized.

The verified workaround on tested hardware is to configure a ~3-second delay in the XDG Autostart command (introducing a 3-second delay has been confirmed to resolve the issue on tested machines; session startup ordering via a systemd user service is a future alternative / improvement candidate):

```desktop
# Verified workaround on tested hardware (e.g. Exec line in ~/.config/autostart/com.github.wammed.toodle.desktop)
Exec=sh -c "sleep 3 && toodle"
# Or with explicit absolute path:
# Exec=sh -c "sleep 3 && $HOME/.local/bin/toodle"
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
| **Modern** | Roboto Bold / Regular | `#FFFFFF` | Contemporary clean aesthetic |
| **Classic** | DejaVu Serif Bold / Regular | `#E2E8F0` | Sophisticated timeless timepiece look |
| **Digital Mono** | JetBrains Mono Bold / Regular | `#38BDF8` | Sleek developer monospace terminal style |
| **Minimal** | Open Sans Bold / Regular | `#94A3B8` | Understated, clean humanist design |
| **Cyberpunk** | JetBrains Mono Bold / Regular | `#EAB308` | Futuristic neon high-contrast look |
| **Nord** | Open Sans Bold / Regular | `#38BDF8` | Cool arctic blue and slate tones |
| **Warm Sunset** | DejaVu Serif Bold / Regular | `#F59E0B` | Warm literary amber and evening glow |
| **Forest** | Roboto Bold / Regular | `#10B981` | Natural fresh emerald green theme |
| **Slate** | JetBrains Mono Bold / Regular | `#94A3B8` | Industrial muted dark metallic slate |
| **Rose Gold** | DejaVu Serif Bold / Regular | `#EC4899` | Elegant pastel rose and violet accents |

---

## Configuration

Settings are saved in `~/.config/toodle/config.toml`:

```toml
[display]
output = "" # Target Wayland output (e.g. "DP-1", empty defaults to primary or first available output)

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
> If an older `config.toml` containing legacy layout fields (`anchor`, `margin_x`, `margin_y`, `width`, `height`) or `font_scale` is loaded, Toodle automatically and deterministically migrates it to the closest `grid_position` and `size_stage`, preserving your preferred position and sizing before immediately rewriting the clean modern structure back to disk.

> **Display & Multi-Monitor Note**:
> Single-display geometry across standard resolutions (from 640×360 up to WQHD 2560×1440 and 4K 3840×2160) is verified through comprehensive automated geometry unit testing. The `OutputManager` state machine manages display binding, connector name matching, and primary display fallback. However, multi-monitor configurations—especially those with differing mixed resolutions or per-monitor fractional scaling—have not yet been sufficiently validated on physical hardware and remain an unverified item until dedicated test hardware is available.

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

This project is licensed under the [MIT License](LICENSE).
The embedded Meteocons color SVG icons by Bas Milius are licensed under the [MIT License](THIRD_PARTY_LICENSES/METEOCONS_LICENSE.txt).
All bundled fonts retain their respective upstream licenses (SIL Open Font License 1.1 / Bitstream Vera & DejaVu License).
For comprehensive licensing details, source distribution policies, and third-party notices, see **[LICENSES.md](LICENSES.md)** and [THIRD_PARTY_LICENSES/README.md](THIRD_PARTY_LICENSES/README.md).

