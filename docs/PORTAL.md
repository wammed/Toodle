# 📚 Toodle Documentation Portal

Welcome to the official documentation portal for Toodle.  
Toodle is a modern, high-precision digital clock and weather widget engineered natively for the System76 COSMIC Desktop Environment and Linux Wayland compositors.

This portal provides structured access to all guides, architecture specifications, and references according to your role and objectives.

---

## 🧭 Comprehensive Documentation Navigation

| Document | Primary Contents | Target Audience |
| :--- | :--- | :--- |
| **[Quick Start](../README.md#-quick-start)** | Prerequisites, build instructions, local installation, running, autostart | New users and packagers |
| **[🖱️ Interaction & Controls Guide](SHORTCUTS.md)** | Mouse controls, calendar, weather forecast, Edit Layout mode, settings app | Users configuring or adjusting the widget |
| **[💡 Feature Specification (FEATURES)](FEATURES.md)** | Complete UI/UX specifications, 9 grid zones, 10 size stages, weather subsystem | Users exploring features and theme options |
| **[📐 Architecture Specification (ARCHITECTURE)](ARCHITECTURE.md)** | Multi-surface hierarchy, dynamic multi-monitor management, subsecond clock stream | Developers and system architects |
| **[🛡️ Reliability & Security Model (SECURITY)](SECURITY.md)** | 2-tier weather cache, atomic persistence, offline fallback, input region safety | Security auditors and curious developers |
| **[🛡️ IP Compliance Due Diligence (IP COMPLIANCE)](../IP_COMPLIANCE.md)** | Toodle icon provenance, similarity review findings, and IP due-diligence record | All users, contributors, and legal reviewers |
| **[🎨 Icon Design & IP Review History (ICON DESIGN HISTORY)](../ICON_DESIGN_HISTORY.md)** | Cross-application AI generation logs, similarity audit history, and design iterations | All users and legal reviewers |
| **[📄 Licensing & Third-Party Notice](../LICENSES/README.md)** | MIT License, Meteocons, embedded fonts, cargo-deny dependency compliance | Legal compliance officers and packagers |

---

## 🎯 Task-Oriented Guides

### 1. Build and Run Toodle for the First Time
- Refer to [README.md: Quick Start](../README.md#-quick-start) to compile binaries via `cargo build --release`.
- Easily install to `$HOME/.local/bin` using `./tools/install-local.sh`.
- Configure COSMIC session autostart and review the verified 3-second startup delay workaround.

### 2. Learn Widget Interaction and Layout Modification
- Review [SHORTCUTS.md](SHORTCUTS.md).
- Learn how to trigger the calendar and 7-day forecast from the right-click menu, manipulate the 9-zone layout and 10 size stages in real-time with zero flicker via the `Edit Layout Panel`, and use the `toodle-settings` application.

### 3. Check Dimensions, Themes, and Weather Integration
- Visit [FEATURES.md](FEATURES.md) for exact dimension charts (Stage 1 to 10), the 10 theme presets with curated palettes, the 2-tier Open-Meteo REST API cache architecture, and embedded vector color SVG icons.

### 4. Understand Architecture and Wayland Integration
- Visit [ARCHITECTURE.md](ARCHITECTURE.md) to explore the multi-surface separation between `Layer::Bottom` (widget surface) and `Layer::Top` (popups and edit panel), multi-monitor resolution tracking via `OutputManager`, and subsecond microsecond-aligned ticker streams.

### 5. Review Cache Integrity, Offline Behavior, and Privacy
- Consult [SECURITY.md](SECURITY.md) for request generation IDs preventing stale weather overwrites, clock skew protection, atomic file writing with PID temporary files, and our strict zero-telemetry policy.

### 6. Verify Licenses, Fonts, and Asset Clearances
- Consult [LICENSES/README.md](../LICENSES/README.md) for the core MIT license, Bas Milius's Meteocons (MIT), bundled fonts (Roboto, Open Sans, JetBrains Mono NL, DejaVu Serif), and automated `cargo-deny` audits documented in [LICENSES/THIRD_PARTY_AUDIT.md](../LICENSES/THIRD_PARTY_AUDIT.md).

### 7. For Legal, Compliance & IP Provenance
- Check the [Licensing Policy](../LICENSES/README.md) for source distribution, third-party crate governance, and font handling.
- Review the [IP Compliance Record](../IP_COMPLIANCE.md) for Toodle icon provenance and due diligence.
- Read the [Icon Design & IP Review History](../ICON_DESIGN_HISTORY.md) for the complete 4-app chronological audit log across Fluffy, Waddle, Rooney, and Toodle.

---

<p align="center">
  <a href="../README.md">← Back to Root README (English)</a> | <a href="PORTAL.ja.md">日本語ポータルはこちら →</a>
</p>
