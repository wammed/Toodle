<div align="center">

# ⏱️ Toodle
### System76 COSMIC Desktop Environment 向け モダン・デジタルクロック＆ウェザーウィジェット

![Banner](images/toodle-banner.svg)

[![Built with libcosmic](https://img.shields.io/badge/libcosmic-Pop!_OS_COSMIC-24C8D8?style=for-the-badge&logo=linux&logoColor=white)](https://github.com/pop-os/libcosmic)
[![Rust](https://img.shields.io/badge/Rust-1.85+-orange?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Wayland](https://img.shields.io/badge/Protocol-wlr--layer--shell-5277C3?style=for-the-badge&logo=wayland&logoColor=white)](https://wayland.freedesktop.org/)
[![Platform](https://img.shields.io/badge/Platform-Linux_(COSMIC_/_Wayland)-FCC624?style=for-the-badge&logo=linux&logoColor=black)](https://www.kernel.org/)
[![Vibe Coding](https://img.shields.io/badge/Built_with-AI_Vibe_Coding-8A2BE2?style=for-the-badge&logo=sparkles&logoColor=white)](#-このプロジェクトについて-ai-vibe-coding)
[![License: MIT](https://img.shields.io/badge/License-MIT-green?style=for-the-badge)](LICENSE)

<p align="center">
  <strong>COSMIC / Wayland ネイティブ × 高精度サブセカンドクロック × 2段階天気キャッシュ × 9ゾーン＆10段階サイズ自由変形</strong><br>
  デスクトップ壁紙の直上に常駐し、ゼロフリッカーで美しく変形する Pop!_OS COSMIC Desktop および Linux Wayland 向けの軽量高機能クロックウィジェット
</p>

<p align="center">
  <a href="README.md">English</a> | <a href="README.ja.md">日本語</a> | <a href="docs/PORTAL.ja.md">📚 ドキュメントポータル</a> | <a href="LICENSES/README.ja.md">📄 ライセンス通知</a>
</p>

</div>

---

## 🚀 クイックスタート

### 1. 必要環境

- [Rust (Cargo)](https://rustup.rs/) (1.85 以上、edition 2024 対応 / 最新 Stable 推奨 / 実機検証: 1.98.1)
- Linux Wayland 環境 / Pop!_OS COSMIC Desktop (`cosmic-comp`)
- システムビルド依存パッケージ (Debian / Pop!_OS / Ubuntu):
  ```bash
  sudo apt install build-essential libxkbcommon-dev wayland-protocols
  ```
  *(Arch Linux の場合: `sudo pacman -S base-devel libxkbcommon wayland`)*

### 2. ビルド

Toodle はソースコード形式で配布されます。ローカル環境でビルドしてください：

```bash
# リポジトリのクローン
git clone https://github.com/wammed/Toodle.git
cd Toodle

# ウィジェット本体および設定アプリのビルド
cargo build --release
```

### 3. ローカルインストール (任意)

ビルドしたバイナリ（`toodle`, `toodle-settings`）を `$HOME/.local/bin` にインストールする場合：

```bash
./tools/install-local.sh
```

### 4. 起動

```bash
# クロックウィジェット本体の起動 (デスクトップ背景 Layer::Bottom で常駐)
./target/release/toodle &
# またはローカルインストール済みの場合:
# toodle &

# 設定アプリケーションの起動
./target/release/toodle-settings
# またはローカルインストール済みの場合:
# toodle-settings
```

#### 💡 自動起動（Autostart）時の推奨設定
COSMIC デスクトップ環境の自動起動（設定アプリの「自動起動」や `~/.config/autostart/*.desktop`）に登録する場合、セッション初期化順序の競合を回避するため、起動コマンドに 3 秒程度の遅延を設定することを推奨します（実機検証済みワークアラウンド）：

```desktop
Exec=sh -c "sleep 3 && toodle"
```

---

## 💡 主な特徴

- 🪟 **COSMIC / Wayland ネイティブ (`wlr-layer-shell`)**: System76 公式ツールキット `libcosmic` を採用。壁紙直上の `Layer::Bottom` 常駐と、`content_bounds` 局所化によるデスクトップ背面クリック透過を実現。
- ⏱️ **マイクロ秒境界同期サブセカンドクロック**: 次の 1 秒境界に同調する非同期ストリームにより、CPU 負荷ゼロで正確無比な 1 秒更新を維持。
- 🌤️ **2段階キャッシュ付き天気サブシステム**: Open-Meteo REST API を利用し、30分（現在）/ 3時間（週間）のローカル永続キャッシュ。リクエスト世代 ID による古いレスポンス破棄、クロックスキュー防御、完全オフラインフォールバックを完備。
- 🖥️ **動的マルチモニター管理 (`OutputManager`)**: Wayland 出力先メタデータを自動追跡し、指定ディスプレイ（例: `"DP-1"`）へシームレスにバインド。未設定時・切断時のプライマリ自動フォールバックとジオメトリ確定遅延をサポート。
- 📐 **9分割配置 & 10段階サイズプリセット**: ディスプレイを 3×3（TopLeft〜BottomRight）に配置。WQHD まで最適化された 10 段階のサイズ（280px〜2060px）と、フォント・SVG 天気アイコンの完全連動スケーリング。
- ⚡ **2サーフェス Edit Layout モード**: 画面上部の独立操作盤パネルとウィジェット本体が連携。サーフェスの破棄・再生成を伴わないインプレース変形により、チラつき（Flicker）ゼロでリアルタイムに変形。
- ⚙️ **独立設定アプリ (`toodle-settings`)**: 4 タブ構成（Appearance, Layout, Weather, Display）のネイティブ XDG Toplevel ウィンドウ。10 種のテーマ、16 色パレット、クイック都市選択、リアルタイム自動保存を搭載。
- 🔄 **inotify 高速ホットリロード & 自動マイグレーション**: `~/.config/toodle/config.toml` の変更を 50ms 集約で即時反映。旧フォーマット設定ファイルは読み込み時に自動マイグレーション。
- 🎨 **完全組み込みフォント & カラーベクター天気アイコン**: Roboto, Open Sans, JetBrains Mono NL, DejaVu Serif を静的組み込み。Bas Milius Meteocons カラー SVG アイコンにより、どの環境・スケールでも鮮やかで美しい天候を表示。

> 📖 **詳細な仕様や設計解説**:
> 各機能の完全な技術仕様は [docs/FEATURES.ja.md](docs/FEATURES.ja.md) を、マルチサーフェスや内部アーキテクチャの詳細は [docs/ARCHITECTURE.ja.md](docs/ARCHITECTURE.ja.md) をご覧ください。

---

## 🖱️ 基本操作 & インタラクション

最頻出の主要操作一覧です。

| 操作対象 | 入力 / アクション | 動作内容 |
| :--- | :--- | :--- |
| **ウィジェット本体** | **右クリック** | **コンテキストメニュー** 表示 (`Calendar`, `Weekly Forecast`, `Edit Layout`, `Settings`, `Quit`) |
| **ウィジェット本体** | **左クリック** (領域外) | **背後のデスクトップへクリック透過** (Input Region 最小化) |
| **ポップアップ群** | **外側クリック** / **`Esc`** | 表示中のメニュー・カレンダー・天気予報を閉じる |
| **カレンダー** | **「<」「>」ボタン** | 前月 / 翌月へスムーズに切り替え（「今日」で即時復帰） |
| **Edit Layout Panel** | **9ゾーンボタン** | ウィジェットの位置を瞬時にデスクトップ上で移動 |
| **Edit Layout Panel** | **1〜10 サイズスライダ** | ウィジェット寸法とフォントをインプレースでリアルタイム変形 |
| **Edit Layout Panel** | **Save** / **Cancel** | 変更を設定ファイルにアトミック保存 / 変更前の状態に復元 |
| **設定アプリ** | **設定項目を変更** | 変更が即座に `config.toml` に自動保存され、ウィジェットへ即時反映 |

> 🖱️ **完全版操作ガイド**:
> 詳細な操作手順やサーフェス構造は **[docs/SHORTCUTS.ja.md](docs/SHORTCUTS.ja.md)** を参照してください。

---

## 📚 ドキュメントポータル

Toodle のより詳しいドキュメントは、以下の専門ドキュメントに体系化されています：

| ドキュメント | 内容 |
| :--- | :--- |
| **[📚 ドキュメントポータル](docs/PORTAL.ja.md)** | 目的別・読者別の総合案内ハブ |
| **[🖱️ 操作 & インタラクションガイド](docs/SHORTCUTS.ja.md)** | 全マウス・キー操作、カレンダー、Edit Layout、設定アプリ操作の完全リファレンス |
| **[💡 詳細機能仕様書 (FEATURES)](docs/FEATURES.ja.md)** | 10段階サイズ寸法表、全10種テーマ、天気キャッシュ仕様、カラーSVGアイコン詳細 |
| **[📐 アーキテクチャ設計書 (ARCHITECTURE)](docs/ARCHITECTURE.ja.md)** | マルチサーフェス階層、動的マルチモニター管理、サブセカンドクロックストリーム |
| **[🛡️ 堅牢性 & セキュリティモデル (SECURITY)](docs/SECURITY.ja.md)** | 2段階キャッシュ整合性、リクエスト世代管理、PIDアトミック保存、Input Region透過 |
| **[🛡️ 知的財産コンプライアンス・デューデリジェンス (IP_COMPLIANCE)](IP_COMPLIANCE.ja.md)** | Toodle アイコンの由来、類似性レビュー結果、知財デューデリジェンス記録 |
| **[🎨 アイコンデザイン・IPレビュー履歴 (ICON_DESIGN_HISTORY)](ICON_DESIGN_HISTORY.ja.md)** | 4アプリ横断の AI 生成ログ、類似性監査履歴、デザイン変遷の記録 |
| **[📄 ライセンス & サードパーティ通知](LICENSES/README.ja.md)** | プロジェクトライセンス、Meteocons、フォント、全依存クレートのライセンス監査記録 |

---

## 🔒 信頼性 & セキュリティ (概要)

- **ゼロトラッキング・完全オフライン耐性**: クラウド送信、アナリティクス、テレメトリは一切行いません。ネットワーク切断時でもローカルキャッシュによりシームレスに動作を継続します。
- **リクエスト世代カウンター (Stale Protection)**: 非同期天気リクエストに世代 ID を付与し、ネットワーク遅延による古いレスポンスの上書きを 100% 遮断。
- **PID 付与一時ファイルによるアトミック保存**: `config.toml` やキャッシュ保存時に `.{file}.tmp.{pid}` への書き込み・`sync_all`・アトミックな `rename` を行うことで、不完全な書き込みによるファイル破損を極小化。
- **Input Region 最小化**: ウィジェットの表示矩形（`content_bounds`）のみを入力領域とし、デスクトップ背面への干渉やクリックジャックを根本から防止。

> 🛡️ **セキュリティ詳細**:
> より詳しい堅牢性・セキュリティ仕様については **[docs/SECURITY.ja.md](docs/SECURITY.ja.md)** をご覧ください。

---

## 🗺️ ロードマップ

- [x] ⏱️ **高精度サブセカンドクロック**: 次の 1 秒マイクロ秒境界同期ストリームによる正確な計時。
- [x] 🌤️ **2段階キャッシュ付き天気サブシステム**: Open-Meteo REST API、リクエスト世代管理、オフラインフォールバック。
- [x] 📐 **9ゾーン配置 & 10段階サイズプリセット**: WQHD 最適化寸法とインプレースリアルタイム変形。
- [x] 🎛️ **独立設定アプリ (`toodle-settings`)**: 4 タブ、リアルタイム自動保存、10 テーマ、16 色パレット。
- [x] 📁 **ライセンス関係の集約 (`LICENSES/`)**: 全アセットライセンスおよび cargo-deny 監査記録の体系化。
- [ ] 🖥️ **Mixed DPI / 分数スケーリング（Fractional Scaling）実機検証**: 異なる DPI 混在マルチモニター環境での検証・堅牢化。
- [ ] 📅 **カレンダーイベント連動**: ローカル iCalendar (.ics) またはシステムカレンダーとのイベント同期表示。
- [ ] ⏰ **アラーム & タイマー機能**: デスクトップ通知と連動したシンプルなタイマー・リマインダー。

---

## 🤖 このプロジェクトについて (AI Vibe Coding)

> [!IMPORTANT]
> ### 💡 AI Vibe Coding による開発
> **Toodle** は、**Google DeepMind の Antigravity (Gemini)** との対話的ペアプログラミング（AI Vibe Coding）によって作成されたプロジェクトです。
> 人間による設計方針・アイデアの提示と、AI による実装・デバッグ・最適化のフローを組み合わせ、低レイヤの Rust `libcosmic` Wayland Layer Shell プロトコル、マルチサーフェス構成、マイクロ秒境界同期クロック、2段階天気キャッシュ、インプレース変形 Edit Layout パネル、独立 XDG Toplevel 設定アプリに至るまでフルスクラッチで実装されました。

---

## 📄 ライセンス

Toodle 本体のソースコードは [MIT License](LICENSE) のもとで公開されています。

ライセンス方針、サードパーティ依存関係の適合性、およびアセットのプロヴェナンス（由来）記録の詳細は以下を参照してください：

- **[ライセンス総合案内 (LICENSES/README.ja.md)](LICENSES/README.ja.md)**: ソースコード配布モデル、依存クレート管理方針（`cargo-deny`）、システム共有ライブラリの動的リンク、パッケージ化時の注意点。
- **[アイコン知的財産コンプライアンス・デューデリジェンス記録 (IP_COMPLIANCE.ja.md)](IP_COMPLIANCE.ja.md)**: Toodle アイコンの由来、類似性レビュー結果、知財デューデリジェンス記録。
- **[アイコンデザイン・IPレビュー履歴 (ICON_DESIGN_HISTORY.ja.md)](ICON_DESIGN_HISTORY.ja.md)**: 4 アプリケーション横断の AI アイコン生成および類似性監査の総合時系列アーカイブ。

プロジェクトで利用・同梱しているすべてのライセンス文書およびサードパーティ監査記録は、ルート直下の **[`LICENSES/`](LICENSES/)** ディレクトリに集約されています：

- **同梱 Meteocons カラー SVG アイコン**: [MIT License (Bas Milius)](LICENSES/METEOCONS_LICENSE.txt)
- **同梱フォント**: [Roboto (SIL OFL 1.1)](LICENSES/ROBOTO_OFL.txt), [Open Sans (SIL OFL 1.1)](LICENSES/OPEN_SANS_OFL.txt), [JetBrains Mono NL (SIL OFL 1.1)](LICENSES/JETBRAINS_MONO_NL_OFL.txt), [DejaVu Serif (Bitstream Vera / DejaVu)](LICENSES/DEJAVU_LICENSE.txt)
- **サードパーティ依存クレート監査記録**: [LICENSES/THIRD_PARTY_AUDIT.md](LICENSES/THIRD_PARTY_AUDIT.md)

<p align="center">
  Crafted via <strong>AI Vibe Coding</strong> 🚀 · Built with ❤️ for Pop!_OS COSMIC & Linux Developers
</p>
