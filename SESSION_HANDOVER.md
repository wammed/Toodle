# Toodle 開発引き継ぎサマリー (Session Handover & Continuation Guide)

本ドキュメントは、これまでの開発・改修内容の全履歴、技術的決定事項、アーキテクチャの変更点、および今後の開発再開時にスムーズに作業を継続できるようにまとめた包括的な引き継ぎ資料です。

---

## 1. プロジェクト概要 & 技術構成

- **アプリケーション名**: Toodle (COSMIC Desktop Clock Widget)
- **リポジトリパス**: `/home/susie/GitHUB/wammed/Toodle`
- **設計ドキュメント**: [`Toodle-Design-Docs.md`](file:///home/susie/GitHUB/wammed/Toodle/Toodle-Design-Docs.md) (v0.2 決定稿)
- **開発言語**: Rust (2021 edition)
- **主要バイナリ**:
  - `toodle`: デスクトップクロックウィジェット本体 (`wlr-layer-shell` `Layer::Bottom`) および Layer Popups (`Layer::Top` コンテキストメニュー、カレンダー、7日間週間予報、Edit Layout モード)
  - `toodle-settings`: XDG Toplevel ネイティブ設定ウィンドウ (Appearance, Layout, Weather, Display の4タブ構成)
- **主要フレームワーク・ライブラリ**:
  - GUI ツールキット: `libcosmic` (v0.1 / git: `pop-os/libcosmic`, `wayland` feature)
  - ウィンドウ管理: `cosmic::iced::wayland::layer_surface` (`Layer::Bottom`, `Layer::Top`)
  - 設定ファイル監視: `notify` (inotify による `config.toml` 非同期ホットリロード)
  - 天気 API / キャッシュ: `reqwest`, `tokio`, `serde`, `serde_json`, `chrono` (Open-Meteo REST API, 2段階ローカル永続キャッシュ)
  - 組み込みフォント: `Roboto`, `JetBrains Mono`, `DejaVu Serif`, `Open Sans` (`include_bytes!` による完全スタンドアロンバイナリ、外部フォント非依存)

---

## 2. これまでのユーザー要望と対応履歴（時系列）

1. **基本設計確定とフェーズ分割計画の策定**:
   - `Toodle-Design-Docs.md` (v0.2) に基づき、Prototype 0 (Phase 0: サーフェス基盤) から Phase 4 (設定アプリ) までのステップバイステップ開発計画を合意。
2. **Phase 0 — Wayland Layer Shell サーフェス基盤の実装**:
   - `cosmic-comp` (COSMIC Desktop Environment) 上で動作するデスクトップウィジェットを作成。
   - `Layer::Bottom` による最背面配置、非矩形入力領域（`set_input_zone`）によるクリック透過（透明領域のクリックをデスクトップ背景へパススルー）。
   - 右クリックによる Edit Layout Mode（レイアウト編集モード）の起動。
3. **ユーザー指摘: チラつき、スライダー追従遅延、サイズ上限、日本語表記排除、右クリックメニュー崩れ**:
   - スライダー操作時の再生成によるチラつきを解消。
   - スライダーのフォントサイズ上限・ウィンドウサイズ上限を拡大。
   - Edit Layout Mode 内および右クリックコンテキストメニューから日本語表記を全廃（英語ネイティブUI化）。
   - 右クリックコンテキストメニューの崩れを修正し、背景透過を廃止してソリッド背景を適用。
4. **ユーザー指摘: Edit Layout Mode 終了時のチラつき解消**:
   - `EditLayoutSave` および `EditLayoutCancel` 時にサーフェスを破棄・再生成せず、`layer_cmd::set_size`、`layer_cmd::set_margin`、`layer_cmd::set_anchor` による同一サーフェスのインプレースプロパティ更新へ移行し、画面のチラつきをゼロ化。
   - **Phase 0 完了の承認を受領**。
5. **Phase 1 (Clock) & Phase 2 (Weather) の実装**:
   - **Phase 1 (Clock)**: 次の秒の境界（サブセカンドアライメント）に正確に同調する `clock_tick_stream` の実装。組み込みフォント（Roboto, JetBrains Mono）の導入。
   - **Phase 2 (Weather)**: `WeatherProvider` トレイト境界による Open-Meteo プロバイダ実装。30分（現在天気）および3時間（週間予報）のローカル JSON 2段階キャッシュ機構、オフライン時の stale キャッシュ自動フォールバック。
6. **Phase 3 — ポップアップ機能群の実装とサイズ決定**:
   - ユーザー指定サイズに基づきポップアップサーフェス（`Layer::Top`）を実装:
     - `ContextMenu`: 340 x 380
     - `Calendar`: 680 x 720
     - `Forecast`: 680 x 720
   - 月送りナビゲーションと「今日」ハイライトを備えた月間カレンダー。
   - 7日間の天気アイコン、気温幅（最低/最高）、降水確率、天候テキストを表示する週間予報。
7. **ユーザー指摘: コンテキストメニューのフォントバランス改善**:
   - 340 x 380 のウィンドウサイズに合わせ、ボタンフォントサイズを 18px に拡大し、コンテンツを中央揃えに配置して視覚的バランスを最適化。
8. **Phase 4 — 専用設定アプリ (`toodle-settings`) の構築**:
   - XDG Toplevel ウィンドウ（720 x 780）として独立起動可能な設定アプリを開発。
   - 10種類のテーマプリセット、16色の固定カラーパレット（Hex見本スウォッチ）、フォントスケールスライダー、テキストシャドウ切り替え、デスクトップアンカー・マージン・サイズ調整、ロケーション・気温単位切り替え、マルチモニター出力指定を実装。
   - `toodle` ウィジェット本体との inotify 連動ホットリロードを実装。
9. **ユーザー指摘（緊急修正4項目）**:
   - **① 設定ウィンドウの背景透過をやめて、通常の背景に**
   - **② Layout スライダーがリアルタイムで変化せず、結果的にディスプレイ外に出てしまう**
   - **③ テーマ（フォント）が何を選択しても変化しない**
   - **④ 現在地を選択してもデフォルトの東京から変化しない（プリセットの London でも）**
10. **4項目の根本原因特定と完全改修（即座に解決・検証完了）**:
    - 設定ウィンドウのルートコンテナに不透明ソリッドダーク背景（`Color::from_rgb(0.12, 0.13, 0.17)`）を適用。
    - スライダー操作時に即座に `config.save()` を実行し、inotify 遅延を 25ms に短縮してリアルタイム描画追従を実現。マージンの負値（-1000等）を廃止し `0..=2560` (X), `0..=1440` (Y) に制限して画面外飛び出しを物理防止。
    - `DejaVu Serif` および `Open Sans` の TTF をバイナリ組み込みに追加（計4フォントファミリ）。10プリセットすべてに固有のフォント（`RobotoSans`, `JetBrainsMono`, `DejaVuSerif`, `OpenSans`）をマッピングし、リアルタイム切り替えを実装。
    - `CachedWeather` に `latitude`, `longitude` フィールドと `is_location_match` 判定を追加。異なる座標が指定された場合は古い東京キャッシュをバイパスして新規取得するよう修正。クイック都市選択の即時自動保存とチェックバッジ表示、および「Apply Location & Refresh Weather」ボタンを追加。

---

## 3. 実施された主要な改善と技術的解決策

### A. Wayland Layer Shell 完全制御 & チラつきゼロのサーフェス更新
- **対象ファイル**: [`src/main.rs`](file:///home/susie/GitHUB/wammed/Toodle/src/main.rs), [`src/widget/mod.rs`](file:///home/susie/GitHUB/wammed/Toodle/src/widget/mod.rs), [`src/popup/mod.rs`](file:///home/susie/GitHUB/wammed/Toodle/src/popup/mod.rs)
- ウィジェット本体は `Layer::Bottom` でデスクトップ背景の上に常駐。
- ポップアップ（メニュー、カレンダー、週間予報）および編集モード枠は `Layer::Top` でデスクトップ最前面に排他表示。
- サーフェスの破棄・再生成を伴わないインプレース属性更新 (`layer_cmd::set_size`, `layer_cmd::set_margin`, `layer_cmd::set_anchor`) を徹底し、状態遷移時（編集モードの開始・終了、設定のホットリロード）の画面のチラつき（フリッカー）を完全に根絶。
- `set_input_zone` による非矩形入力領域制御により、時計・日付・天気の文字がある領域のみクリックを受け付け、それ以外の透明背景領域のクリックは下のデスクトップアイコンや壁紙へ透過。

### B. inotify 高速ファイル監視による設定ホットリロード & リアルタイム追従
- **対象ファイル**: [`src/config/mod.rs`](file:///home/susie/GitHUB/wammed/Toodle/src/config/mod.rs), [`src/settings/main.rs`](file:///home/susie/GitHUB/wammed/Toodle/src/settings/main.rs)
- `toodle` はバックグラウンドスレッドで `~/.config/toodle/` ディレクトリを `notify::recommended_watcher` で非同期監視。
- `toodle-settings` 側でスライダーを動かした瞬間、色スウォッチをクリックした瞬間、テーマや都市を選んだ瞬間にアトミック保存（一時ファイル `.tmp` 書込 → `rename`）を実行。
- ファイル変更検知の sleep を 25ms に最適化し、スライダー操作にデスクトップウィジェットが遅延なく追従。
- スライダー範囲を安全域（マージン `0..=2560`/`0..=1440`、幅 `150..=1200`、高さ `60..=800`）に厳格クランプし、設定値による画面外消失を防止。

### C. 組み込み 4 フォントファミリ & 10 テーマプリセットの完全連携
- **対象ファイル**: [`src/clock/fonts.rs`](file:///home/susie/GitHUB/wammed/Toodle/src/clock/fonts.rs), [`src/config/theme.rs`](file:///home/susie/GitHUB/wammed/Toodle/src/config/theme.rs), [`src/widget/mod.rs`](file:///home/susie/GitHUB/wammed/Toodle/src/widget/mod.rs)
- `include_bytes!` により 4 種類の高品質オープンソースフォント（計8ファイル）をバイナリへ静的組み込み（外部システムフォント非依存、設計第10項準拠）:
  - **Roboto** (Regular / Bold): クリーンな現代的サンセリフ
  - **JetBrains Mono** (Regular / Bold): 開発者向け洗練モノスペース
  - **DejaVu Serif** (Regular / Bold): クラシックで重厚なセリフ書体
  - **Open Sans** (Regular / Bold): 高い視認性を誇るヒューマニストサンセリフ
- 10種類のテーマプリセットごとに最適なフォント種別（`ThemeFontKind`）とデフォルト推奨カラーを定義:
  - `Modern` (Roboto Sans, `#FFFFFF`)
  - `Classic` (DejaVu Serif, `#E2E8F0`)
  - `Digital Mono` (JetBrains Mono, `#38BDF8`)
  - `Minimal` (Open Sans, `#94A3B8`)
  - `Cyberpunk` (JetBrains Mono, `#EAB308`)
  - `Nord` (Open Sans, `#38BDF8`)
  - `Warm Sunset` (DejaVu Serif, `#F59E0B`)
  - `Forest` (Roboto Sans, `#10B981`)
  - `Slate` (JetBrains Mono, `#94A3B8`)
  - `Rose Gold` (DejaVu Serif, `#EC4899`)
- テーマ選択時にフォントとカラーが自動連動し、即座にウィジェットへ反映。

### D. 天気キャッシュのロケーション座標検証 & オフライン耐性
- **対象ファイル**: [`src/weather/cache.rs`](file:///home/susie/GitHUB/wammed/Toodle/src/weather/cache.rs), [`src/weather/service.rs`](file:///home/susie/GitHUB/wammed/Toodle/src/weather/service.rs), [`src/weather/provider.rs`](file:///home/susie/GitHUB/wammed/Toodle/src/weather/provider.rs)
- Open-Meteo REST API（無料・APIキー不要・商用可）をバックエンドとし、現在天気と7日間予報を取得。
- キャッシュ構造体 `CachedWeather` に `latitude`, `longitude` を保持。
- `is_location_match(lat, lon)`（許容誤差約 2km / 0.02度）による厳密な座標一致判定を導入。異なる都市が指定された場合、古いキャッシュの誤返却を完全に防止。
- 現在天気は 30 分間、週間予報は 3 時間キャッシュを保持。ネットワーク障害・オフライン時は最後の有効キャッシュへ安全に自動フォールバック。
- 設定画面で都市を変更した際、ウィジェット側の表示を直ちに「...」ローディング状態へリセットし、新都市の天候が到着した段階で瞬時更新。

### E. ソリッドダーク UI スタイリング & 不透明ウィンドウ設計
- **対象ファイル**: [`src/settings/main.rs`](file:///home/susie/GitHUB/wammed/Toodle/src/settings/main.rs), [`src/popup/mod.rs`](file:///home/susie/GitHUB/wammed/Toodle/src/popup/mod.rs), [`src/popup/calendar.rs`](file:///home/susie/GitHUB/wammed/Toodle/src/popup/calendar.rs), [`src/popup/forecast.rs`](file:///home/susie/GitHUB/wammed/Toodle/src/popup/forecast.rs)
- 設定ウィンドウおよび全ポップアップサーフェス（右クリックメニュー、カレンダー、週間予報）において、デスクトップ背景が透けて可読性を損なうことがないよう、`Color::from_rgb(0.12, 0.13, 0.17)` による深みのある不透明ダークソリッド背景、上品なボーダーライン、ソフトなドロップシャドウを統一適用。

---

## 4. 変更された重要ファイル一覧

| ファイルパス | 主な役割・実装内容 |
|---|---|
| [`src/main.rs`](file:///home/susie/GitHUB/wammed/Toodle/src/main.rs) | ウィジェット本体のエントリポイント。Wayland Layer Shell 管理（`Layer::Bottom` 常駐ウィジェット、`Layer::Top` ポップアップ）、状態遷移、inotify 設定リロード監視受信、時計・天気の非同期タスクハンドリング |
| [`src/settings/main.rs`](file:///home/susie/GitHUB/wammed/Toodle/src/settings/main.rs) | 独立設定アプリ (`toodle-settings`) のエントリポイント。ソリッド背景 XDG Toplevel ウィンドウ（720x780）、4カテゴリタブ（Appearance/Layout/Weather/Display）、リアルタイム自動保存、安全スライダー制御、都市クイック選択 |
| [`src/widget/mod.rs`](file:///home/susie/GitHUB/wammed/Toodle/src/widget/mod.rs) | メイン時計ウィジェットの UI レンダリング、動的フォント割り当て（`get_font_pair_for_theme`）、テキストシャドウ、Edit Layout Mode のハイライト枠描画、非矩形入力領域（`content_bounds`）算出 |
| [`src/clock/mod.rs`](file:///home/susie/GitHUB/wammed/Toodle/src/clock/mod.rs) | サブセカンド同調の非同期ストリーム (`clock_tick_stream`)、1秒単位の正確な時刻更新 |
| [`src/clock/fonts.rs`](file:///home/susie/GitHUB/wammed/Toodle/src/clock/fonts.rs) | 4フォントファミリ（Roboto, JetBrains Mono, DejaVu Serif, Open Sans）計8ファイルのバイナリ組み込み（`include_bytes!`）および Iced フォント定義 |
| [`src/config/mod.rs`](file:///home/susie/GitHUB/wammed/Toodle/src/config/mod.rs) | アプリケーション設定構造体 (`Config`, `DisplayConfig`, `LayoutConfig`, `AppearanceConfig`, `WeatherConfig`)、TOML シリアライズ・アトミック保存、inotify ファイル監視ストリーム (`Config::watch`) |
| [`src/config/theme.rs`](file:///home/susie/GitHUB/wammed/Toodle/src/config/theme.rs) | 10テーマプリセット定義、16色固定カラーパレット、`ThemeFontKind`（フォントペアのマッピング）、Hex カラーパーサー |
| [`src/weather/mod.rs`](file:///home/susie/GitHUB/wammed/Toodle/src/weather/mod.rs) | 天気モジュールの公開インターフェース |
| [`src/weather/provider.rs`](file:///home/susie/GitHUB/wammed/Toodle/src/weather/provider.rs) | `WeatherProvider` トレイト、Open-Meteo REST API クライアント、WMO 天候コードから英語テキストへの変換ロジック |
| [`src/weather/cache.rs`](file:///home/susie/GitHUB/wammed/Toodle/src/weather/cache.rs) | `CachedWeather`（緯度・経度、取得日時、天気データ）、座標一致バリデーション (`is_location_match`)、30分/3時間キャッシュ有効期限判定、永続 JSON 保存・読込 |
| [`src/weather/service.rs`](file:///home/susie/GitHUB/wammed/Toodle/src/weather/service.rs) | キャッシュ優先取得、プロバイダ連携、オフライン時の stale キャッシュフォールバック、定期更新ストリーム |
| [`src/popup/mod.rs`](file:///home/susie/GitHUB/wammed/Toodle/src/popup/mod.rs) | ポップアップ共通メッセージ定義、ソリッド背景コンテキストメニュー（340x380, 18pxフォント、中央配置） |
| [`src/popup/calendar.rs`](file:///home/susie/GitHUB/wammed/Toodle/src/popup/calendar.rs) | 月間カレンダーポップアップ（680x720）、前月/翌月/今月移動、本日日付ハイライト、ソリッド背景 |
| [`src/popup/forecast.rs`](file:///home/susie/GitHUB/wammed/Toodle/src/popup/forecast.rs) | 週間天気予報ポップアップ（680x720）、7日間予報カード一覧、天候アイコン・気温幅表示、ソリッド背景 |
| `resources/fonts/` | 組み込みフォントファイル実体（`DejaVuSerif-*.ttf`, `OpenSans-*.ttf`, `Roboto-*.ttf`, `JetBrainsMono-*.ttf`） |

---

## 5. ビルド・検証コマンド

```bash
# 全バイナリのコンパイル確認 (toodle, toodle-settings)
cargo check --bin toodle && cargo check --bin toodle-settings

# 全単体テストの実行 (全10件すべてパス)
cargo test

# 天気モジュール単体テスト (キャッシュ検証、オフラインフォールバック、座標別キャッシュバイパス)
cargo test --lib weather

# リリースビルドの生成
cargo build --release

# デスクトップクロックウィジェット本体の起動
./target/debug/toodle

# 設定アプリケーションの起動
./target/debug/toodle-settings
```

---

## 6. 次回再開時の検討・作業候補 (Next Steps)

1. **Geocoding 自動解決の統合 (設計第12項)**:
   - 現状はクイック都市選択または緯度・経度の直接入力で動作中。
   - Open-Meteo Geocoding API (`geocoding-api.open-meteo.com`) を呼び出し、任意の都市名（例: "Kyoto", "Berlin"）の入力から緯度・経度を自動解決する debounce 入力ハンドラーを実装。
2. **Systemd User Service ユニットファイルの整備**:
   - COSMIC セッション起動時に `toodle` を自動起動するための `toodle.service`（`~/.config/systemd/user/toodle.service`）の作成。
3. **Arch Linux 向け PKGBUILD & デスクトップエントリの作成**:
   - `toodle-settings.desktop`、アイコンアセットの配置、Arch Linux (AUR) 向けの `PKGBUILD` の整備。
4. **マルチモニター出力切り替え (`DisplayConfig::output`) の動的マイグレーション**:
   - COSMIC 側のディスプレイ接続/切断イベントに追従した出力先サーフェスの動的付け替え。

---

## 7. 今後の機能追加・修正時におけるドキュメント同期方針 (Documentation Policy)

今後コード変更や機能追加を行う際は、以下の原則に則ってドキュメントを同期維持すること：

1. **`SESSION_HANDOVER.md` の継続更新**:
   - 新たなセッションで機能追加やバグ修正を実施した場合、セクション 2（要望履歴）、セクション 3（技術的解決策）、セクション 4（変更ファイル一覧）に追記し、常に最新の引き継ぎ状態を保つこと。
2. **日英ドキュメントの対称性**:
   - `README.md` (EN) と `README.ja.md` (JA)、`docs/ARCHITECTURE.md` (EN) と `docs/ARCHITECTURE.ja.md` (JA) は、機能追加時に必ず両言語同時に更新すること。
3. **設計ドキュメント (`Toodle-Design-Docs.md`) との整合性**:
   - 変更が設計原則（Section 1〜23）に影響を与える場合、設計ドキュメントの更新または設計ドキュメントからの逸脱理由を明確に文書化すること。
