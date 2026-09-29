# Toodle 機能仕様書 (Feature Specification)

<p align="center">
  <a href="FEATURES.md">English</a> | <strong>日本語</strong> | <a href="PORTAL.ja.md">📚 ドキュメントポータル</a> | <a href="../README.ja.md">← ルート README</a>
</p>

---

本ドキュメントは、COSMIC Desktop 向けデジタルクロックウィジェット **Toodle** の機能仕様および操作体系を詳述します。

> **仕様 Baseline**: 本仕様は [`docs/Drafts/Toodle-Design-Docs-v0.3.md`](file:///home/susie/GitHUB/wammed/Toodle/docs/Drafts/Toodle-Design-Docs-v0.3.md)（v0.3 Phase 0実機検証反映版 / Current Implementation Baseline）に完全準拠しています。

---

## 1. デジタル時計 & 日付表示機能

- **高精度デジタルクロック**:
  - 24時間制 `HH:MM:SS` 形式で時・分・秒を表示。
  - 次の 1 秒のマイクロ秒境界にアライメントする非同期同調ストリームにより、CPU 負荷なしに正確な 1 秒間隔更新を実現。
- **フル日付表示**:
  - 曜日、月名（英語フルスペル）、日、年を表示（例: `Monday, September 20, 2026`）。
  - English (US) 表記に固定。
- **統合タイポグラフィ連動スケーリング**:
  - 10段階の固定サイズステージに連動し、時刻（38〜310px）、日付（14〜110px）、余白、パディングが 1:1 で自動同期。ぼかしや文字潰れ、下部見切れの発生を物理的に根絶。
- **テキストドロップシャドウ**:
  - 明るい壁紙や写真壁紙の上でも高い視認性を保つソフトなアンビエントシャドウ（`Color::from_rgba(0, 0, 0, 0.65)`, オフセット: `(1.0, 2.0)`, ぼかし: `6.0px`）の ON/OFF 切り替えに対応。

---

## 2. 天気情報 & 週間予報サブシステム

- **現在天気**:
  - 天候テキスト（例: `Clear sky`, `Partly cloudy`, `Rain`）と現在の気温を表示。
  - 気温単位の切り替え（摂氏 `°C` / 華氏 `°F`）。
  - 都市変更中は一時的にローディング `"..."` 表示へリセット。
- **7日間週間予報ポップアップ (`680 x 720`)**:
  - 右クリックコンテキストメニューからワンクリックで開閉。
  - 7日分のカード一覧表示、日付表記（例: `Mon, Sep 21`）。
  - 本日カードに `(Today)` バッジを表示。
  - 最低気温、最高気温、天候テキスト、降水確率、天候アイコンを明瞭に描画。
  - 壁紙に左右されないソリッドダーク背景。
- **2段階永続キャッシュ & 堅牢性**:
  - 現在天気キャッシュ: 30分間有効。
  - 週間予報キャッシュ: 3時間有効。
  - 地理座標の厳密検証（`is_location_match`）により、都市変更時は古いキャッシュを即座にバイパスして最新データを取得。
  - リクエスト世代管理（`WeatherStateManager`）により、都市変更時や再試行時に遅れて届いた古い非同期レスポンスを破棄し、表示の上書きを防止。
  - 起動時キャッシュ検証により、設定座標と一致し（`is_location_match`）、有効期限内（`is_current_valid`）である場合のみ起動直後に表示。
  - 厳格なレスポンス検証: 気温や天気コードの欠落時に無音で `0.0°C` や快晴へフォールバックせず明示的なパースエラーを返却。`daily` オブジェクトの存在、配列長の完全一致、最低7日間の予報データ充足を検証。
  - プロセスID/ナノ秒タイムスタンプ一時ファイル（`.pid.nanos.tmp`）による競合セーフなキャッシュアトミック保存と、保存失敗時の診断警告ログ記録。
  - 座標の有限範囲検証（緯度 `-90.0..=90.0`、経度 `-180.0..=180.0`）により不正なリクエストを防止。
  - システム時刻変更時のキャッシュ永久凍結を防ぐクロックスキュー保護。
  - オフライン時・通信障害時は現在の都市における最後の有効キャッシュへ自動フォールバック。

---

## 3. ポップアップ & メニュー群 (`Layer::Top`)

すべてのポップアップは独立した `Layer::Top` サーフェスであり、ソリッドダーク背景（`Color::from_rgb(0.12, 0.13, 0.17)`）、上品な境界線、ドロップシャドウを備えています。

### 3.1 コンテキストメニュー (`340 x 380`)
- クロックウィジェット上のどこでも右クリックで起動。
- 18px の視認性の高い中央揃えアクションボタン：
  - `Calendar`: 月間カレンダーポップアップを開く
  - `Weekly Forecast`: 7日間週間予報ポップアップを開く
  - `Edit Layout`: デスクトップレイアウト編集モードへ移行
  - `Settings`: 独立設定アプリ（`toodle-settings`）を起動
  - `Quit`: ウィジェットデーモンを安全に終了

### 3.2 月間カレンダーポップアップ (`680 x 720`)
- 現在の年月、および洗練された 7 列カレンダーグリッドを表示。
- 曜日ヘッダー: `Mon`, `Tue`, `Wed`, `Thu`, `Fri`, `Sat`, `Sun`。
- インタラクティブな月送り操作：
  - `<` (前月へ)
  - `Today` (今月・本日の位置へジャンプ)
  - `>` (翌月へ)
- 今日の日付をハイライトカラーで強調表示。

---

## 4. デスクトップ Edit Layout モード (2 サーフェス方式)

ウィジェット本体を編集 UI に変形させるのではなく、表示と操作を分離した 2 サーフェス構造を採用しています：
- **アーキテクチャ**:
  - **Bottom サーフェス**: `Layer::Bottom` で動作する実際の `Toodle` ウィジェット。
  - **Top サーフェス**: `Layer::Top` に独立して生成される `Edit Layout Panel`。
- **パネル上の操作コントロール**:
  - **9分割グリッド配置**: 3×3 の直感的なマトリクスボタン（`TopLeft`, `TopCenter`, `TopRight`, `MiddleLeft`, `Center`, `MiddleRight`, `BottomLeft`, `BottomCenter`, `BottomRight`）。
  - **10段階固定サイズ（WQHD対応）**: 2行のプリセットサイズボタン（`1: 280`, `2: 380`, `3: 490`, `4: 620`, `5: 780` / `6: 960`, `7: 1180`, `8: 1440`, `9: 1740`, `10: 2060`）。
  - `Save` ボタン: 変更を `config.toml` に保存し、パネルを閉じて入力領域を通常へ復元。
  - `Cancel` ボタン: 編集を破棄し、編集前の状態をインプレース Layer Command で復元して閉じる。
- **入力領域（Input Region）の制御**:
  - Edit モード開始時、ウィジェット本体の入力領域を全体（`None`）へ拡張。
  - 終了時に通常の `content_bounds()` 矩形領域へ復帰。
- **チラつきゼロのインプレース更新**:
  - ボタン選択時はサーフェスを再生成せず、既存サーフェスへ Layer Command（`set_margin`, `set_size`, `set_anchor`）を発行して 0ms で滑らかに更新。

---

## 5. 独立設定アプリケーション (`toodle-settings`)

ソリッドダーク背景を備えたネイティブ XDG Toplevel ウィンドウ（`720 x 780`）。以下の 4 つのカテゴリタブを提供：

### 5.1 Appearance タブ
- **テーマプリセット一覧**:
  - 10種類のプリセット（フォント名併記）: `Modern` (Roboto), `Classic` (DejaVu Serif), `Digital Mono` (JetBrains Mono), `Minimal` (Open Sans), `Cyberpunk` (JetBrains Mono), `Nord` (Open Sans), `Warm Sunset` (DejaVu Serif), `Forest` (Roboto), `Slate` (JetBrains Mono), `Rose Gold` (DejaVu Serif)。
  - プリセット選択時にフォントと推奨アクセントカラーがウィジェットへ即時反映。
- **16色固定カラーパレット**:
  - チェックマーク（`✓`）付きのカラーパレット。
  - 厳選カラー: Pure White, Soft Silver, Cool Slate, Sky Blue, COSMIC Blue, Indigo, Purple, Rose Pink, Crimson, Coral Red, Orange, Amber Gold, Sun Yellow, Lime, Emerald Green, Teal Cyan。
- **Text Shadow スイッチ**: テキストシャドウの ON/OFF を即時切り替え。

### 5.2 Layout タブ
- **9分割ディスプレイ配置**: 3×3 のグリッド配置ボタン（`TopLeft` 〜 `BottomRight`）。
- **10段階サイズセレクタ**: 画面サイズに合わせた 10 段階の寸法プリセットボタン。ウィンドウサイズ変更に伴い、フォントサイズ（時刻・日付・天候）が最適比率で完全連動。
- **リアルタイム反映**: ボタン操作時にアトミック保存され、inotify 経由でデスクトップウィジェットが遅延なく連動。

### 5.3 Weather タブ
- **ロケーション入力**: `Location Name`, `Latitude`, `Longitude` テキスト入力フィールド。
- **クイック都市プリセット**: ワンクリックボタン（`Tokyo`, `Gifu`, `London`, `New York`, `Paris`）。
- **気温単位切り替え**: `Celsius (°C)` / `Fahrenheit (°F)`。
- **Apply & Refresh ボタン**: 即座に新設定を反映して最新天気を強制取得。

### 5.4 Display タブ
- **出力先ディスプレイ名**: 対象 Wayland 出力先（例: `"DP-1"`）の指定フィールド。
- **動的出力先解決**: `OutputManager` により動的に解決。接続中の画面名と照合し、切断時や未指定時はアクティブ/プライマリ画面へ安全にフォールバック、画面ごとの個別解像度に基づいてジオメトリを算出。
- **実機検証状況**: Wayland 環境下の同一機種 2560×1440 × 2（`DP-1`, `DP-2`）において、出力先の切り替え、表示位置、表示サイズが正しく動作することを実機にて検証完了。（異なる解像度の混在、マルチモニターでの mixed DPI、分数スケーリングは未検証事項）。

---

## 6. ビルド & ローカルインストール (`tools/install-local.sh`)

- **`cargo build --release`**: バイナリ（`toodle`, `toodle-settings`）のコンパイルのみを行い、ユーザーのホームディレクトリやシステム環境を一切変更しません。
- **`./tools/install-local.sh`**: 開発者がローカル環境への反映を意図した場合に明示的に実行し、リリースバイナリを `$HOME/.local/bin` にインストールします。

---

## 7. ライセンス & 意匠プロヴェナンス (Licensing & Visual Asset Provenance)

- **組み込みフォント & 天気アセット**:
  - バイナリに組み込まれた 4 系統のフォント（Roboto, Open Sans, JetBrains Mono NL, DejaVu Serif）および Bas Milius 氏による Meteocons SVG アイコンは、各オープンソースライセンスに準拠して利用されています。詳細は [`LICENSES/README.ja.md`](../LICENSES/README.ja.md) を参照してください。
- **アプリアイコン & ビジュアルアイデンティティのクリアランス**:
  - Toodle のアプリアイコン（頭文字「T」・時計文字盤・太陽/天気記号の統合シンボル）は、AI 支援による生成を経て類似性・ブランディングに関する監査が実施されています。
  - 正式なプロヴェナンスおよび知的財産デューデリジェンス記録は、[IP_COMPLIANCE.ja.md](../IP_COMPLIANCE.ja.md) および 4 アプリ総合監査アーカイブ [ICON_DESIGN_HISTORY.ja.md](../ICON_DESIGN_HISTORY.ja.md) に記録されています。

---

<p align="center">
  <a href="PORTAL.ja.md">📚 ドキュメントポータル</a> | <a href="../README.ja.md">← ルート README</a> | <a href="FEATURES.md">English Spec →</a>
</p>

