<div align="center">

# ⏱️ Toodle
### Modern Digital Clock & Weather Widget for System76 COSMIC Desktop Environment

![Banner](images/toodle-banner.svg)

[![Built with libcosmic](https://img.shields.io/badge/libcosmic-Pop!_OS_COSMIC-24C8D8?style=for-the-badge&logo=linux&logoColor=white)](https://github.com/pop-os/libcosmic)
[![Rust](https://img.shields.io/badge/Rust-1.80+-orange?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Wayland](https://img.shields.io/badge/Protocol-wlr--layer--shell-5277C3?style=for-the-badge&logo=wayland&logoColor=white)](https://wayland.freedesktop.org/)
[![Platform](https://img.shields.io/badge/Platform-Linux_(COSMIC_/_Wayland)-FCC624?style=for-the-badge&logo=linux&logoColor=black)](https://www.kernel.org/)
[![Vibe Coding](https://img.shields.io/badge/Built_with-AI_Vibe_Coding-8A2BE2?style=for-the-badge&logo=sparkles&logoColor=white)](#-about-this-project-ai-vibe-coding)
[![License: MIT](https://img.shields.io/badge/License-MIT-green?style=for-the-badge)](LICENSE)

<p align="center">
  <strong>Native COSMIC / Wayland × Subsecond Precision Clock × 2-Tier Weather Cache × 9-Zone & 10-Stage Smooth Layout</strong><br>
  A lightweight, feature-rich desktop clock widget residing directly above wallpapers with zero-flicker live in-place deformation for Pop!_OS COSMIC Desktop and Linux Wayland compositors.
</p>

<p align="center">
  <a href="README.md">English</a> | <a href="README.ja.md">日本語</a> | <a href="docs/PORTAL.md">📚 Documentation Portal</a> | <a href="LICENSES/README.md">📄 Licensing Notice</a>
</p>

</div>

---

## 🚀 Quick Start

### 1. Prerequisites

- [Rust (Cargo)](https://rustup.rs/) (1.80 or newer)
- Linux Wayland compositor / Pop!_OS COSMIC Desktop (`cosmic-comp`)
- System build dependencies (Debian / Pop!_OS / Ubuntu):
  ```bash
  sudo apt install build-essential libxkbcommon-dev wayland-protocols
  ```
  *(Arch Linux: `sudo pacman -S base-devel libxkbcommon wayland`)*

### 2. Build

Toodle is distributed in source code form. Build locally using standard Cargo:

```bash
# Clone repository
git clone https://github.com/wammed/Toodle.git
cd Toodle

# Build widget daemon and companion settings application
cargo build --release
```

### 3. Local Installation (Optional)

To copy the release binaries (`toodle`, `toodle-settings`) to `$HOME/.local/bin`:

```bash
./tools/install-local.sh
```

### 4. Running

```bash
# Start the clock widget (runs on Layer::Bottom above desktop wallpaper)
./target/release/toodle &
# Or if installed locally:
# toodle &

# Launch companion configuration application
./target/release/toodle-settings
# Or if installed locally:
# toodle-settings
```

#### 💡 Recommended Autostart Configuration
When configuring Toodle in COSMIC session autostart (`~/.config/autostart/*.desktop`), add a 3-second startup delay to prevent race conditions during compositor login initialization (verified hardware workaround):

```desktop
Exec=sh -c "sleep 3 && toodle"
```

---

## 💡 Key Features

- 🪟 **Native COSMIC / Wayland (`wlr-layer-shell`)**: Built with System76's official `libcosmic` toolkit. Operates on `Layer::Bottom` with minimal localized `content_bounds` input regions, allowing mouse clicks outside text bounds to pass straight through to desktop wallpapers.
- ⏱️ **Subsecond Microsecond-Aligned Ticker**: Asynchronous clock stream synchronized to the upcoming microsecond second boundary, delivering pinpoint accuracy with zero CPU overhead.
- 🌤️ **2-Tier Resilient Weather Subsystem**: Powered by Open-Meteo REST API with 30-min current and 3-hour weekly persistent local caches. Includes request generation IDs to discard stale responses, clock skew defense, and graceful offline fallback.
- 🖥️ **Dynamic Multi-Monitor Management (`OutputManager`)**: Automatically tracks Wayland output metadata, seamlessly binding to designated displays (e.g. `"DP-1"`), with geometry settle delays and automatic fallback to primary displays.
- 📐 **9-Zone Grid & 10 Size Stages**: 3×3 desktop positioning (TopLeft through BottomRight) and 10 size stages (280px to 2060px) optimized up to WQHD, dynamically scaling fonts, spacing, and vector weather icons.
- ⚡ **Dual-Surface Edit Layout Mode**: Floating `Layer::Top` control panel coordinates with live `Layer::Bottom` widget. In-place layer commands enable zero-flicker real-time reshaping without surface recreation.
- ⚙️ **Companion Settings App (`toodle-settings`)**: Dedicated 4-tab XDG Toplevel window (Appearance, Layout, Weather, Display) offering 10 theme presets, 16-color palette, quick city selector, and real-time auto-saving.
- 🔄 **Fast inotify Hot-Reload & Migration**: Automatically detects changes to `~/.config/toodle/config.toml` within 50ms. Legacy configurations are automatically migrated to modern parameters on startup.
- 🎨 **Embedded Fonts & Color Vector Weather Icons**: Statically bundles Roboto, Open Sans, JetBrains Mono NL, and DejaVu Serif. Renders crisp Bas Milius Meteocons color SVG icons at any scale without external system dependencies.

> 📖 **Detailed Technical Specifications**:
> For detailed specifications visit [docs/FEATURES.md](docs/FEATURES.md); for internal architecture and multi-surface designs see [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md).

---

## 🖱️ Controls & Interaction

Summary of primary controls:

| Component | Input / Action | Resulting Action |
| :--- | :--- | :--- |
| **Main Widget** | **Right-Click** | Show **Context Menu** (`Calendar`, `Weekly Forecast`, `Edit Layout`, `Settings`, `Quit`) |
| **Main Widget** | **Left-Click** (outside bounds) | **Click-Through** directly to desktop wallpaper (Minimal input region) |
| **Popups** | **Click Outside** / **`Esc`** | Dismiss active popup (Menu, Calendar, Forecast) |
| **Calendar** | **`<` and `>` buttons** | Navigate previous / next months (click `Today` to return) |
| **Edit Layout Panel** | **9 Grid Zone Buttons** | Instantly reposition widget across desktop zones |
| **Edit Layout Panel** | **Stage 1–10 Slider / Buttons** | Morph widget size and font scaling in place |
| **Edit Layout Panel** | **Save** / **Cancel** | Atomically commit layout / Revert to original settings |
| **Settings App** | **Adjust any setting** | Auto-saves immediately to `config.toml` with live widget hot-reload |

> 🖱️ **Full Interaction Guide**:
> See **[docs/SHORTCUTS.md](docs/SHORTCUTS.md)** for exhaustive details on mouse interactions and surface navigation.

---

## 📚 Documentation Portal

Comprehensive documentation is organized across specialized guides:

| Document | Primary Contents |
| :--- | :--- |
| **[📚 Documentation Portal](docs/PORTAL.md)** | Role-based navigation hub and task guide |
| **[🖱️ Interaction & Controls Guide](docs/SHORTCUTS.md)** | Full mouse/keyboard controls, calendar, Edit Layout mode, settings application |
| **[💡 Feature Specification (FEATURES)](docs/FEATURES.md)** | Stage dimension charts, 10 theme palettes, weather caching, SVG icons |
| **[📐 Architecture Specification (ARCHITECTURE)](docs/ARCHITECTURE.md)** | Multi-surface hierarchy, dynamic monitor management, ticker clock stream |
| **[🛡️ Reliability & Security Model (SECURITY)](docs/SECURITY.md)** | 2-tier cache integrity, stale protection, PID atomic writes, layer isolation |
| **[📄 Licensing & Third-Party Notice](LICENSES/README.md)** | Core MIT License, Meteocons, embedded fonts, cargo-deny dependency compliance |

---

## 🔒 Reliability & Security (Overview)

- **Zero Telemetry & Full Offline Capability**: No user tracking, remote analytics, or background telemetry. Widget functions seamlessly during network outages via persistent local caches.
- **Request Generation Counter (Stale Protection)**: Discards out-of-order delayed asynchronous weather updates, guaranteeing the UI never displays outdated conditions.
- **Atomic Persistence with PID Temporary Files**: File writes to `config.toml` and cache utilize `.{file}.tmp.{pid}`, `sync_all`, and atomic filesystem renames to prevent partial write corruption.
- **Minimized Input Regions**: Limits mouse capture strictly to visible content bounding boxes (`content_bounds`), preventing click-jacking and desktop shell interference.

> 🛡️ **Detailed Security Specification**:
> For complete security models, consult **[docs/SECURITY.md](docs/SECURITY.md)**.

---

## 🗺️ Roadmap

- [x] ⏱️ **Subsecond Precision Clock**: Microsecond boundary-aligned non-blocking ticker stream.
- [x] 🌤️ **2-Tier Resilient Weather Subsystem**: Open-Meteo REST API, request generation IDs, offline fallback.
- [x] 📐 **9-Zone Grid & 10 Size Stages**: WQHD-optimized layout presets and in-place real-time reshaping.
- [x] 🎛️ **Companion Settings App (`toodle-settings`)**: 4-tab configuration, real-time auto-save, 10 themes, 16-color palette.
- [x] 📁 **Centralized Licensing (`LICENSES/`)**: Consolidated asset licenses and cargo-deny audit records.
- [ ] 🖥️ **Mixed DPI / Fractional Scaling Verification**: Comprehensive hardware testing across heterogeneous multi-monitor setups.
- [ ] 📅 **Calendar Event Integration**: Local iCalendar (.ics) or system calendar synchronization.
- [ ] ⏰ **Alarms & Timers**: Lightweight desktop timers and reminders integrated with notifications.

---

## 🤖 About This Project (AI Vibe Coding)

> [!IMPORTANT]
> ### 💡 Built with AI Vibe Coding
> **Toodle** was engineered through interactive pair-programming (**AI Vibe Coding**) with **Google DeepMind's Antigravity (Gemini)**.
> Blending human architecture direction and rapid AI implementation, debugging, and verification, Toodle was built from scratch—spanning low-level Rust `libcosmic` Wayland layer shell protocols, multi-surface management, microsecond-aligned ticker streams, 2-tier weather caching, zero-flicker in-place layout transformation, and a native XDG Toplevel settings application.

---

## 📄 License

Toodle's source code is released under the [MIT License](LICENSE).

All authoritative third-party asset licenses and dependency compliance records are centralized in the **[`LICENSES/`](LICENSES/)** directory:

- **Comprehensive Licensing Notice**: **[LICENSES/README.md](LICENSES/README.md)** ([Japanese: LICENSES/README.ja.md](LICENSES/README.ja.md))
- **Embedded Meteocons Color SVG Icons**: [MIT License (Bas Milius)](LICENSES/METEOCONS_LICENSE.txt)
- **Embedded Fonts**: [Roboto (SIL OFL 1.1)](LICENSES/ROBOTO_OFL.txt), [Open Sans (SIL OFL 1.1)](LICENSES/OPEN_SANS_OFL.txt), [JetBrains Mono NL (SIL OFL 1.1)](LICENSES/JETBRAINS_MONO_NL_OFL.txt), [DejaVu Serif (Bitstream Vera / DejaVu)](LICENSES/DEJAVU_LICENSE.txt)
- **Third-Party Dependency Audit Records**: [LICENSES/THIRD_PARTY_AUDIT.md](LICENSES/THIRD_PARTY_AUDIT.md)

<p align="center">
  Crafted via <strong>AI Vibe Coding</strong> 🚀 · Built with ❤️ for Pop!_OS COSMIC & Linux Developers
</p>
