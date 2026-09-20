# Toodle 開発引き継ぎサマリー (Session Handover & Continuation Guide)

本ドキュメントは、これまでの開発・改修内容の全履歴、技術的決定事項、実機検証による設計変更、および今後の開発再開時にスムーズに作業を継続できるようにまとめた包括的な引き継ぎ資料です。

> **設計書 Baseline**: 今後の実装・設計の正となる基準文書は [`docs/Drafts/Toodle-Design-Docs-v0.3.md`](file:///home/susie/GitHUB/wammed/Toodle/docs/Drafts/Toodle-Design-Docs-v0.3.md)（v0.3 Phase 0実機検証反映版 / Current Implementation Baseline）です。
> 実機検証で成立した Surface / State / 操作モデルとの整合性を最優先とし、「設計とコードが違う」ことだけを理由にコードを旧設計（v0.2）へ戻さないことを大原則とします。

---

## 1. プロジェクト概要 & 技術構成

- **アプリケーション名**: Toodle (COSMIC Desktop Clock Widget)
- **リポジトリパス**: `/home/susie/GitHUB/wammed/Toodle`
- **設計ドキュメント**: [`docs/Drafts/Toodle-Design-Docs-v0.3.md`](file:///home/susie/GitHUB/wammed/Toodle/docs/Drafts/Toodle-Design-Docs-v0.3.md) (v0.3 Baseline)
- **開発言語**: Rust (2021 edition)
- **主要バイナリ**:
  - `toodle`: デスクトップクロックウィジェット本体 (`wlr-layer-shell` `Layer::Bottom`) および独立 Layer Top サーフェス群 (`Layer::Top`: コンテキストメニュー、カレンダー、7日間週間予報、Edit Layout Panel)
  - `toodle-settings`: XDG Toplevel ネイティブ設定ウィンドウ (Appearance, Layout, Weather, Display の4タブ構成)
- **主要フレームワーク・ライブラリ**:
  - GUI ツールキット: `libcosmic` (v0.1 / git: `pop-os/libcosmic`, `wayland` feature)
  - ウィンドウ管理: `cosmic::iced::wayland::layer_surface` (`Layer::Bottom`, `Layer::Top`)
  - 設定ファイル監視: `notify` (inotify による `config.toml` 非同期ホットリロード)
  - 天気 API / キャッシュ: `reqwest`, `tokio`, `serde`, `serde_json`, `chrono` (Open-Meteo REST API, 2段階ローカル永続キャッシュ)
  - 組み込みフォント: `Roboto`, `JetBrains Mono`, `DejaVu Serif`, `Open Sans` (`include_bytes!` による完全スタンドアロンバイナリ、外部フォント非依存)
- **ライセンス**: `GPL-3.0-or-later` ([LICENSE](LICENSE))

---

## 2. コアルール (Core Rules v0.3)

設計書 v0.3 Section 1 に基づき、以下のルールを設計上の基本固定事項とします：

1. Widget 本体は Layer Shell / `Layer::Bottom` で壁紙上に常駐表示する。
2. Context Menu、Calendar、Weekly Forecast、Edit Layout Panel は独立した Layer Shell / `Layer::Top` Surface とする (`xdg_popup` は不採用)。
3. Settings は通常の XDG Toplevel Window として `toodle-settings` に分離する。
4. Widget の表示状態は `Normal` と `Edit` の2状態を基本とする。
5. Edit Mode は Widget 本体の Surface を直接編集するのではなく、`Layer::Top` の独立 Edit Layout Panel と組み合わせて実現する（2 Surface 方式）。
6. Normal Mode では Widget 本体の通常 Surface を操作対象とし、右クリックで Context Menu を開く。
7. Edit Mode では Widget 本体の Input Region を一時的に全体（`None`）へ拡張し、同時に Edit Layout Panel を表示する。終了時に通常矩形へ戻す。
8. レイアウト変更は Anchor / Margin / Width / Height を基本とし、Font Scale も Edit Layout から変更できる。
9. レイアウト変更中は既存の Widget Surface を破棄・再生成せず、Layer Surface Command (`set_anchor`, `set_margin`, `set_size`, `set_input_zone`) によるインプレース更新を徹底する（フリッカー防止）。
10. 設定は `~/.config/toodle/config.toml` に保存する。
11. `toodle-settings` は設定変更をアトミック保存し、`toodle` は file watcher により変更を再読み込みする。
12. Weather API との境界は `WeatherProvider` トレイトとする。
13. API Key 管理は行わない。
14. 時刻は OS の Local Time を使用する。
15. Clock Tick は次の秒境界（サブセカンドアライメント）を基準に再同期する。
16. Widget は1インスタンスを基本とする。
17. 外部システムフォントには依存せず、使用フォント（4ファミリ計8ファイル）をバイナリへ組み込む。
18. 通常運用時は Settings GUI プロセスを常駐させず、Widget 本体を独立して動作させる。
19. Popup および Settings は透明背景による可読性問題を避けるためソリッドダーク背景を使用する。

---

## 3. これまでのユーザー要望と対応履歴（時系列）

1. **基本設計確定とフェーズ分割計画の策定**:
   - 初期設計に基づき、Prototype 0 (Phase 0: サーフェス基盤) から Phase 4 (設定アプリ) までのステップバイステップ開発計画を合意。
2. **Phase 0 — Wayland Layer Shell サーフェス基盤の実装**:
   - `cosmic-comp` (COSMIC Desktop Environment) 上で動作するデスクトップウィジェットを作成。
   - `Layer::Bottom` による最背面配置、`content_bounds()` による矩形入力領域（透明領域のクリックをデスクトップ背景へパススルー）。
   - 右クリックによる Context Menu および Edit Layout Panel の起動。
3. **ユーザー指摘: チラつき、スライダー追従遅延、サイズ上限、日本語表記排除、右クリックメニュー崩れ**:
   - スライダー操作時の再生成によるチラつきを解消。
   - スライダーのフォントサイズ上限・ウィンドウサイズ上限を拡大。
   - Edit Layout Panel 内および右クリックコンテキストメニューから日本語表記を全廃（英語ネイティブUI化）。
   - 右クリックコンテキストメニューの崩れを修正し、背景透過を廃止してソリッドダーク背景を適用。
4. **ユーザー指摘: Edit Layout 終了時のチラつき解消**:
   - `EditLayoutSave` および `EditLayoutCancel` 時にサーフェスを破棄・再生成せず、`layer_cmd::set_size`、`layer_cmd::set_margin`、`layer_cmd::set_anchor` による同一サーフェスのインプレースプロパティ更新へ移行し、画面のチラつきをゼロ化。
   - **Phase 0 完了の承認を受領**。
5. **Phase 1 (Clock) & Phase 2 (Weather) の実装**:
   - **Phase 1 (Clock)**: 次の秒の境界（サブセカンドアライメント）に正確に同調する `clock_tick_stream` の実装。組み込みフォント（Roboto, JetBrains Mono）の導入。
   - **Phase 2 (Weather)**: `WeatherProvider` トレイト境界による Open-Meteo プロバイダ実装。ローカル JSON 2段階キャッシュ機構、オフライン時の stale キャッシュ自動フォールバック。
6. **Phase 3 — ポップアップ機能群の実装とサイズ決定**:
   - ユーザー指定サイズに基づき独立 Layer::Top サーフェスを実装:
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
9. **ユーザー指摘（実機検証緊急修正4項目）**:
   - **① 設定ウィンドウの背景透過をやめて、通常の背景に**
   - **② Layout スライダーがリアルタイムで変化せず、結果的にディスプレイ外に出てしまう**
   - **③ テーマ（フォント）が何を選択しても変化しない**
   - **④ 現在地を選択してもデフォルトの東京から変化しない（プリセットの London でも）**
10. **4項目の根本原因特定と完全改修（即座に解決・検証完了）**:
    - 設定ウィンドウのルートコンテナに不透明ソリッドダーク背景（`Color::from_rgb(0.12, 0.13, 0.17)`）を適用。
    - スライダー操作時に即座に `config.save()` を実行し、inotify 遅延を 25ms に短縮してリアルタイム描画追従を実現。マージンを安全域（`0..=2560` X, `0..=1440` Y）に制限して画面外飛び出しを物理防止。
    - `DejaVu Serif` および `Open Sans` の TTF をバイナリ組み込みに追加（計4フォントファミリ・8ファイル）。10プリセットすべてに固有のフォントをマッピングし、リアルタイム切り替えを実装。
    - `CachedWeather` に `latitude`, `longitude` フィールドと `is_location_match` 判定を追加。異なる座標が指定された場合は古いキャッシュをバイパスして新規取得するよう修正。クイック都市選択の即時自動保存とチェックバッジ表示を追加。
11. **アプリアイコンの統合とシステム登録**:
    - `images/toodle-icon.svg` に基づき全解像度 PNG および SVG を生成し、`~/.local/share/icons/hicolor/` 配下にインストール。
    - デスクトップエントリファイルを作成・登録。
12. **設計書 v0.3 の策定と公式 Baseline 化**:
    - 実機検証で成立した挙動・決定事項（独立 Top サーフェス、2 サーフェス Edit Layout、インプレース更新、ソリッド背景、矩形 Input Zone、Known Gaps の明示）を新設計書 [`docs/Drafts/Toodle-Design-Docs-v0.3.md`](file:///home/susie/GitHUB/wammed/Toodle/docs/Drafts/Toodle-Design-Docs-v0.3.md) に集約。

---

## 4. 確立された主要アーキテクチャ & 技術的解決策

### A. Wayland Layer Shell 完全制御 & チラつきゼロのサーフェス更新
- **対象ファイル**: [`src/main.rs`](file:///home/susie/GitHUB/wammed/Toodle/src/main.rs), [`src/widget/mod.rs`](file:///home/susie/GitHUB/wammed/Toodle/src/widget/mod.rs), [`src/popup/mod.rs`](file:///home/susie/GitHUB/wammed/Toodle/src/popup/mod.rs)
- ウィジェット本体は `Layer::Bottom` でデスクトップ背景の上に常駐。
- ポップアップ（メニュー、カレンダー、週間予報）および Edit Layout Panel は `Layer::Top` の独立サーフェスとして必要時のみ生成・排他表示。
- サーフェスの破棄・再生成を伴わないインプレース属性更新 (`layer_cmd::set_size`, `layer_cmd::set_margin`, `layer_cmd::set_anchor`, `layer_cmd::set_input_zone`) を徹底し、状態遷移時や設定リロード時の画面のチラつき（フリッカー）を完全に根絶。
- `set_input_zone` により、Normal 時は `content_bounds()`（Layout 幅/高さを基準とした矩形領域）に入力を制限し、余白の透明領域はデスクトップへクリック透過。Edit 時は入力領域を全体（`None`）に拡張。

### B. Edit Layout アーキテクチャ (2 Surface 方式)
- **対象ファイル**: [`src/main.rs`](file:///home/susie/GitHUB/wammed/Toodle/src/main.rs), [`src/widget/mod.rs`](file:///home/susie/GitHUB/wammed/Toodle/src/widget/mod.rs)
- v0.2 の「Widget Surface 自体を編集 UI に変形する」方式から、v0.3 では「Bottom Widget Surface + Top 独立 Edit Layout Panel」の 2 Surface 方式を正式採用。
- Edit Layout Panel から Margin, Size, Anchor, Font Scale を操作すると、Bottom の Widget Surface に対してインプレース Layer Command が即時発行されリアルタイムに変形。
- Save 時は Config 保存後に Panel を破棄し、Cancel 時は編集前の状態を Layer Command で復元。

### C. inotify 高速ファイル監視による設定ホットリロード & リアルタイム追従
- **対象ファイル**: [`src/config/mod.rs`](file:///home/susie/GitHUB/wammed/Toodle/src/config/mod.rs), [`src/settings/main.rs`](file:///home/susie/GitHUB/wammed/Toodle/src/settings/main.rs)
- `toodle` はバックグラウンドスレッドで `~/.config/toodle/` ディレクトリを `notify::recommended_watcher` で非同期監視。
- `toodle-settings` 側でスライダー等を動かした瞬間にアトミック保存（一時ファイル `.tmp` 書込 → `rename`）を実行。
- ファイル変更検知後に新設定を読み込み、ウィジェット側へインプレース反映。安全域クランプ（マージン `0..=2560`/`0..=1440`、幅 `150..=1200`、高さ `60..=800`）により画面外消失を防止。

### D. 組み込み 4 フォントファミリ & 10 テーマプリセット
- **対象ファイル**: [`src/clock/fonts.rs`](file:///home/susie/GitHUB/wammed/Toodle/src/clock/fonts.rs), [`src/config/theme.rs`](file:///home/susie/GitHUB/wammed/Toodle/src/config/theme.rs), [`src/widget/mod.rs`](file:///home/susie/GitHUB/wammed/Toodle/src/widget/mod.rs)
- `include_bytes!` により 4 種類の高品質オープンソースフォント（計8ファイル）をバイナリへ静的組み込み（外部システムフォント非依存）:
  - **Roboto** (Regular / Bold)
  - **JetBrains Mono** (Regular / Bold)
  - **DejaVu Serif** (Regular / Bold)
  - **Open Sans** (Regular / Bold)
- 10種類のテーマプリセットごとに最適なフォント種別（`ThemeFontKind`）とデフォルト推奨カラーをマッピング。

### E. 天気キャッシュのロケーション座標検証 & オフライン耐性
- **対象ファイル**: [`src/weather/cache.rs`](file:///home/susie/GitHUB/wammed/Toodle/src/weather/cache.rs), [`src/weather/service.rs`](file:///home/susie/GitHUB/wammed/Toodle/src/weather/service.rs), [`src/weather/provider.rs`](file:///home/susie/GitHUB/wammed/Toodle/src/weather/provider.rs)
- Open-Meteo REST API（無料・APIキー不要）をバックエンドとし、現在天気と7日間予報を取得。
- `CachedWeather` に `latitude`, `longitude` を保持し、`is_location_match(lat, lon)`（許容誤差約 2km / 0.02度）による一致判定を実施。都市変更時は古いキャッシュを即座にバイパスして新規取得。
- オフライン・障害時は前回の有効キャッシュへ安全に自動フォールバック。都市変更時はウィジェット表示を一時的にローディング `"..."` にリセット。

### F. ソリッドダーク UI スタイリング
- **対象ファイル**: [`src/settings/main.rs`](file:///home/susie/GitHUB/wammed/Toodle/src/settings/main.rs), [`src/popup/mod.rs`](file:///home/susie/GitHUB/wammed/Toodle/src/popup/mod.rs), [`src/popup/calendar.rs`](file:///home/susie/GitHUB/wammed/Toodle/src/popup/calendar.rs), [`src/popup/forecast.rs`](file:///home/susie/GitHUB/wammed/Toodle/src/popup/forecast.rs)
- デスクトップ背景による可読性低下を防ぐため、設定ウィンドウおよび全ポップアップサーフェス（右クリックメニュー、カレンダー、週間予報、Edit Layout Panel）に不透明ダークソリッド背景（`Color::from_rgb(0.12, 0.13, 0.17)`）を統一適用。

---

## 5. 既知の実装ギャップ (Known Implementation Gaps v0.3)

設計書 v0.3 Section 25 に基づき、現在確認されている設計とコードの差分・未実装項目を以下に明記します。これらを「実装済み」と誤認せず、今後のフェーズで計画的に対応します：

1. **Display Output 接続**:
   - `DisplayConfig::output`（例: `"DP-1"`）は設定として存在するが、現在の Layer Surface 生成では `IcedOutput::Active` が固定で使用されている（特定ディスプレイへのバインドが未接続）。
2. **Weather Current / Forecast Cache の独立 TTL 分離**:
   - 設計値（現在天気 30分 / 予報 3時間）は定義されているが、`WeatherService` の実際の取得・キャッシュ判定単位が完全に分離されておらず、一括フェッチ・判定となっている。
3. **Weather 429 / 5xx Exponential Backoff**:
   - API エラー時の指数バックオフ（1s, 2s, 4s... max 30m）は未実装。現在はエラーを返し stale cache にフォールバックするのみ。
4. **Geocoding 自動解決**:
   - 現在は都市プリセット（Tokyo, London 等）または手動の緯度・経度入力のみ。Location Name 文字列からの自動緯度・経度解決は未実装。
5. **Request Generation / Stale Result 保護**:
   - 都市変更時に以前の都市の遅延レスポンスが新都市の表示を上書きしないための Request Generation ID 管理は未導入。
6. **Popup Dismissal**:
   - ポップアップのフォーカス喪失（focus loss）や領域外クリック（outside click）による自動閉鎖は、実機挙動の検証および実装仕様の確定が必要。

---

## 6. プロジェクト構成 & 変更ファイル一覧

```text
Toodle/
├── Cargo.toml
├── Cargo.lock
├── README.md
├── README.ja.md
├── SESSION_HANDOVER.md
├── docs/
│   ├── ARCHITECTURE.md
│   ├── ARCHITECTURE.ja.md
│   ├── FEATURES.md
│   ├── FEATURES.ja.md
│   └── Drafts/
│       ├── Toodle-Design-Docs-v0.1.md
│       ├── Toodle-Design-Docs-v0.2.md
│       ├── Toodle-Design-Docs-v0.2-Final-Draft.md
│       └── Toodle-Design-Docs-v0.3.md  <-- 正式設計基準 (Baseline)
├── src/
│   ├── main.rs          # ウィジェット本体, Layer Shell管理, メッセージルーティング
│   ├── lib.rs
│   ├── widget/          # ウィジェット描画, content_bounds, Edit Layout State
│   ├── popup/           # Context Menu, Calendar, Forecast
│   ├── clock/           # クロックTickストリーム, 組み込みフォント定義
│   ├── weather/         # Provider, Cache, Service
│   ├── config/          # Config, Theme, 永続化, inotify Watcher
│   └── settings/        # toodle-settings XDG Toplevel UI
└── resources/
    ├── fonts/           # 組み込みTTF (4ファミリ計8ファイル)
    └── icons/           # アプリアイコン群
```

---

## 7. ビルド・検証コマンド

```bash
# 全バイナリのコンパイル確認 (toodle, toodle-settings)
cargo check --bin toodle && cargo check --bin toodle-settings

# 全単体テストの実行 (全10件)
cargo test

# 天気モジュール単体テスト (キャッシュ検証、座標一致、オフラインフォールバック)
cargo test --lib weather

# リリースビルドの生成
cargo build --release

# デスクトップクロックウィジェット本体の起動
./target/debug/toodle

# 設定アプリケーションの起動
./target/debug/toodle-settings
```

---

## 8. 実装ロードマップ (Implementation Roadmap v0.3)

新設計書 v0.3 Section 28 に基づく今後の作業優先度：

### Phase 5 — Robustness (堅牢化 & 未接続機能の解消)
1. **Display Output の実装接続**:
   - `DisplayConfig::output` を Layer Surface の `IcedOutput`（出力名指定）に正しく配線。
2. **Popup Dismissal の実機検証・実装**:
   - 外側クリック / フォーカス喪失時の自動クローズ機構の検証と実装。
3. **Weather Current / Forecast Cache の独立化**:
   - 現在天気（30分）と週間予報（3時間）の取得・キャッシュ判定を完全独立化。
4. **Weather 429 / 5xx Backoff の実装**:
   - レート制限やサーバーエラー時の指数バックオフリトライ。
5. **Weather Request Generation 管理**:
   - ロケーション変更時の stale な遅延レスポンス破棄機構。
6. **Geocoding 自動解決**:
   - Open-Meteo Geocoding API を用いた都市名入力からの自動座標解決。

### Phase 6 — Distribution (配布・システム統合)
1. **Systemd User Service**:
   - COSMIC セッション自動起動用 `toodle.service` の整備。
2. **Desktop Entry / Packaging**:
   - デスクトップエントリの洗練、Arch Linux (AUR) 向け PKGBUILD 作成。

---

## 9. ドキュメント運用ルール (Documentation Rule v0.3 Appendix B)

今後の実装変更では、以下の順序で文書を更新すること：

1. **実機で成立した挙動を確認する**。
2. **アーキテクチャ上の変更なら設計書（`Toodle-Design-Docs-v0.3.md`）を更新する**。
3. **実装上の不足・残課題なら Known Implementation Gaps を更新する**。
4. **`SESSION_HANDOVER.md` に変更理由と作業履歴を記録する**。
5. **`README.md` / `README.ja.md` / `docs/ARCHITECTURE.*` / `docs/FEATURES.*` の公開説明も同時に更新する**（日英の対称性を厳格に維持）。

> **重要原則**: **「設計とコードが違う」ことだけを理由にコードを旧設計へ戻さない。** 実機検証によって確定した挙動を新しい設計の基準とし、必要な場合のみ設計変更として文書化すること。
