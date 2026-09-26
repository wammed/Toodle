# Licensing & Third-Party Notice

<p align="center">
  <strong>English</strong> | <a href="README.ja.md">日本語</a> | <a href="../README.md">← Root README</a>
</p>

---

This document provides a comprehensive overview of the licensing policies, source code distribution model, third-party dependency governance, and asset clearance rules for the **Toodle** project. All license texts and audit records are aggregated under this directory (`LICENSES/`).

---

## 1. Project License (Toodle)

The core source code of **Toodle** is released under the **MIT License**.

- The full text of the license is available in the root [LICENSE](../LICENSE) file (also mirrored in [LICENSE-MIT.txt](LICENSE-MIT.txt)).
- Copyright (c) 2026 wammed.
- You are free to use, modify, distribute, sublicense, and create derivative works from Toodle's source code, provided that the original copyright notice and permission notice are preserved in substantial portions of the software.

---

## 2. Distribution & Build Model

### Source Distribution and Build Model

Toodle is distributed from this repository as source code.

- **No Pre-compiled Binaries on GitHub**: The project does not distribute pre-compiled binary executables, static library archives, or installer packages through GitHub. Users and downstream packagers are expected to build and package Toodle for their own environment.
- **Intentional Distribution Policy**: This approach is an intentional project operational policy. Distributing pre-built binaries would require the project to manage and present the applicable redistribution notices and license conditions for the bundled third-party libraries, embedded fonts, vector icons, and other assets. Keeping the GitHub distribution strictly source-based avoids making those redistribution and licensing notices unnecessarily complex while allowing downstream distributors and packagers to apply the packaging and licensing requirements appropriate to their own distribution environment.
- **Policy Clarification**: This is a project distribution policy, not a restriction imposed by the MIT License or by the upstream licenses of the third-party components used by Toodle.
- **Local Building and Installation**:
  - **Compilation**: Users compile the binaries locally using standard Cargo commands:
    ```bash
    cargo build --release
    ```
    This strictly builds `toodle` and `toodle-settings` into `target/release/` without modifying the user's system or home directory.
  - **Local Installation Tool**: A convenience script [`tools/install-local.sh`](../tools/install-local.sh) is provided for local users wishing to copy the compiled binaries to `$HOME/.local/bin`. This script is intended purely as a local developer/user helper, not as a GitHub release artifact distribution mechanism.
- **Direct Upstream Registry Fetching**: During compilation, all Rust dependencies are downloaded directly from the official package registry ([crates.io](https://crates.io/)) or designated official Git repositories (e.g. `libcosmic`). Toodle does not vendor or redistribute external crate sources in this repository.

---

## 3. Third-Party Dependencies & License Compliance

### Permissive and Weak-Copyleft Policy
To maintain license cleanliness and prevent unintended copyleft infection for downstream builders and packagers, all Rust dependencies in Toodle are governed by our license configuration in [`deny.toml`](../deny.toml):

- **Permitted Licenses**: Only permissive open-source licenses and weak-copyleft licenses with explicit boundaries (such as MPL-2.0 for COSMIC desktop libraries) are permitted:
  - `0BSD`
  - `Apache-2.0` / `Apache-2.0 WITH LLVM-exception`
  - `BSD-2-Clause` / `BSD-3-Clause`
  - `BSL-1.0` (Boost Software License)
  - `CC0-1.0`
  - `ISC`
  - `MIT`
  - `MPL-2.0` (Mozilla Public License 2.0)
  - `Unicode-3.0`
  - `Unlicense`
  - `Zlib`
- **Copyleft Exclusion**: Strong copyleft licenses (such as GPL-2.0, GPL-3.0, and AGPL) are excluded from the dependency graph.

### Automated License Auditing
Toodle leverages [`cargo-deny`](https://github.com/EmbarkStudios/cargo-deny) for automated license auditing:
- Run locally via:
  ```bash
  cargo deny check licenses
  ```
- Any dependency with missing or non-standard manifest metadata is investigated and documented in [`THIRD_PARTY_AUDIT.md`](THIRD_PARTY_AUDIT.md).

### System Libraries & Dynamic Linking
Toodle executes as a native COSMIC / Wayland client and relies on host system shared libraries:
- System libraries like `wayland-client` and `libxkbcommon` are dynamically linked (`.so`) from the host OS at runtime.
- These libraries are not bundled or statically linked into the repository source tree.

---

## 4. Fonts & Visual Assets Policy

Unlike conventional desktop applications relying solely on ambient host system fonts, Toodle statically embeds high-quality fonts and vector weather icons via Rust's `include_bytes!` to guarantee identical typography, metrics, and visual clarity across all Linux distributions.

All bundled third-party assets retain their respective upstream licenses:

### Embedded Fonts (`resources/fonts/`)

| Font Family | Files | Author / Copyright Holder | Upstream License | License Document |
| :--- | :--- | :--- | :--- | :--- |
| **Roboto** | `Roboto-Regular.ttf`<br>`Roboto-Bold.ttf` | Christian Robertson, Google LLC | SIL Open Font License 1.1 | [ROBOTO_OFL.txt](ROBOTO_OFL.txt) |
| **Open Sans** | `OpenSans-Regular.ttf`<br>`OpenSans-Bold.ttf` | Steve Matteson, Monotype Design Team | SIL Open Font License 1.1 | [OPEN_SANS_OFL.txt](OPEN_SANS_OFL.txt) |
| **JetBrains Mono NL** | `JetBrainsMono-Regular.ttf`<br>`JetBrainsMono-Bold.ttf` | JetBrains s.r.o., Philipp Nurullin, Konstantin Bulenkov | SIL Open Font License 1.1 | [JETBRAINS_MONO_NL_OFL.txt](JETBRAINS_MONO_NL_OFL.txt) |
| **DejaVu Serif** | `DejaVuSerif-Regular.ttf`<br>`DejaVuSerif-Bold.ttf` | DejaVu fonts team, Bitstream Inc., Tavmjong Bah | Bitstream Vera / DejaVu Fonts License | [DEJAVU_LICENSE.txt](DEJAVU_LICENSE.txt) |

### Weather Icons & Visual Assets

- **Meteocons (Weather Color SVG Icons)**:
  - Location: `resources/icons/meteocons/`
  - Author: Bas Milius
  - License: **MIT License** (Copyright (c) 2020-present Bas Milius)
  - License Document: [METEOCONS_LICENSE.txt](METEOCONS_LICENSE.txt)
  - *Note*: While both Toodle and Meteocons are under the MIT License, they are independent creative works by different authors.
- **Application Icons & Desktop Entries**:
  - Location: `resources/icons/toodle*.png`, `resources/icons/*.svg`, `resources/*.desktop`
  - Created specifically for Toodle and covered by the project's **MIT License**.

---

## 5. Aggregated License Files (`LICENSES/`)

The authoritative license texts aggregated within this directory:

```text
LICENSES/
├── README.ja.md                  # Comprehensive licensing notice (Japanese)
├── README.md                     # Comprehensive licensing notice (English)
├── LICENSE-MIT.txt               # Toodle Core MIT License full text
├── DEJAVU_LICENSE.txt            # Bitstream Vera / DejaVu Fonts License text
├── JETBRAINS_MONO_NL_OFL.txt     # SIL Open Font License 1.1 text
├── METEOCONS_LICENSE.txt         # MIT License (Bas Milius) text
├── OPEN_SANS_OFL.txt             # SIL Open Font License 1.1 text
├── ROBOTO_OFL.txt                # SIL Open Font License 1.1 text
└── THIRD_PARTY_AUDIT.md          # Upstream dependency license audit records (cargo-deny)
```

---

## 6. How to Extract Dependency Licenses for Packaging

Downstream packagers bundling Toodle into distribution packages (Arch AUR PKGBUILD, Debian `.deb`, Fedora `.rpm`, Flatpak, etc.) can generate unified third-party license reports using standard Rust tooling:

```bash
# Generate a unified HTML/Markdown report via cargo-about:
cargo install cargo-about
cargo about generate about.hbs > THIRD_PARTY_LICENSES_RUST.html

# Or bundle all raw license files into a YAML package:
cargo install cargo-bundle-licenses
cargo bundle-licenses --format yaml --output THIRDPARTY.yaml
```

---

## Summary

| Component | License / Governing Policy | Reference |
| :--- | :--- | :--- |
| **Toodle Core Source** | MIT License | [LICENSE](../LICENSE) / [LICENSE-MIT.txt](LICENSE-MIT.txt) |
| **Rust Dependencies** | Permissive / Weak-Copyleft | Enforced via [deny.toml](../deny.toml) |
| **System Shared Libraries** | System Provided (Wayland, libxkbcommon) | Dynamically linked from host OS |
| **Meteocons** | MIT License (Bas Milius) | [METEOCONS_LICENSE.txt](METEOCONS_LICENSE.txt) |
| **Roboto** | SIL Open Font License 1.1 | [ROBOTO_OFL.txt](ROBOTO_OFL.txt) |
| **Open Sans** | SIL Open Font License 1.1 | [OPEN_SANS_OFL.txt](OPEN_SANS_OFL.txt) |
| **JetBrains Mono NL** | SIL Open Font License 1.1 | [JETBRAINS_MONO_NL_OFL.txt](JETBRAINS_MONO_NL_OFL.txt) |
| **DejaVu Serif** | Bitstream Vera / DejaVu License | [DEJAVU_LICENSE.txt](DEJAVU_LICENSE.txt) |
| **App Icons & Assets** | MIT License | `resources/icons/`, `resources/*.desktop` |

---

<p align="center">
  <a href="../README.md">← Back to Root README (English)</a> | <a href="README.ja.md">日本語の案内はこちら →</a>
</p>
