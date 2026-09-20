
# COSMIC Desktop Clock Widget

## 仕様書・設計書 v0.2

**Status:** Architecture Refined / Ready for Prototype 0

**Target:** COSMIC Desktop

**Platform:** Linux / Wayland

**Language:** Rust

**UI Framework:** libcosmic

---

# 1. プロジェクト概要

COSMIC Desktop専用の、デスクトップ上に常駐する軽量な時計・天気ウィジェット（`cosmic-clock`）および設定管理ツール（`cosmic-clock-settings`）を開発する。

壁紙を活かしたミニマルなデザインを基本とし、ウィジェット本体は完全透明な背景を持つ。通常時は最小限の情報（時計・秒・日付・天気）を表示し、カレンダー・週間天気予報などの詳細機能は右クリックメニューから独立した上位レイヤーのPopupとして呼び出す。

---

# 2. 基本コンセプト

> **A lightweight, transparent desktop clock for COSMIC.**

「デスクトップの壁紙の一部として溶け込む時計」を最優先とする。
通常のアプリケーションウィンドウではなく、デスクトップの背景直上に常駐するLayer Shellウィジェットとして動作する。

---

# 3. 対象環境・サポート範囲

## 3.1 対象OS / Desktop

* Linux
* COSMIC Desktop (Wayland環境)

## 3.2 サポート範囲

* COSMIC Desktop専用（移植性は考慮せず、COSMICおよびWayland Layer Shell固有の機能を前提とする）
* 対象外: GNOME, KDE Plasma, X11, Sway, Hyprland 等

---

# 4. プロセスモデルとシステム構成

ウィジェット本体と設定画面は**別バイナリ**として分離し、プロセスレベルで責務を切り離す。

```text
cosmic-clock (Widget常駐プロセス)
    │
    ├── Layer Shell (Bottom): 時計・天気ウィジェット本体
    │
    └── Layer Shell (Top / Overlay Popup):
            ├── Context Menu
            ├── Calendar Popup
            └── Weekly Forecast Popup

cosmic-clock-settings (設定GUIバイナリ)
    └── 通常のXDG Toplevel Window (libcosmic)

```

## 4.1 プロセス間通信 / 状態同期

* 設定の変更はファイル（`config.toml` 等）へ即時永続化される。
* ウィジェット本体側は、設定ファイル変更通知（`file watcher` / `inotify`）またはIPCシグナル（SIGUSR1 等）を受信して、ホットリロードで変更を反映する。
* ウィジェット本体の常駐メモリとフットプリントを最小化し、設定UIを開いていない平時のリソース消費をゼロに抑える。

## 4.2 ディスプレイ配置仕様

* **複数ディスプレイ環境でも起動するインスタンスは「1つ」のみ。**
* 起動対象となるディスプレイ（Output）は設定から選択可能（例: Primary / Display名指定）。

---

# 5. Surface & Window Architecture

Waylandのレイヤー制約と入力透過性を担保するため、以下の構成をとる。

```text
Overlay Layer
    └── (Popup) Context Menu, Calendar, Weekly Forecast [必要時のみ最前面・フォーカス取得]
Top Layer
Panel Layer
──────────────────────────────
Normal Application Windows (ブラウザ、エディタ等)
──────────────────────────────
Bottom Layer
    └── Desktop Widget 本体 (完全透明・文字/ハンドル部のみInput Region定義)
──────────────────────────────
Background Layer (壁紙)

```

## 5.1 Desktop Widget 本体 (`Layer::Bottom`)

* 壁紙の直上、通常アプリケーションウィンドウの直下に常駐。
* ワークスペースの切り替えに影響されず、デスクトップ上に固定される。

## 5.2 Context Menu / Calendar / Forecast Popups (`Layer::Top` または `Overlay`)

* ウィジェット本体（`Layer::Bottom`）に直接xdg_popupをぶら下げると、通常ウィンドウの下に隠れてしまうかフォーカス取得で問題が発生するため、**Popup類は独立して `Layer::Top`（または `Overlay`）として生成**する。
* マウスフォーカスが外れた場合（auto-dismiss）、即座に破棄される。

## 5.3 Settings Window

* `cosmic-clock-settings` により起動される、COSMIC標準の通常ウィンドウ。

---

# 6. 入力領域制御（Input Region Management）

透明背景上での操作性を損なわないため、Waylandの入力領域を厳格に制御する。

* **通常モード時**:
* 時計の文字やアイコンなど、**コンテンツが実際に描画されている領域のみ**を入力領域（`wl_surface.set_input_region`）としてCompositorへ通知する。
* コンテンツ周囲の透明領域はInput Pass-through（完全透過）とし、デスクトップ壁紙のクリック・ドラッグ、デスクトップアイコンの選択等を一切阻害しない。


* **右クリック**:
* コンテンツ描画領域を右クリックすることで、コンテキストメニューを展開する。



---

# 7. 配置・リサイズ仕様（Edit Mode）

Layer Shellのプロトコル特性に合わせ、自由なドラッグ移動を廃止し、「Edit Mode（編集モード）」を導入する。

```text
通常表示 (Lock)
    │
    │ (右クリック → "Edit Layout" を選択)
    ▼
Edit Mode (編集・配置モード)
    │
    ├── 全体に入力バウンディングボックスと枠線を表示
    ├── 4隅のいずれかへのアンカー固定 + オフセット(Margin X/Y)のドラッグ調整
    └── リサイズハンドルのドラッグによるサイズ変更
    │
    │ ("Done" ボタン / Escキー / 枠外クリック)
    ▼
通常表示 (新配置・サイズを保存してInput Regionを文字領域のみに再縮小)

```

### メリット

* 毎フレームの連続的なプロトコル再ネゴシエーションによる描画の乱れ・Compositor負荷を防止。
* 平常時は文字領域のみに入力判定を絞り、配置変更時のみ明確にSurface全体の入力操作を有効化できる。

---

# 8. 時計仕様

## 8.1 表示形式

* **常に秒を表示**する形式とする。

```text
14:32:18

```

* 24時間制固定。
* 日付・曜日表示例：

```text
Sep 20, 2026
Sunday

```

* 表示言語はEnglish (US)固定。

## 8.2 Time Source & 更新スケジューリング

* OSのローカルシステム時刻（`std::time` / `chrono`）を使用。
* **更新アライメント**:
* 次のミリ秒境界（`1.000秒` の頭）に合わせて正確にTickを発火させるスケジューラを実装し、秒針の遅れや描画スキップを防ぐ。



---

# 9. フォント・リソース管理

フォント未導入環境での表示崩れやフォールバックによるレイアウト破綻を防ぐため、**すべてのThemeフォントをバイナリに直接埋め込む（`include_bytes!`）**。

* システムの外部フォントへの依存をゼロにする。
* 完全オフライン環境でも同一のデザイン・メトリクスを保証する。

---

# 10. 天気・Geocoding アーキテクチャ

UI層からネットワークアクセスを完全に分離し、API制限や障害に対するフォールトトレランスを持たせる。

```text
UI (Widget / Forecast Popup)
 │
 ▼
Weather Service (Cache & Worker)
 ├── In-memory Cache (Current: 30分, Forecast: 3時間)
 ├── Rate Limiter & Exponential Backoff
 └── Validation Layer
      ├── Geocoding Provider (Open-Meteo / Nominatim等)
      └── Weather Provider (Open-Meteo 等)

```

## 10.1 レート制限（Rate Limit）とAPI保護

1. **地名バリデーション & デバウンス**:
* 設定画面での地名入力時、キー入力ごとの即時問い合わせを行わず、入力停止後（500ms〜800ms）にクエリを実行。
* 入力文字列のトリム・文字数チェックを行い、無効なリクエスト送信を防止。


2. **キャッシュ戦略**:
* 一度成功したGeocoding結果（地名 → 緯度・経度）はローカルに永続化し、地名が変更されない限りAPIを再呼び出ししない。
* 天気データは設定されたインターバル（現在天気: 30分、予報: 3時間）厳守でキャッシュを再利用。


3. **リトライバックオフ（Exponential Backoff）**:
* HTTP 429（Too Many Requests）や 5xx エラー受信時は、指数関数的バックオフ（1分、2分、4分、最大30分）を設けてリトライを制御。
* 制限超過時やオフライン時は直近のキャッシュを優先表示し、キャッシュがない場合は `Weather unavailable` を静かに表示する（時計等の他機能には一切影響を与えない）。


4. **APIキー拡張性**:
* ProviderごとにカスタムAPIキーを設定可能な抽象インターフェース（`trait WeatherProvider`）を定義。



---

# 11. Theme & Appearance

* **10種類のテーマプリセット**（埋め込みフォント・文字間隔・レイアウト定義）
* **16色カラーパレット**:
* 壁紙に対する視認性を確保するため、ユーザーが16色から選択。


* **Text Shadow**:
* 明暗が混在する壁紙上で視認性を確保するための補助ドロップシャドウ（ON/OFF可能）。



---

# 12. 設定・永続化項目 (`config.toml`)

設定ファイル（XDG Base Directory準拠: `~/.config/cosmic-clock/config.toml`）に以下を保持する。

```toml
[display]
output = "DP-1"            # 表示先ディスプレイ

[layout]
anchor = "TopRight"        # TopLeft, TopRight, BottomLeft, BottomRight
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
temperature_unit = "Celsius" # Celsius | Fahrenheit
api_key = ""                 # 任意

```

---

# 13. 実装フェーズ（Roadmap）

## Phase 0: Technology Prototype（最重要）

* [ ] Rust + `libcosmic` で Layer Shell の起動
* [ ] `Layer::Bottom` で壁紙上・通常ウィンドウ下に配置
* [ ] ウィジェット背景の完全透過
* [ ] **`set_input_region` による文字領域のみの入力受付と、余白の完全クリック透過（Pass-through）の検証**
* [ ] **Edit Mode によるサイズ変更・マージン移動の基本検証**
* [ ] **`Layer::Top` / `Overlay` による独立Popup生成とフォーカス・dismissの挙動確認**
* [ ] 指定ディスプレイ（Output）へのSurfaceアタッチ検証

## Phase 1: Core Engine & Clock

* [ ] 埋め込みフォントローダーの実装
* [ ] ミリ秒境界アライメントによる常時秒表示Tickスケジューラ
* [ ] 日付・曜日の描画

## Phase 2: Weather & Resilience

* [ ] `WeatherProvider` / `GeocodingProvider` トレイト定義
* [ ] レート制限、指数バックオフ、ローカルキャッシュ層の実装
* [ ] 地名バリデーション
* [ ] オフライン・エラー時のフォールバック表示

## Phase 3: Popups & UI

* [ ] 右クリックコンテキストメニュー（`Layer::Top`）
* [ ] 月間カレンダーPopup
* [ ] 7日間週間天気予報Popup

## Phase 4: Settings Binary & IPC

* [ ] `cosmic-clock-settings` バイナリの実装
* [ ] ディスプレイ選択、外観（Theme / 16色 / Shadow）、地点設定UI
* [ ] 設定ファイル経由のホットリロード機構

---

# 14. Prototype 0 の合格基準

1. 壁紙上に文字が表示され、通常ウィンドウの下に潜ること。
2. 文字以外の透明部分をクリックした際、下のデスクトップや壁紙の操作が100%透過して機能すること。
3. 文字をクリックして編集モードに入り、配置（マージン）・サイズを変更して確定できること。
4. 右クリックでテストPopup（`Layer::Top`）が通常ウィンドウの前面に問題なく開き、外側クリックで閉じること。