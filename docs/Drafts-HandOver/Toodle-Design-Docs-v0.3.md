# Toodle Design Document v0.3

**Status:** Phase 0実機検証反映版 / Current Implementation Baseline  
**Target:** COSMIC Desktop  
**Platform:** Linux / Wayland  
**Language:** Rust  
**UI Framework:** libcosmic

> v0.3 は、Phase 0 の実機検証によって確定した現在の実装アーキテクチャを正とし、v0.2 の設計との差異を「設計変更」として反映した版である。
>
> 本書では、過去の設計を維持することよりも、現在の `toodle` 実装と実機で成立している Surface / State / 操作モデルとの整合性を優先する。

---

# 1. Core Rules

Toodle の現在の設計上の基本ルールを以下に固定する。

1. Widget 本体は Layer Shell / `Layer::Bottom` で表示する。
2. Context Menu、Calendar、Weekly Forecast、Edit Layout Panel は独立した Layer Shell / `Layer::Top` Surface とする。
3. Settings は通常の XDG Toplevel Window として `toodle-settings` に分離する。
4. Widget の表示状態は `Normal` と `Edit` の2状態を基本とする。
5. Edit Mode は Widget 本体の Surface を直接編集するのではなく、`Layer::Top` の独立 Edit Layout Panel と組み合わせて実現する。
6. Normal Mode では Widget 本体の通常 Surface を操作対象とする。
7. Edit Mode では Widget 本体の Input Region を一時的に全体へ拡張し、同時に Edit Layout Panel を表示する。
8. レイアウト変更は Anchor / Margin / Width / Height を基本とし、Font Scale も Edit Layout から変更できる。
9. レイアウト変更中は既存の Widget Surface を破棄・再生成せず、Layer Surface Command によるインプレース更新を優先する。
10. Context Menu、Calendar、Forecast、Edit Layout Panel は `xdg_popup` ではなく独立 Layer Surface とする。
11. 設定は `~/.config/toodle/config.toml` に保存する。
12. `toodle-settings` は設定変更を保存し、`toodle` は file watcher により変更を再読み込みする。
13. Weather API との境界は `WeatherProvider` とする。
14. API Key 管理は行わない。
15. 時刻は OS の Local Time を使用する。
16. Clock Tick は次の秒境界を基準に再同期する。
17. Widget は1インスタンスを基本とする。
18. 外部システムフォントには依存せず、使用フォントをバイナリへ組み込む。
19. 通常運用時は Settings GUI プロセスを常駐させず、Widget 本体を独立して動作させる。

---

# 2. Project Concept

Toodle は COSMIC Desktop 上に常駐するデジタル時計・天気 Widget である。

Widget は壁紙上に直接表示され、通常のアプリケーション Window より背面に配置される。

通常表示する情報：

- デジタル時計
- 秒
- 日付
- 曜日
- 現在の天気
- 現在の気温

追加 UI：

- Context Menu
- Monthly Calendar
- Weekly Forecast
- Edit Layout Panel

設定は Widget 本体とは分離した `toodle-settings` から行う。

---

# 3. Non-Goals

現在の設計では以下を対象外とする。

- `xdg_popup` の使用
- Normal Mode での自由ドラッグ
- GPS / Network Location による自動位置取得
- API Key 管理
- イベント
- TODO
- カレンダー同期
- リマインダー
- 複数 Widget インスタンス
- ユーザーによるカスタムフォントファイル読み込み
- 任意 RGB カラー
- ローカライズ
- 天気アラート
- Sunrise / Sunset
- Moon Phase
- 他 Desktop Environment への対応
- Windows / macOS 対応

Geocoding 自動解決、Systemd User Service、Arch パッケージング等は今後の拡張候補であり、現行実装の必須機能には含めない。

---

# 4. Process Architecture

Toodle は2つの独立バイナリで構成する。

```text
toodle
├── Desktop Widget
│   └── Layer Shell / Bottom
│
├── Context Menu
│   └── Layer Shell / Top
│
├── Calendar
│   └── Layer Shell / Top
│
├── Weekly Forecast
│   └── Layer Shell / Top
│
└── Edit Layout Panel
    └── Layer Shell / Top


toodle-settings
└── XDG Toplevel Window
```

## 4.1 `toodle`

責務：

- Widget 表示
- Clock
- Date / Weekday
- Current Weather
- Context Menu
- Calendar
- Weekly Forecast
- Edit Layout Panel
- Layer Surface の生成・破棄
- Widget Surface の Anchor / Margin / Size 更新
- Input Region の切り替え
- `config.toml` の監視と再読み込み
- Weather の非同期取得

## 4.2 `toodle-settings`

責務：

- Appearance
- Layout
- Weather
- Display
- Theme
- Color
- Font Scale
- Text Shadow
- Temperature Unit
- Configuration の保存

設定変更は `config.toml` を介して `toodle` へ伝播する。

---

# 5. Surface Architecture

現在の Surface 構成を以下に固定する。

```text
┌────────────────────────────────────────────────────────────┐
│ COSMIC Desktop                                              │
│                                                            │
│ Layer::Top                                                  │
│   ├── Context Menu                                          │
│   ├── Monthly Calendar                                      │
│   ├── Weekly Forecast                                       │
│   └── Edit Layout Panel                                     │
│                                                            │
│ Normal Application Windows                                  │
│   └── toodle-settings                                       │
│                                                            │
│ Layer::Bottom                                                │
│   └── Toodle Widget                                         │
│                                                            │
│ Wallpaper                                                    │
└────────────────────────────────────────────────────────────┘
```

## 5.1 Widget Surface

Widget は Layer Shell の Bottom Layer に配置する。

特徴：

- 壁紙上に表示する
- 通常の Application Window より背面に置く
- `exclusive_zone = 0`
- 背景は透明
- Anchor / Margin / Size により配置する

Widget の Surface は通常運用中は常駐する。

## 5.2 Top Surfaces

Context Menu、Calendar、Weekly Forecast、Edit Layout Panel は Widget とは独立した Layer::Top Surface として生成する。

これらは通常時には存在せず、必要になったときだけ生成する。

### Context Menu

```text
340 x 380
```

### Monthly Calendar

```text
680 x 720
```

### Weekly Forecast

```text
680 x 720
```

### Edit Layout Panel

現在の実装では独立した Layer::Top Surface とし、Widget の編集操作を行う。

---

# 6. Widget State

Widget の基本状態は以下の2つとする。

```text
Normal
Edit
```

## 6.1 Normal

通常表示状態。

```text
Normal
 ├── Clock
 ├── Date / Weekday
 └── Weather
```

Normal 状態では Widget Surface が通常の表示と入力を担当する。

Right Click により Context Menu を開く。

## 6.2 Edit

Edit Layout Panel が表示され、Widget のレイアウトを調整する状態。

遷移：

```text
Normal
   │
   │ Right Click
   ▼
Context Menu
   │
   │ Edit Layout
   ▼
Edit
   │
   ├── Edit Layout Panel を生成
   └── Widget Input Region を全体へ変更
```

Edit 終了時：

```text
Edit
 ├── Save
 └── Cancel / Escape
        ↓
     Normal
```

Save では編集内容を `config.toml` に保存する。

Cancel では現在の編集内容を破棄し、保存済み設定へ戻す。

---

# 7. Edit Layout Architecture

v0.2 の「Widget Surface 自体を Edit Mode の専用 UI とする」方式から変更し、現在は **Widget Surface と Edit Layout Panel の2 Surface 方式**を正式仕様とする。

```text
Layer::Bottom
└── Widget Surface
    └── 編集対象となる実際の Widget

Layer::Top
└── Edit Layout Panel
    └── Margin / Size / Anchor / Font Scale 操作
```

## 7.1 Edit Layout Panel

Edit Layout Panel は独立した Layer::Top Surface とする。

役割：

- Anchor の変更
- Margin X の変更
- Margin Y の変更
- Width の変更
- Height の変更
- Font Scale の変更
- Save
- Cancel

## 7.2 Widget Surface の更新

Edit 中の Slider / Anchor 操作では、既存 Widget Surface に対して Layer Surface Command を適用する。

主な操作：

```text
set_anchor
set_margin
set_size
set_input_zone
```

Surface を破棄して再生成する方式は採用しない。

これは Phase 0 の実機検証で確認されたフリッカー対策を正式な設計原則とするためである。

## 7.3 Save

Save 時は、

1. Edit State の値を Config へ反映
2. `config.toml` を保存
3. Edit Layout Panel を破棄
4. Widget を Normal State へ戻す
5. Widget の Input Region を通常状態へ戻す

という順序で処理する。

既に編集値に合わせて Widget Surface が更新済みであるため、Save 時に不要な Surface 再配置を行わない。

## 7.4 Cancel

Cancel / Escape 時は、

1. Edit Layout Panel を破棄
2. 編集前の Config を基準に Widget Surface を復元
3. Widget を Normal State へ戻す
4. Normal Mode の Input Region を復元

する。

---

# 8. Input Region

Input Region は Widget Surface の状態に応じて切り替える。

## 8.1 Normal

現在の実装では `content_bounds()` により Input Zone を設定する。

この値は現行実装では Widget の Layout Width / Height を基準とした矩形である。

```text
Input Region
┌─────────────────────────┐
│                         │
│      Toodle Widget      │
│                         │
└─────────────────────────┘
```

したがって、v0.3 では「文字の実描画形状に沿った非矩形 Hit Area」を仕様として要求しない。

> 旧 v0.2 にあった「文字の実描画領域だけを Input Region にする」という記述は、現在の実装仕様から削除する。

## 8.2 Edit

Edit 開始時には Widget Surface の Input Region を `None` とし、Surface 全体を入力対象にする。

```text
Edit:
Input Region = entire Widget Surface
```

Edit 終了時に Normal 用の `content_bounds()` へ戻す。

## 8.3 Design Intent

Input Region の制御自体は引き続き重要な設計要素とする。

ただし v0.3 では、実装済みの矩形 Input Zone を正とし、将来的により細かい Hit Area が必要になった場合は別途設計変更として扱う。

---

# 9. Layout

Widget の配置は Anchor + Margin 方式で管理する。

## 9.1 Anchor

現在使用する Anchor：

```text
TopLeft
TopRight
BottomLeft
BottomRight
```

## 9.2 Position

配置：

```text
Anchor
Margin X
Margin Y
```

## 9.3 Size

サイズ：

```text
Width
Height
```

## 9.4 Font Scale

Appearance の `font_scale` を Widget 表示へ反映する。

Edit Layout Panel からも Font Scale を変更できる。

## 9.5 Safety Limits

現在の設定 UI では、設定値が画面外へ大きく飛び出すことを防ぐため、Layout 値にクランプを設ける。

現在の実装上の範囲：

```text
margin_x : 0 ..= 2560
margin_y : 0 ..= 1440
width    : 150 ..= 1200
height   : 60 ..= 800
```

この範囲は固定仕様というより、現在の安全域として扱う。

---

# 10. Clock

## 10.1 Display

24時間制。

秒を常に表示する。

例：

```text
14:32:18
```

日付と曜日を表示する。

例：

```text
Saturday, September 20, 2026
```

曜日・日付表記は English (US) に固定する。

## 10.2 Time Source

OS Local Time を使用する。

現在の実装では Rust の `chrono` を使用する。

## 10.3 Tick

Clock 更新は次の秒境界を基準に再同期する。

```text
14:32:18.xxx
       ↓
14:32:19.xxx
```

固定1秒間隔の単純な周期タイマーではなく、各 Tick の時点から次の秒境界までの待機時間を計算する。

これにより長時間動作時の単純な Timer Drift を抑える。

なお、OS Scheduler、Tokio Timer、Compositor のスケジューリング遅延までゼロにするものではない。

---

# 11. Fonts

フォントは `resources/fonts/` に保持し、`include_bytes!` 等で Binary に組み込む。

現在の Font Family：

```text
Roboto
JetBrains Mono
DejaVu Serif
Open Sans
```

各 Family について Regular / Bold を使用する。

合計：

```text
4 families
8 font files
```

外部システムフォントへの依存は行わない。

目的：

- オフライン動作
- 環境による Font Metric 差の削減
- 配布時の表示差の削減

---

# 12. Theme

Theme は固定 Preset 方式とする。

現在は10種類：

```text
Modern
Classic
Digital Mono
Minimal
Cyberpunk
Nord
Warm Sunset
Forest
Slate
Rose Gold
```

Theme は Font Family と推奨 Color を持つ。

例：

```text
Modern
  → Roboto
  → #FFFFFF

Digital Mono
  → JetBrains Mono
  → #38BDF8

Classic
  → DejaVu Serif
  → #E2E8F0
```

Color は固定 Palette から選択する。

現在の Palette は16色。

任意 RGB 入力は行わない。

Text Shadow は ON / OFF を切り替え可能とする。

---

# 13. Weather Architecture

Weather は以下の責務分離で構成する。

```text
Widget
  ↓
WeatherService
  ├── WeatherCache
  └── WeatherProvider
          ↓
      Open-Meteo API
```

## 13.1 WeatherProvider

外部 Weather API との境界を `WeatherProvider` Trait とする。

現在の Provider：

```text
OpenMeteoProvider
```

API Key は使用しない。

## 13.2 WeatherService

WeatherService の責務：

- Cache の確認
- Provider 呼び出し
- Cache 保存
- stale cache fallback
- Location 変更時の再取得
- Widget への結果返却

Network 処理は非同期で実行する。

---

# 14. Weather Cache

Weather データは永続 JSON Cache として保存する。

`CachedWeather` には少なくとも以下の情報を保持する。

```text
Weather Data
Latitude
Longitude
Fetched At
```

## 14.1 Location Matching

Cache は緯度・経度を用いて現在の設定 Location と照合する。

現在の実装では概ね：

```text
latitude difference  < 0.02
longitude difference < 0.02
```

を一致条件とする。

異なる Location の Cache をそのまま返さない。

## 14.2 Current / Forecast Cache Policy

設計上の Cache TTL は以下とする。

```text
Current Weather
  30 minutes

Forecast
  3 hours
```

ただし、現行実装では Weather Service の取得単位が完全に分離されていないため、Current / Forecast の TTL を独立して利用する部分には実装上の残課題がある。

この差異は v0.3 では既知の Implementation Gap として管理し、過去の「完全実装済み」という記述は行わない。

## 14.3 Offline / Error

Network Request が失敗した場合：

```text
Valid / stale cache exists
    ↓
use cached weather

No cache
    ↓
Weather unavailable
```

Location が変更された場合は、古い Location の Cache を再利用しない。

---

# 15. Weather Network Control

Network 処理は Background / Async Task で実行し、UI Thread を同期 HTTP Request でブロックしない。

現在の Provider では HTTP エラーを Weather Error として返し、利用可能な Cache があれば fallback する。

> v0.2 の設計に記載されていた 429 / 5xx Exponential Backoff は、現行実装ではまだ完全には実装されていない。
>
> したがって v0.3 ではこれを「現在の必須実装」ではなく Known Gap として扱う。

将来的な Retry 方針：

```text
1
2
4
8
...
max 30 minutes
```

---

# 16. Current Weather

Widget の通常表示では以下を表示する。

- Weather condition
- Temperature

Temperature Unit：

```text
Celsius
Fahrenheit
```

Weather が Location 変更などで再取得中の場合、現在の UI ではローディング状態として `...` を使用する。

Network Error 時は Cache が利用可能なら stale data を継続表示する。

---

# 17. Weekly Forecast

Weekly Forecast は Layer::Top Surface として表示する。

現在の UI は7日間予報を表示する。

各日の情報：

- Date
- Weather condition
- Minimum temperature
- Maximum temperature
- Precipitation probability
- Weather icon

Popup Size：

```text
680 x 720
```

なお、現在の Provider / UI 実装では「API レベルで必ず7日だけを取得する」保証を独立仕様としては置かず、UI が7日間予報として扱う。

---

# 18. Monthly Calendar

Monthly Calendar は Layer::Top Surface として表示する。

Popup Size：

```text
680 x 720
```

機能：

- 月間カレンダー表示
- 前月
- 翌月
- 今日への移動
- 今日の日付ハイライト

対象外：

- Events
- TODO
- Reminder
- External Calendar Sync

---

# 19. Context Menu

Right Click により Context Menu を開く。

現在の基本項目：

```text
Calendar
Weekly Forecast
Edit Layout
Settings
Quit
```

Popup Size：

```text
340 x 380
```

UI はソリッド背景を使用する。

Context Menu から：

```text
Edit Layout
```

を選択すると Edit Layout Panel を生成して Edit State へ遷移する。

---

# 20. Configuration

設定は：

```text
~/.config/toodle/config.toml
```

に保存する。

現在の構成：

```toml
[display]
output = ""

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
location_name = "Tokyo, Japan"
latitude = 35.6895
longitude = 139.6917
temperature_unit = "Celsius"
```

`output` は Display 設定として保持する。

API Key は持たない。

## 20.1 Atomic Save

Settings からの保存は、一時ファイルへ書き出してから Rename する方式を使用する。

概念：

```text
config.toml.tmp
      ↓
    rename
      ↓
config.toml
```

---

# 21. Configuration Reload

設定変更の基本経路：

```text
toodle-settings
       ↓
 config.toml
       ↓
 file watcher
       ↓
    toodle
       ↓
 ConfigReloaded
       ↓
 surface / state / weather update
```

`notify::recommended_watcher` を使用する。

`toodle` は設定ファイル変更を受けると Config を再読み込みする。

Layout / Appearance の変更は可能な範囲で既存 Widget Surface に対する Layer Command で反映する。

Location が変更された場合は現在の Weather 表示をリセットし、新しい座標で Weather Request を開始する。

## 21.1 変更イベントの扱い

Settings は頻繁な操作時にも即座に設定を保存する。

Watcher 側では現在、変更検知後に短い待機を挟んで再読み込みする。

現在の実装ではこの待機は厳密な debounce 実装ではないため、v0.3 では「25ms debounce」という表現を仕様として固定しない。

## 21.2 SIGUSR1

SIGUSR1 は現在の主要な設定同期方式ではない。

必要になった場合の将来拡張として扱う。

---

# 22. Display / Multi-Monitor

Widget は1インスタンスのみとする。

Configuration には Display Output 設定を持つ。

```toml
[display]
output = "DP-1"
```

ただし、**現行実装では `DisplayConfig::output` の値が Layer Surface の `IcedOutput` にまだ接続されておらず、現在は `IcedOutput::Active` が使用されている。**

したがって v0.3 では：

```text
Design:
  Output selection is a supported configuration concept.

Implementation:
  Currently Active Output.

Status:
  Known Gap.
```

と明示する。

複数 Display へ同時に Widget を表示することは行わない。

将来の拡張候補：

- Display 接続 / 切断への追従
- Output 名による動的 Surface Migration

---

# 23. UI / Window Styling

Widget 本体：

```text
transparent
```

Context Menu / Calendar / Forecast / Edit Layout Panel / Settings：

```text
solid dark background
```

現在の Settings / Popup UI は、透明背景による可読性問題を避けるためソリッドダーク背景を使用する。

Settings Window：

```text
720 x 780
```

Settings のカテゴリ：

```text
Appearance
Layout
Weather
Display
```

---

# 24. Phase 0 Real-World Verification Decisions

Phase 0 の実機検証では、当初の v0.2 設計から複数の重要な変更が発生した。

これらは v0.3 で正式な設計判断として扱う。

## 24.1 `xdg_popup` を採用しない

Popup を `xdg_popup` として Widget に従属させる方式ではなく、独立 Layer::Top Surface とする。

対象：

- Context Menu
- Calendar
- Weekly Forecast
- Edit Layout Panel

## 24.2 Edit Layout は独立 Surface とする

Edit Mode の UI を Widget Surface 内に埋め込む方式ではなく、

```text
Bottom Widget
+
Top Edit Layout Panel
```

とする。

これにより、Widget の表示 Surface と編集操作 UI を分離する。

## 24.3 Surface 再生成を避ける

Phase 0 の実機検証で確認されたフリッカーを避けるため、レイアウト変更時には既存 Surface を再生成せず、Layer Surface Command によるインプレース更新を優先する。

## 24.4 Popup / Settings の背景をソリッド化

透明 UI は実際のデスクトップ背景によって可読性が変化するため、Context Menu、Calendar、Forecast、Edit Layout Panel、Settings はソリッドダーク背景を使用する。

## 24.5 Layout 値を安全域へ制限

実機操作で画面外へ Widget が飛び出す問題を避けるため、Settings 側の Margin / Size にクランプを設ける。

---

# 25. Known Implementation Gaps

v0.3 は「現在の実装と設計を同期する」ための文書であり、未実装部分を実装済みとして扱わない。

現在確認されている主な Gap：

## 25.1 Display Output

`DisplayConfig::output` は設定として存在するが、現在の Layer Surface は `IcedOutput::Active` を使用している。

## 25.2 Independent Weather TTL

Current 30min / Forecast 3h の設計値は存在するが、Weather Service の実際の取得・Cache 判定は完全には分離されていない。

## 25.3 Exponential Backoff

429 / 5xx に対する Exponential Backoff は設計上の候補として残っているが、現在の実装で完全には成立していない。

## 25.4 Geocoding

現在は都市プリセットまたは緯度・経度入力を基本とする。

Location Name からの自動 Geocoding は未実装。

## 25.5 Request Generation / Stale Result Protection

Location 変更時の非同期 Weather Request に対して、古い Request の結果を新しい Location の表示へ適用しないための明示的な Request ID / Generation 管理は今後の堅牢化候補とする。

## 25.6 Popup Dismissal

Popup の focus loss / outside click による自動閉鎖は、実機挙動を確認したうえで実装仕様を確定する。

---

# 26. Project Structure

現在の基本構成：

```text
Toodle/
├── Cargo.toml
├── Cargo.lock
├── README.md
├── README.ja.md
├── Toodle-Design-Docs.md
├── SESSION_HANDOVER.md
├── docs/
│   ├── ARCHITECTURE.md
│   ├── ARCHITECTURE.ja.md
│   ├── FEATURES.md
│   └── FEATURES.ja.md
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── widget/
│   ├── popup/
│   ├── clock/
│   ├── weather/
│   ├── config/
│   └── settings/
└── resources/
    └── fonts/
```

主要責務：

```text
src/main.rs
    Application / Surface lifecycle / Message routing

src/widget/
    Widget rendering / Input bounds / Edit state

src/popup/
    Context Menu / Calendar / Forecast

src/clock/
    Clock Tick / Font loading

src/weather/
    Provider / Cache / Service

src/config/
    Config / Theme / Persistence / Watcher

src/settings/
    toodle-settings UI
```

---

# 27. Verification Strategy

実装変更時は、以下を基本確認とする。

## 27.1 Build

```bash
cargo check --bin toodle
cargo check --bin toodle-settings
cargo build --release
```

## 27.2 Unit Tests

```bash
cargo test
cargo test --lib weather
```

## 27.3 COSMIC 実機確認

コードだけでは判定できない項目については、COSMIC / Wayland 実機で確認する。

最低限：

```text
[ ] Widget Layer::Bottom 表示
[ ] 通常 Window との前後関係
[ ] Context Menu Layer::Top
[ ] Calendar Layer::Top
[ ] Forecast Layer::Top
[ ] Edit Layout Panel Layer::Top
[ ] Edit 中の Input Region
[ ] Edit Save / Cancel
[ ] Layout 更新時の Flicker
[ ] Settings → config.toml → Widget reload
[ ] Weather Location 変更
[ ] Popup の dismissal
[ ] Display Output の現在の挙動
```

---

# 28. Implementation Roadmap

v0.3 の今後の作業は「旧設計への回帰」ではなく、現在のアーキテクチャを維持したまま不足部分を完成させる。

## Phase 5 — Robustness

優先候補：

1. Display Output の実装接続
2. Popup dismissal の実機確認・実装
3. Weather Current / Forecast Cache の独立化
4. Weather 429 / 5xx Backoff
5. Weather Request Generation 管理
6. Geocoding

## Phase 6 — Distribution

候補：

1. Systemd User Service
2. Desktop Entry
3. Icon Assets
4. Arch Linux / AUR packaging

---

# 29. Design Decision Summary

Toodle の現在の Surface Architecture は以下である。

```text
Widget
    = Layer Shell / Bottom

Context Menu
    = Layer Shell / Top

Calendar
    = Layer Shell / Top

Weekly Forecast
    = Layer Shell / Top

Edit Layout Panel
    = Layer Shell / Top

Settings
    = XDG Toplevel
```

Widget の状態は：

```text
Normal
Edit
```

Edit Mode は Widget Surface の置換ではなく、

```text
Bottom Widget
+
Top Edit Layout Panel
```

で構成する。

Layout 更新は既存 Surface に対する in-place Layer Command を優先する。

Configuration は：

```text
toodle-settings
      ↓
config.toml
      ↓
file watcher
      ↓
toodle
```

で同期する。

Weather は：

```text
WeatherService
      ↓
WeatherProvider
      ↓
Open-Meteo
```

とし、Cache と stale fallback を備える。

Clock は OS Local Time を基準にし、次の秒境界へ再同期する。

Font は Binary に組み込む。

---

# 30. Version History

## v0.2

初期の決定稿。

Prototype 0 から Phase 4 までの基本設計を定義。

## v0.3

Phase 0 の実機検証および Phase 1〜4 の実装結果を反映。

主な変更：

- `xdg_popup` を正式に不採用
- Popup を独立 Layer::Top Surface 化
- Edit Layout を独立 Layer::Top Surface 化
- Widget Surface と Edit Layout Panel を分離
- Surface 再生成ではなく in-place 更新を正式化
- ソリッド UI 背景を正式化
- Input Region の仕様を現行実装に合わせて再定義
- 旧設計の未実装機能を Known Gap として明示
- Display Output / Weather Backoff / Geocoding 等の実装状況を設計書上で明示
- 「実装済み」と「設計上の将来項目」を分離

---

# Appendix A. v0.2 から v0.3 への主要設計変更

| 項目 | v0.2 | v0.3 |
|---|---|---|
| Popup | Layer::Top Popup | 独立 Layer::Top Surface |
| `xdg_popup` | Non-Goal | 引き続き不採用 |
| Edit Mode | Widget Surface 内の編集状態を想定 | Top Edit Layout Panel + Bottom Widget |
| Resize | Resize Handle を想定 | Edit Panel の Size 操作 |
| Layout 更新 | Edit Mode の操作 | 既存 Surface への in-place update |
| Input Region Normal | content bounds | 現行 `content_bounds()` の矩形 |
| Input Region Edit | Widget 全体 | Widget 全体 |
| Popup 背景 | 透明を想定可能 | Solid Dark |
| Settings | XDG Toplevel | XDG Toplevel |
| Config reload | File watcher | File watcher |
| Display Output | 設計上サポート | 設定項目は存在、実装接続は Gap |
| Weather TTL | Current 30m / Forecast 3h | 設計値として維持、実装分離は Gap |
| Backoff | 必須設計 | Known Gap |
| Geocoding | 設計項目 | Known Gap |

---

# Appendix B. Documentation Rule

今後の実装変更では、以下の順序で文書を更新する。

1. 実機で成立した挙動を確認する。
2. アーキテクチャ上の変更なら本 Design Document を更新する。
3. 実装上の不足なら Known Implementation Gaps を更新する。
4. `SESSION_HANDOVER.md` に変更理由と作業履歴を記録する。
5. `README.md` / `README.ja.md` / `docs/ARCHITECTURE.*` の公開説明も同期する。

特に、**「設計とコードが違う」ことだけを理由にコードを旧設計へ戻さない。**

実機検証によって確定した挙動を新しい設計の基準とし、必要な場合のみ設計変更として文書化する。
