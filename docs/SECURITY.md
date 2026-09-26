# 🛡️ Toodle Reliability & Security Model (SECURITY)

<p align="center">
  <strong>English</strong> | <a href="SECURITY.ja.md">日本語</a> | <a href="PORTAL.md">📚 Documentation Portal</a> | <a href="../README.md">← Root README</a>
</p>

---

This document outlines the data protection principles, offline resilience, cache integrity design, filesystem I/O safety, and Wayland layer shell isolation models implemented in **Toodle**.

---

## 1. Zero Telemetry & Privacy Principles

Toodle is built on a strict privacy-first foundation:

- **Zero Tracking / Zero Telemetry**:
  - No user activity tracking, remote crash telemetry, analytics pings, or background advertising beacons.
- **Strict Data Isolation**:
  - Toodle restricts its disk access strictly to `~/.config/toodle/` (configuration and weather cache). It never scans user directories, inspects keystrokes, or probes memory belonging to other processes.
- **Unprivileged Operation**:
  - Toodle runs entirely within standard user permissions. It requires no root privileges, `sudo` access, kernel capabilities, or setuid binaries.

---

## 2. Weather Subsystem Network Security & Cache Integrity

Toodle utilizes the [Open-Meteo API](https://open-meteo.com/) for forecast data. Network operations and caching are fortified against race conditions, network failures, and data corruption:

### ① 2-Tier Persistent Local Cache
- **Current Weather**: 30-minute persistent cache.
- **Weekly Forecast**: 3-hour persistent cache.
- **Offline Resilience**:
  - During network outages or API downtime, Toodle continues displaying the latest cached forecast seamlessly.
  - If no cache exists, the UI safely falls back to clean placeholder dashes (`--`) without freezing or crashing.

### ② Request Generation Counter (Stale Response Protection)
Asynchronous network responses may arrive out of order during variable latency. To prevent delayed responses from overwriting newer user requests:
- Every weather update request is tagged with a monotonically increasing **Generation ID**.
- Responses that do not match the latest generation counter are immediately discarded upon arrival.

### ③ Clock Skew Defense & Strict Schema Validation
- **Clock Skew Protection**: Handles abrupt NTP corrections or system clock shifts safely to prevent cache expiration locks.
- **Rigid JSON Validation**: Incoming API payloads are validated for array completeness (minimum 7-day forecast entries), sensible temperature/precipitation ranges, and absence of invalid NaN/null values before disk caching.

---

## 3. Filesystem I/O Safety & Atomic Writes

Configuration (`config.toml`) and cache (`cache.json`) files are written using an atomic persistence pattern to prevent corruption during sudden power losses or SIGKILL events:

```text
[Data Payload]
     │
     ▼
Write to PID-tagged temporary file (`.{filename}.tmp.{pid}`)
     │
     ▼
Kernel flush to physical storage (`sync_all()`)
     │
     ▼
Atomic filesystem replacement (`fs::rename`) ───> Target File (Zero Corruption)
```

1. **PID-Scoped Temporary Files**:
   - Written to `.{filename}.tmp.{pid}` to avoid conflicts between concurrent processes or rapid consecutive edits.
2. **Physical Disk Sync (`sync_all`)**:
   - Ensures data is committed from OS buffer cache to non-volatile storage.
3. **Atomic `fs::rename`**:
   - Uses POSIX atomic file replacement, guaranteeing that readers never encounter a partially written or truncated file.

---

## 4. Wayland Layer Shell Isolation & Input Pass-Through

Toodle interfaces with Wayland via the standard `wlr-layer-shell` protocol:

- **Minimized Input Region**:
  - The widget never captures input across the entire screen. Only the bounding box around active text and icons (`content_bounds`) is submitted as an active input region.
  - Surrounding desktop space allows mouse clicks to pass straight through to desktop icons and wallpaper handlers without click-jacking risk.
- **Surface Layer Separation**:
  - The ambient clock stays on `Layer::Bottom`, never covering normal application windows (`Layer::Top` / XDG Toplevels).
  - Popups and the Edit Layout Panel only appear temporarily on `Layer::Top` during active interaction and are immediately destroyed when closed.

---

<p align="center">
  <a href="PORTAL.md">📚 Documentation Portal</a> | <a href="../README.md">← Root README</a> | <a href="SECURITY.ja.md">日本語仕様書はこちら →</a>
</p>
