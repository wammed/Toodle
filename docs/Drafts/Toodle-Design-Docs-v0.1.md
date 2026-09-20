# COSMIC Desktop Clock Widget

## 仕様書・設計書 v0.1

**Status:** Architecture Defined / Prototype Pending
**Target:** COSMIC Desktop
**Platform:** Linux / Wayland
**Language:** Rust
**UI Framework:** libcosmic

---

# 1. プロジェクト概要

COSMIC Desktop専用の、デスクトップ上に常駐する軽量な時計・天気ウィジェットを開発する。

壁紙を活かしたミニマルなデザインを基本とし、ウィジェット自体には背景を持たせず、時計・日付・天気などの情報のみを表示する。

通常時は最小限の情報だけを表示し、カレンダー・週間天気予報・設定などの追加機能は右クリックメニューから呼び出す。

---

# 2. 基本コンセプト

## 2.1 コンセプト

> **A lightweight, transparent desktop clock for COSMIC.**

「デスクトップに置いてある時計」であることを最優先する。

一般的なデスクトップアプリのようにウィンドウを開いて使用するものではなく、壁紙の一部として常時表示されるウィジェットを目指す。

---

# 3. 対象環境

## 3.1 対象OS / Desktop

* Linux
* COSMIC Desktop
* Wayland

## 3.2 サポート範囲

COSMIC Desktop専用とする。

対象外：

* GNOME
* KDE Plasma
* X11
* Sway
* Hyprland
* その他Desktop Environment / Compositor

将来的な移植性は考慮しない。COSMIC固有機能を積極的に利用する。

---

# 4. 技術スタック

## 4.1 Core

* Rust

## 4.2 UI

* libcosmic

## 4.3 Window / Surface

Wayland Layer Shellを使用する。

メインWidget：

```text
Layer Shell
    └── Layer::Bottom
```

目的：

* 壁紙より上
* 通常のアプリケーションより下
* デスクトップ上に常駐
* ワークスペース上で継続表示

---

# 5. Surface Architecture

アプリケーション内で用途に応じて異なるSurfaceを使用する。

```text
cosmic-clock
│
├── Desktop Widget
│   └── Wayland Layer Shell
│       └── Layer::Bottom
│
├── Calendar
│   └── Wayland Popup
│
├── Weekly Forecast
│   └── Wayland Popup
│
└── Settings
    └── Normal COSMIC Window
```

## 5.1 Desktop Widget

時計本体。

Wayland Layer Shellを利用する。

## 5.2 Calendar

右クリックメニューから開くPopup。

時計本体の状態には含めない。

## 5.3 Weekly Forecast

右クリックメニューから開くPopup。

時計本体の状態には含めない。

## 5.4 Settings

通常のCOSMICアプリケーションWindowとして表示する。

---

# 6. Widget State

Widget自体の状態は2種類のみとする。

```text
Widget
├── Normal
└── Settings
```

ただし、SettingsはDesktop WidgetそのものをSettings表示に切り替えるという意味ではなく、設定画面を開くための論理状態として扱う。

Calendar / Weekly Forecastは独立Popupとする。

---

# 7. Normal State

通常時の表示。

例：

```text
        14:32

     Sep 20, 2026
        Sunday

       ☀ 27°C
        Clear
```

表示要素：

* Time
* Date
* Day of week
* Current temperature
* Current weather condition
* Weather icon
* Theme

背景は完全透明。

---

# 8. 時計仕様

## 8.1 時刻形式

24時間制固定。

デフォルト：

```text
14:32
```

Seconds表示はSettingsから切り替え可能。

```text
14:32:18
```

## 8.2 Time Source

OSのローカルシステム時刻を使用する。

Weather APIには依存しない。

```text
System Clock
     │
     ▼
Clock Model
     │
     ▼
Clock Renderer
```

ネットワーク接続がなくても時計は正常動作する。

---

# 9. 日付・曜日

表示言語はEnglish (US)固定。

例：

```text
Sep 20, 2026
Sunday
```

Localizationを実装しない。

対象外：

* 日本語
* 英語以外の言語
* ユーザーによる言語変更

---

# 10. Widget Background

ウィジェットの背景は透明とする。

目標：

```text
Wallpaper
──────────────────────

       14:32
   Sep 20, 2026
      ☀ 27°C

──────────────────────
```

背景色、カード、ボーダーなどは表示しない。

---

# 11. Widget Layer

メインWidgetは、

```text
Wayland Layer Shell
Layer::Bottom
```

を第一候補とする。

想定される表示順：

```text
Overlay
Top
Panel
────────────────
Normal Windows
────────────────
Clock Widget
────────────────
Wallpaper
```

これにより、通常のアプリケーション操作を妨害しない。

---

# 12. Mouse Interaction

Normal Widgetはマウス操作を受け付ける。

対応：

* Left click
* Right click
* Resize interaction

通常時の左クリック動作は、v0.1では原則として定義しない。

右クリック：

```text
Right Click
    ↓
Context Menu
```

---

# 13. Context Menu

右クリックで以下のメニューを表示する。

```text
┌─────────────────────┐
│ Calendar             │
│ Weekly Forecast      │
│ ─────────────────── │
│ Settings             │
└─────────────────────┘
```

項目：

1. Calendar
2. Weekly Forecast
3. Settings

予定・TODO関連項目は存在しない。

---

# 14. Calendar

Calendarは独立したWayland Popupとして表示する。

## 14.1 機能

* 月間カレンダー
* Previous Month
* Next Month
* Today
* Current Date Highlight

例：

```text
       September 2026

      <              >

 Mon Tue Wed Thu Fri Sat Sun
      1   2   3   4   5   6
  7   8   9  10  11  12  13
 14  15  16  17  18  19 [20]
 21  22  23  24  25  26  27
 28  29  30

             Today
```

## 14.2 Scope

Calendarは日付確認専用。

実装しない：

* Events
* Schedule
* TODO
* Tasks
* Reminders
* Calendar synchronization

---

# 15. Weather

天気はユーザーが手動設定した固定地点について取得する。

自動位置情報取得は使用しない。

---

# 16. Location

ユーザーがSettingsからLocationを指定する。

例：

```text
Gifu, Japan
```

内部では、

```text
Location
├── name
├── latitude
└── longitude
```

として保存する。

Weather APIには緯度・経度を渡す。

一度設定されたLocationは毎回Geocodingしない。

Location変更時のみGeocodingを実行する方式を基本とする。

---

# 17. Weather Architecture

Weather APIとUIを分離する。

```text
UI
 │
 ▼
Weather Service
 │
 ▼
Weather Provider
 │
 ▼
Weather API
```

## 17.1 Weather Provider

Providerは交換可能な設計とする。

```text
trait WeatherProvider
```

相当の抽象化を行い、特定APIへの依存をUI層へ持ち込まない。

---

# 18. Current Weather

Normal Widgetに現在の天気を表示する。

例：

```text
☀ 27°C
Clear
```

最低限：

* Temperature
* Weather Condition
* Weather Icon

を表示する。

---

# 19. Temperature Unit

Settingsで変更可能。

```text
Celsius
Fahrenheit
```

デフォルト：

```text
Celsius
```

---

# 20. Weekly Forecast

右クリックメニューからPopupとして表示する。

7日程度の予報を表示する。

例：

```text
Weekly Forecast

Mon  Sep 21    ☀   28°C
Tue  Sep 22    ☀   27°C
Wed  Sep 23    ☁   25°C
Thu  Sep 24    🌧  23°C
Fri  Sep 25    ☀   26°C
Sat  Sep 26    ☀   28°C
Sun  Sep 27    ☀   29°C
```

最低限表示：

* Day
* Date
* Weather icon
* Temperature

---

# 21. Weather Cache

ClockとWeatherは完全に独立させる。

```text
Clock
    ↓
System Time
    ↓
Continuous update

Weather
    ↓
Network
    ↓
Cached result
```

目安：

```text
Current Weather
→ 約30分

Forecast
→ 約3時間
```

正確な更新間隔は実装時に調整する。

---

# 22. Network Failure

Weather APIに接続できなくてもWidget全体は正常動作する。

優先順位：

```text
Fresh Weather
    ↓
Cached Weather
    ↓
Unavailable
```

表示例：

```text
Weather unavailable
```

または最後に取得したデータを表示する。

時計・カレンダー・Settingsはネットワーク障害の影響を受けない。

---

# 23. Theme System

フォント単体ではなく、Widget全体をThemeとして扱う。

v0.1では約10種類。

候補：

```text
01 Minimal
02 Modern
03 Digital
04 Thin
05 Mono
06 Classic
07 Bold
08 Elegant
09 Compact
10 Retro
```

---

# 24. Theme Model

```text
Theme
├── id
├── name
├── font
├── clock style
├── date style
├── day style
├── weather style
├── spacing
├── alignment
└── weather icon style
```

ThemeとColorは分離する。

```text
Theme
   +
Theme Color
   ↓
Rendered Widget
```

---

# 25. Theme Color

壁紙による視認性低下を防ぐため、ユーザーが文字色を選択できるようにする。

v0.1では16色の固定パレットを使用する。

例：

```text
● ● ● ●
● ● ● ●
● ● ● ●
● ● ● ●
```

候補：

* Black
* Dark Gray
* Gray
* White
* Red
* Orange
* Yellow
* Green
* Cyan
* Blue
* Indigo
* Purple
* Pink
* Brown
* Lime
* Teal

最終的なHEX/RGB値はデザインフェーズで決定する。

---

# 26. Theme Color適用範囲

基本的に選択された色をWidget全体へ適用する。

対象：

* Clock
* Date
* Day
* Weather
* Weather icon
* Calendar
* Forecast

Settings UIについてはCOSMICの標準UIとの整合性を優先する。

---

# 27. Text Shadow

透明背景上での視認性を補助するためText Shadowを使用可能とする。

Settings：

```text
Text Shadow
☑ Enable
```

Shadowは控えめにする。

目的：

> 壁紙と文字色のコントラストが不足する場合の補助

であり、装飾目的ではない。

---

# 28. Resize

Widgetはサイズ変更可能とする。

ただしLayer Shell surfaceは通常のWindow Managerによるresizeとは挙動が異なるため、Widget内部にResize Interactionを実装する方式を第一候補とする。

例：

```text
       14:32
   Sep 20, 2026
      ☀ 27°C

                 ╲
                  ╲
                   ╲
                    ◇
```

右下などにResize Handleを設ける方式を検討する。

サイズ変更後：

```text
Widget Size
    ↓
Layout calculation
    ↓
Font / spacing / element size
```

を再計算する。

---

# 29. Widget Position

Desktop上の位置をユーザーが変更可能とする。

第一候補：

* Widgetをドラッグして移動
* 現在位置を保存
* 次回起動時に復元

ただしLayer Shellのanchor/position制約についてはPrototypeで確認する。

実装が困難な場合のFallback：

```text
Settings
├── Anchor
│   ├── Top Left
│   ├── Top Right
│   ├── Bottom Left
│   └── Bottom Right
│
├── Margin X
└── Margin Y
```

---

# 30. Settings

Settingsは通常のCOSMIC Windowとして開く。

右クリック：

```text
Settings
   ↓
Normal COSMIC Window
```

---

# 31. Settings UI

```text
Clock Settings

Appearance
────────────────

Theme
[ Modern ▼ ]

Theme Color
● ● ● ●
● ● ● ●
● ● ● ●
● ● ● ●

Font Size
[────────●──────]

☑ Text Shadow

Clock
────────────────

☑ Show Date
☑ Show Weather
☐ Show Seconds

Weather
────────────────

Location
[ Gifu, Japan ]

Temperature
○ Celsius
○ Fahrenheit
```

---

# 32. Settings項目

## Appearance

* Theme
* Theme Color
* Font Size
* Text Shadow

## Clock

* Show Date
* Show Weather
* Show Seconds

## Weather

* Location
* Temperature Unit

---

# 33. Configuration Persistence

以下を永続化する。

```text
Theme
Theme Color
Font Size
Text Shadow
Show Date
Show Weather
Show Seconds
Location
Temperature Unit
Widget Position
Widget Size
```

COSMIC/Linuxの標準的なユーザー設定保存場所を利用する。

設定フォーマットはJSON / TOML等を候補とし、実装時に決定する。

---

# 34. Data Model

## 34.1 Clock

```text
ClockState
├── time
├── date
└── weekday
```

## 34.2 Location

```text
Location
├── name
├── latitude
└── longitude
```

## 34.3 Current Weather

```text
CurrentWeather
├── temperature
├── weather_code
├── condition
└── timestamp
```

## 34.4 Forecast

```text
DailyForecast
├── date
├── weather_code
├── condition
├── temperature_min
└── temperature_max
```

---

# 35. Recommended Project Structure

```text
cosmic-clock/
│
├── Cargo.toml
├── README.md
├── LICENSE
│
├── src/
│   ├── main.rs
│   ├── app.rs
│   │
│   ├── clock/
│   │   ├── mod.rs
│   │   └── time.rs
│   │
│   ├── weather/
│   │   ├── mod.rs
│   │   ├── model.rs
│   │   ├── provider.rs
│   │   └── service.rs
│   │
│   ├── location/
│   │   ├── mod.rs
│   │   └── location.rs
│   │
│   ├── calendar/
│   │   ├── mod.rs
│   │   └── model.rs
│   │
│   ├── theme/
│   │   ├── mod.rs
│   │   ├── themes.rs
│   │   └── colors.rs
│   │
│   ├── settings/
│   │   ├── mod.rs
│   │   └── config.rs
│   │
│   └── ui/
│       ├── mod.rs
│       ├── widget.rs
│       ├── context_menu.rs
│       ├── calendar.rs
│       ├── forecast.rs
│       └── settings.rs
│
└── resources/
    └── fonts/
```

---

# 36. UI LayerとDomain Layerの分離

UIからDomain Logicへ直接依存しすぎない。

```text
             UI
              │
              ▼
        Application State
              │
      ┌───────┼────────┐
      ▼       ▼        ▼
    Clock   Weather  Calendar
              │
              ▼
         Infrastructure
              │
              ▼
          HTTP / OS
```

Weather API、System Clock、Configuration StorageなどはUIから独立させる。

---

# 37. Async / Background Processing

ネットワーク処理はUI threadで実行しない。

```text
UI
 │
 ├── Clock update
 │
 └── Weather request
          │
          ▼
     async task
          │
          ▼
      API request
          │
          ▼
      Weather Model
          │
          ▼
           UI
```

Weather APIの取得には非同期処理を使用する。

候補：

* Tokio
* async HTTP client

---

# 38. Clock Update

時計は毎秒更新可能とする。

ただし描画自体は変更があった場合のみ行うことを検討する。

Seconds非表示時は、

```text
14:32
```

が表示されるため、毎秒描画する必要性は低い。

---

# 39. Performance Goals

アプリは軽量性を重視する。

目標：

* Normal時のCPU使用率を極小化
* 不要なネットワーク通信を行わない
* Weather APIを毎秒呼び出さない
* 不要な再描画を行わない
* 常駐メモリを可能な限り小さくする

特にNormal状態ではClock / Weather / UIを必要最小限の更新にする。

---

# 40. Startup

ログイン時の自動起動に対応する。

```text
Login
  ↓
COSMIC Desktop
  ↓
cosmic-clock
  ↓
Desktop Widget
```

自動起動方式はCOSMIC/Linuxの標準方式を調査して採用する。

---

# 41. Multi-Monitor

v0.1では最低限、複数モニター環境でWidgetが正常動作することを確認する。

検討項目：

* Monitorごとの位置
* Monitor移動
* Resolution変更
* Monitor disconnect
* Workspace切り替え

初期実装では、複数Widgetを同時に配置する機能は必須としない。

---

# 42. Error Handling

エラーはWidget全体を停止させない。

例：

```text
Weather API error
      ↓
Use cache
      ↓
If unavailable
      ↓
Hide weather / show unavailable
```

Settings読み込みエラー時は安全なデフォルト値を使用する。

---

# 43. セキュリティ / Privacy

自動位置情報取得は行わない。

位置情報はユーザーが明示的に設定した地点のみ使用する。

アプリが利用する外部通信はWeather APIのみとする。

ユーザーの位置情報を第三者へ送信する目的の機能は持たない。

---

# 44. Scope外

v0.1では以下を実装しない。

* Calendar events
* TODO
* Task manager
* Reminders
* Calendar synchronization
* Automatic location
* GPS location
* Multi-language
* Google Calendar
* Microsoft Calendar
* Multiple widget instances
* Animated weather
* Weather alerts
* Sunrise / Sunset
* Moon phase
* Custom arbitrary color
* Custom font upload
* Cross-desktop compatibility

---

# 45. MVP開発フェーズ

## Phase 0 — Technology Prototype

最重要フェーズ。

実装：

```text
Transparent Layer Shell Widget
```

確認：

* COSMICで起動
* Layer::Bottom
* Wallpaper上
* Normal Windowより下
* Transparent
* Mouse input
* Right click
* Resize
* Position
* Workspace behavior

時計・天気はまだ実装しない。

---

## Phase 1 — Clock

* System Clock
* 24-hour display
* Date
* Weekday
* Seconds option
* Responsive layout

---

## Phase 2 — Theme

* 10 Themes
* 16 Colors
* Text Shadow
* Font scaling
* Theme persistence

---

## Phase 3 — Interaction

* Right-click menu
* Popup management
* Calendar Popup
* Settings Window

---

## Phase 4 — Calendar

* Month navigation
* Today
* Current date highlight

---

## Phase 5 — Weather

* WeatherProvider
* Manual Location
* Current weather
* Temperature
* Weather condition
* Weather icon
* Cache
* Error handling

---

## Phase 6 — Forecast

* 7-day forecast
* Forecast Popup
* Temperature
* Weather icons

---

## Phase 7 — Persistence / Polish

* Widget position
* Widget size
* Settings persistence
* Auto start
* Multi-monitor
* Performance
* Error handling

---

# 46. Prototype 0の成功条件

Prototype 0では、以下を満たせば成功とする。

```text
COSMIC Desktop
       │
       ▼
┌──────────────────────┐
│                      │
│  Transparent Surface │
│                      │
└──────────────────────┘
```

### 必須

* [ ] Rust project builds
* [ ] libcosmic starts
* [ ] Layer Shell surface
* [ ] Layer::Bottom
* [ ] Transparent background
* [ ] Surface visible above wallpaper
* [ ] Surface below normal windows
* [ ] Mouse input
* [ ] Right click
* [ ] Resize mechanism
* [ ] Position mechanism

### Prototype 0では不要

* Clock
* Weather
* Calendar
* Settings
* Theme
* Persistence

---

# 47. MVP完成条件

## Desktop Widget

* [ ] COSMIC Desktop常駐
* [ ] Transparent
* [ ] Layer::Bottom
* [ ] Resize
* [ ] Position
* [ ] Position persistence
* [ ] Size persistence

## Clock

* [ ] 24-hour
* [ ] Date
* [ ] Weekday
* [ ] Optional seconds

## Theme

* [ ] 10 themes
* [ ] 16-color palette
* [ ] Text Shadow
* [ ] Responsive font size

## Calendar

* [ ] Monthly view
* [ ] Previous / Next
* [ ] Today
* [ ] Current date highlight

## Weather

* [ ] Manual location
* [ ] Current weather
* [ ] Temperature
* [ ] Weather condition
* [ ] Weather icon
* [ ] 7-day forecast
* [ ] Cache
* [ ] Offline fallback

## Interaction

* [ ] Right-click menu
* [ ] Calendar popup
* [ ] Forecast popup
* [ ] Settings window

## Language

* [ ] English (US) only

## Scope

* [ ] No events
* [ ] No TODO
* [ ] No automatic location

---

# 48. 主要な技術リスク

## Risk 1 — Layer Shellのサイズ変更

通常のWindow Manager resizeとは異なるため、Widget内部のResize Handle方式を第一候補とする。

**優先度：High**

---

## Risk 2 — 自由な位置移動

Layer Shellのanchor / marginモデルとの兼ね合いをPrototype 0で確認する。

**優先度：High**

Fallbackとしてanchor + marginを用意する。

---

## Risk 3 — 完全透明

Surfaceのalpha / rendering / compositor側の挙動を実機で確認する。

**優先度：High**

---

## Risk 4 — Popupのfocus / input

Calendar / Forecast / Context MenuがDesktop Widgetと干渉しないことを確認する。

**優先度：Medium**

---

## Risk 5 — COSMICのAPI変化

libcosmicは開発が進行中であるため、特定バージョンに依存するAPIを最小化する。

**優先度：Medium**

---

# 49. 設計上の重要原則

1. **COSMIC専用として設計する**
2. **Desktop Widgetと通常Windowを明確に分離する**
3. **Main WidgetにはLayer Shellを使用する**
4. **Calendar / ForecastはPopupにする**
5. **Settingsは通常Windowにする**
6. **ClockとWeatherを完全に分離する**
7. **Weather APIをUIから分離する**
8. **ネットワーク障害でClockを停止させない**
9. **透明背景を基本とする**
10. **視認性のため16色Paletteを提供する**
11. **ThemeとColorを分離する**
12. **Normal状態を可能な限り軽量にする**
13. **予定・TODOなどの機能を追加しない**
14. **English (US)に固定する**
15. **最初にLayer Shell Prototypeを完成させる**

---

# 50. 最終アーキテクチャ

```text
                         cosmic-clock
                              │
              ┌───────────────┼────────────────┐
              │               │                │
              ▼               ▼                ▼
       Desktop Widget      Popups          Settings
              │               │                │
        Layer Shell       Wayland Popup    Normal Window
        Layer::Bottom          │
              │          ┌─────┴─────┐
              │          │           │
              │      Calendar     Forecast
              │
       ┌──────┼──────────────┐
       │      │              │
       ▼      ▼              ▼
     Clock  Theme         Weather
       │      │              │
       │      │         Weather Service
       │      │              │
       │      │         Weather Provider
       │      │              │
       │      │           Weather API
       │
       ▼
  System Time

       └──────────────┐
                      ▼
               Configuration
                      │
                      ▼
                Persistent Storage
```

---

# 51. プロジェクトの完成イメージ

最終的なデスクトップ上では、

```text
┌──────────────────────────────────────────────┐
│                                              │
│                                              │
│                  14:32                       │
│              Sep 20, 2026                    │
│                 Sunday                       │
│                ☀ 27°C                       │
│                                              │
│                                              │
│                              Wallpaper       │
│                                              │
└──────────────────────────────────────────────┘
```

という非常にシンプルな状態を基本とする。

右クリックすると、

```text
┌──────────────────────┐
│ Calendar             │
│ Weekly Forecast      │
│ ─────────────────── │
│ Settings             │
└──────────────────────┘
```

必要な情報だけをPopupとして展開する。

---

# 52. 開発開始時の最初のタスク

最初に実装するものは時計ではない。

以下の最小Prototypeを作る。

```text
Rust
  +
libcosmic
  +
Wayland Layer Shell
  +
Layer::Bottom
  +
Transparent Surface
```

その上で、

1. COSMIC Desktopで起動
2. 壁紙上に表示
3. 通常Windowの下に表示
4. マウス入力確認
5. Right Click確認
6. Resize確認
7. Position確認
8. Workspace確認

を行う。

**このPrototypeが成立したことを確認してからClock UIの実装へ進む。**

これを本プロジェクトの最初の技術的マイルストーンとする。
