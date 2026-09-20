# COSMIC Desktop Clock Widget

## 仕様書・設計書 v0.2 決定稿

**Status:** Architecture Finalized / Ready for Prototype 0  
**Target:** COSMIC Desktop  
**Platform:** Linux / Wayland  
**Language:** Rust  
**UI Framework:** libcosmic

---

# 1. Core Rules

このプロジェクトの設計上の基本ルールを以下に固定する。

1. WidgetはLayer Shell / Bottomで表示する。
2. PopupはLayer Shell / Topで表示する。
3. Settingsは通常のXDG Toplevel Windowとする。
4. WidgetはNormal ModeとEdit Modeの2状態だけを持つ。
5. Normal Modeではcontent boundsだけをInput Regionにする。
6. Edit Modeではwidget全体をInput Regionにする。
7. 操作は `Right Click → Context Menu → Edit Layout → Edit Mode` に統一する。
8. 設定は`config.toml`に保存する。
9. Widgetはfile watcherを第一方式として設定変更を検知する。
10. `SIGUSR1`はfallback / 将来拡張とする。
11. Weather Serviceの外部API境界は`WeatherProvider`とする。
12. API Key管理はv0.2では行わない。
13. 時刻はOS local timeを使用する。
14. Tickは次の秒境界に合わせる。
15. Widgetは1インスタンスのみとし、表示先outputを設定で指定する。

---

# 2. Project Concept

COSMIC Desktop上に常駐する、透明なデスクトップ時計・天気Widgetを実装する。

Widgetは壁紙上に直接表示され、通常のアプリケーションウィンドウより背面に配置する。

通常時に表示する情報は以下とする。

- デジタル時計
- 秒
- 日付
- 曜日
- 現在の天気
- 現在の気温

追加機能として以下をPopupで提供する。

- Context Menu
- Monthly Calendar
- Weekly Forecast

設定はWidget本体とは分離した通常のCOSMIC Windowとして提供する。

---

# 3. Non-Goals

v0.2では以下を実装対象外とする。

- `xdg_popup`の使用
- 通常時の自由ドラッグによる配置変更
- 自動位置情報取得
- GPS / Network Location
- API Key管理
- イベント
- TODO
- カレンダー同期
- リマインダー
- 複数Widgetインスタンス
- カスタムフォントファイルのユーザー読み込み
- 任意のフォントアップロード
- 任意カラー設定
- ローカライズ
- 天気アラート
- Sunrise / Sunset
- Moon Phase
- 他Desktop Environmentへの対応
- Windows / macOS対応

---

# 4. Process Architecture

プロジェクトは2つのバイナリで構成する。

```text
cosmic-clock
├── Desktop Widget
│   └── Layer Shell / Bottom
│
└── Popup
    └── Layer Shell / Top

cosmic-clock-settings
└── XDG Toplevel Window
```

## 4.1 cosmic-clock

Widget本体を常駐させる。

責務：

- 時計表示
- 日付・曜日表示
- 天気表示
- Widget配置
- Widgetサイズ
- Input Region管理
- Context Menu表示
- Calendar Popup表示
- Weekly Forecast Popup表示
- 設定ファイル監視
- 設定変更の再読み込み

## 4.2 cosmic-clock-settings

通常のCOSMIC Windowとして起動する。

責務：

- Display設定
- Layout設定
- Theme設定
- Font Scale設定
- Text Shadow設定
- Weather Location設定
- Temperature Unit設定

設定値は`config.toml`へ保存する。

設定UIを開いていない平時は、設定GUIプロセスを起動せず、Widget本体のリソース消費を最小限に抑える。

---

# 5. Surface Architecture

COSMIC Desktop上のSurface構成を固定する。

```text
Overlay / Top
    Popup Surface
        ├── Context Menu
        ├── Calendar
        └── Weekly Forecast

Normal Application Windows

Bottom
    Desktop Widget

Wallpaper
```

## 5.1 Desktop Widget

WidgetはLayer ShellのBottom Layerに配置する。

目的：

- 壁紙の上に表示する
- 通常のアプリケーションWindowより背面に置く
- 透明部分をクリック透過させる

Widget背景は完全透明とする。

## 5.2 Popup Surface

Context Menu、Calendar、Weekly ForecastはすべてPopup Surfaceとして扱う。

PopupはLayer ShellのTop Layerに独立したSurfaceとして生成する。

Popupは必要時のみ表示し、表示時はフォーカスを取得する。

フォーカスを失った場合は自動的に閉じる。

---

# 6. Widget State

Widgetは2状態だけを持つ。

```text
Normal Mode
Edit Mode
```

## 6.1 Normal Mode

通常のWidget表示状態。

- 時計を表示
- 日付・曜日を表示
- 天気を表示
- Input Regionはcontent boundsのみ
- 透明部分はクリック透過
- Right ClickでContext Menuを開く

## 6.2 Edit Mode

Widgetの配置とサイズを変更するための状態。

遷移：

```text
Normal Mode
    ↓ Right Click
Context Menu
    ↓ Edit Layout
Edit Mode
```

Edit ModeではWidget全体をInput Regionとする。

- Widget全体を操作可能
- Anchorを4隅から選択
- Margin X/Yを変更
- Resize Handleでサイズ変更
- Doneで確定
- Escでキャンセル
- outside clickで確定してNormal Modeへ戻る

確定後は新しい配置・サイズを`config.toml`へ保存する。

Normal Modeへ戻った後、Input Regionはcontent boundsのみに戻す。

---

# 7. Input Region

Input Regionのルールを固定する。

## Normal Mode

```text
Input Region = content bounds only
```

時計、日付、天気など実際に描画されているコンテンツ領域だけをInput Regionとする。

コンテンツ外の透明領域は完全にクリック透過する。

## Edit Mode

```text
Input Region = entire widget surface
```

Widget全体をInput Regionとして、配置・サイズ変更を可能にする。

実装には`wl_surface.set_input_region`を使用する。

---

# 8. Layout

通常時の配置は自由ドラッグではなく、Anchor + Marginで管理する。

Anchorは以下の4種類。

```text
TopLeft
TopRight
BottomLeft
BottomRight
```

配置は以下で表現する。

```text
Anchor
Margin X
Margin Y
```

サイズは以下で表現する。

```text
Width
Height
```

Edit ModeではAnchorを固定した状態でMargin X/YとWidth/Heightを変更する。

この方式により、通常時に連続的なWindow移動処理を行わず、Compositorへの負荷を抑える。

---

# 9. Clock

## 9.1 Display

24時間制を使用する。

秒を常に表示する。

例：

```text
14:32:18
```

日付と曜日も表示する。

例：

```text
Saturday, September 20, 2026
```

曜日・日付表記はEnglish (US)に固定する。

## 9.2 Time Source

```text
Time source:
OS local time
```

Implementation:

```text
time / chrono等のRust date-time crateを実装時に選定
```

## 9.3 Tick

時計の更新は次の秒境界に合わせてTickする。

```text
14:32:18.xxx
        ↓
14:32:19.000
```

Tickは秒表示のドリフトや表示スキップを避けるため、次の秒境界を基準とする。

---

# 10. Fonts

Themeで使用するフォントはプロジェクト内に保持する。

```text
resources/
└── fonts/
```

フォントは`include_bytes!`等によってBinaryへ組み込む。

外部システムフォントには依存しない。

目的：

- オフラインでも同一表示
- フォントメトリクスの固定
- 環境による表示差の削減

---

# 11. Weather Architecture

Weather機能は以下の構成とする。

```text
UI
 ↓
Weather Service
├── Cache
├── Worker
└── WeatherProvider
        ↓
    Weather API
```

Weather ServiceがUIと外部Weather APIの間を担当する。

## 11.1 WeatherProvider

外部Weather APIとの境界を以下のTraitにする。

```rust
trait WeatherProvider
```

Weather Providerの具体的な実装はWeather Serviceから分離する。

v0.2ではAPI Key管理を行わない。

---

# 12. Geocoding

Location設定時はGeocodingを使用する。

入力は500〜800ms debounceする。

入力値について以下を検証する。

- trim
- length check
- 空文字チェック

Geocoding結果はPersistent Cacheへ保存する。

同一のLocation Nameについては、不要な再問い合わせを行わない。

---

# 13. Weather Cache

WeatherデータはCacheする。

```text
Current Weather
    Cache: 30 minutes

Forecast
    Cache: 3 hours
```

Cacheが有効な場合はNetwork Requestを行わない。

OfflineまたはRequest Errorの場合は、利用可能なら最後のCacheを表示する。

Cacheが存在しない場合は、

```text
Weather unavailable
```

を表示する。

---

# 14. Network Control

Weather Network処理はBackground Workerで実行する。

HTTP 429 / 5xxではExponential Backoffを使用する。

```text
1
2
4
8
...
max 30 minutes
```

Network処理によってUI Threadをブロックしない。

---

# 15. Current Weather

Widgetの通常表示では以下を表示する。

- Weather condition
- Temperature

Temperature Unitは設定可能とする。

```text
Celsius
Fahrenheit
```

---

# 16. Weekly Forecast

Weekly ForecastはPopup Surfaceとして表示する。

予報は7日間とする。

各日について以下を表示する。

- Date
- Weather condition
- Minimum temperature
- Maximum temperature

---

# 17. Monthly Calendar

Monthly CalendarはPopup Surfaceとして表示する。

表示内容は月間カレンダーのみとする。

Events、TODO、Reminder等は扱わない。

---

# 18. Context Menu

Right ClickでContext Menuを表示する。

基本項目：

```text
Calendar
Weekly Forecast
Edit Layout
Settings
Quit
```

`Edit Layout`を選択するとEdit Modeへ遷移する。

`Settings`を選択すると`cosmic-clock-settings`を起動する。

---

# 19. Theme

Themeは固定Preset方式とする。

v0.2では約10種類のTheme Presetを用意する。

Themeには以下を含める。

- Font
- Letter Spacing
- Layout
- Visual Style

Colorは固定16色のPaletteから選択する。

Text ShadowはON / OFFを切り替え可能とする。

任意のRGB Color設定は行わない。

---

# 20. Configuration

設定は`config.toml`に保存する。

例：

```toml
[display]
output = "DP-1"

[layout]
anchor = "TopRight"
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
location_name = "Gifu, Japan"
latitude = 35.4233
longitude = 136.7606
temperature_unit = "Celsius"
```

API Keyは設定項目として持たない。

---

# 21. Configuration Reload

設定変更の経路を一本化する。

```text
cosmic-clock-settings
        ↓
    config.toml
        ↓
   file watcher
        ↓
    cosmic-clock
        ↓
      reload
```

File watcher / inotifyを第一方式とする。

設定ファイル更新時にはWidgetが設定を再読み込みする。

SIGUSR1はfallback / 将来拡張とする。

設定ファイルの書き込みは可能な限りAtomic Writeとする。

---

# 22. Display / Multi-Monitor

Widgetは1インスタンスのみ起動する。

表示先はConfigurationで指定する。

```toml
[display]
output = "DP-1"
```

`output`にはPrimary DisplayまたはDisplay Nameを指定できる。

複数Displayへ同時にWidgetを表示することはv0.2では行わない。

---

# 23. Project Structure

基本構成：

```text
cosmic-clock/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── widget/
│   ├── popup/
│   ├── clock/
│   ├── weather/
│   ├── config/
│   └── settings/
├── resources/
│   └── fonts/
└── config/
```

Settings BinaryはWidget Binaryとは分離する。

---

# 24. Prototype 0

Prototype 0では、機能を実装する前にDesktop Surface Architectureを検証する。

## 24.1 実装対象

1. Layer Shell / Bottom
2. 完全透明Widget
3. Text描画
4. Normal ModeのInput Region
5. Edit ModeのInput Region
6. Context Menu
7. Layer Shell / Top Popup
8. Anchor / Marginによる配置
9. Resize Handle
10. OutputへのAttach

## 24.2 Pass Criteria

以下をすべて満たした場合、Prototype 0を完了とする。

1. WidgetのTextがWallpaper上に表示され、通常のApplication Windowより背面にある。
2. Normal Modeで透明な非コンテンツ領域のクリックが完全に透過する。
3. Context MenuからEdit Layoutを選択するとEdit Modeに入り、配置（マージン）・サイズを変更して確定できる。
4. Edit ModeではWidget全体を操作できる。
5. Edit Mode終了後、Normal ModeのInput Regionがcontent boundsだけに戻る。
6. Right ClickでTest PopupがLayer::Topに表示され、通常のApplication Windowより前面に出る。
7. Popupはoutside click / focus lossで閉じる。
8. 指定したOutputへWidgetをAttachできる。

Prototype 0ではWeather、Calendar、Settingsなどの実装を行わない。

---

# 25. Implementation Roadmap

## Phase 0 — Desktop Surface

- Layer Shell / Bottom
- Transparent Widget
- Input Region
- Normal / Edit Mode
- Anchor / Margin
- Resize
- Layer Shell / Top Popup
- Popup focus / dismissal
- Output attach

## Phase 1 — Clock

- Embedded Fonts
- Clock rendering
- OS local time
- Next-second Tick
- Date
- Weekday

## Phase 2 — Weather

- Weather Service
- WeatherProvider
- Geocoding
- Background Worker
- Cache
- Rate Limit
- Exponential Backoff
- Offline / Error fallback

## Phase 3 — Popup Features

- Context Menu
- Monthly Calendar
- 7-day Weekly Forecast

## Phase 4 — Settings

- `cosmic-clock-settings`
- Display selection
- Layout
- Theme
- Color Palette
- Font Scale
- Text Shadow
- Location
- Temperature Unit
- config.toml
- File watcher reload

---

# 26. Design Decision Summary

このプロジェクトでは、Desktop Widgetを通常のWindowとして扱わない。

```text
Widget
    = Layer Shell / Bottom

Popup
    = Layer Shell / Top

Settings
    = XDG Toplevel
```

Widgetの状態は、

```text
Normal Mode
Edit Mode
```

の2つだけとする。

Input Regionは、

```text
Normal Mode
    = content bounds

Edit Mode
    = entire widget surface
```

とする。

配置は、

```text
Anchor + Margin
```

で管理する。

設定は、

```text
config.toml
    ↓
file watcher
    ↓
Widget reload
```

で反映する。

Weather APIとの境界は、

```text
WeatherProvider
```

だけとする。

時刻は、

```text
OS local time
```

を使用し、Tickは次の秒境界に合わせる。

これをv0.2の実装上の基本設計として固定する。
