# Toodle

> **System76 COSMIC Desktop Environment 向け モダン・デジタルクロックウィジェット**

![Banner](images/toodle-banner.svg)

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-2021%20Edition-orange.svg)](https://www.rust-lang.org/)
[![COSMIC](https://img.shields.io/badge/Desktop-COSMIC-purple.svg)](https://github.com/pop-os/cosmic-epoch)
[![Wayland](https://img.shields.io/badge/Protocol-wlr--layer--shell-green.svg)](https://wayland.freedesktop.org/)

[English (英語ドキュメント)](README.md) | [ライセンス通知](LICENSES.ja.md) | [アーキテクチャ設計書](docs/ARCHITECTURE.ja.md) | [機能仕様書](docs/FEATURES.ja.md) | [設計書 v0.3 (Baseline)](docs/Drafts/Toodle-Design-Docs-v0.3.md) | [引き継ぎサマリー](SESSION_HANDOVER.md)

---

## 主な特徴 (Highlights)

- **COSMIC / Wayland ネイティブ**: System76 の公式ツールキット `libcosmic` および `wlr-layer-shell` プロトコルを採用。
- **動的マルチモニター管理 (`OutputManager`)**: Wayland コンポジタのメタデータから出力先を自動管理し、指定されたディスプレイ（例: `"DP-1"`）へシームレスにバインド。切断時や未設定時はアクティブ/プライマリ画面へ安全にフォールバックし、画面ごとの個別解像度管理、ジオメトリ確定までの再生成遅延、およびホットプラグ再接続に対応。
- **高精度サブセカンドクロック**: 次の 1 秒のマイクロ秒境界に同調する非同期ストリームにより、CPU 負荷なしに正確な 1 秒更新を実現。
- **堅牢な2段階キャッシュ付き天気サブシステム**: Open-Meteo REST API を利用し、30分（現在天気）/ 3時間（週間予報）のローカル永続キャッシュ、リクエスト世代IDによる古い非同期レスポンスの破棄、起動時キャッシュの座標＆有効期限検証、厳格な JSON レスポンス検証（最低7日分の配列充足確認）、座標範囲検証、クロックスキュー保護、プロセスID一時ファイルによる並行書き込み安全性、オフライン自動フォールバックを完備。
- **独立ポップアップサーフェス群 (`Layer::Top`)**:
  - 右クリックコンテキストメニュー（340 x 380、18px 中央配置ボタン）。
  - 月間カレンダー（680 x 720、滑らかな月送り、「今日」ハイライト）。
  - 7日間週間天気予報（680 x 720、天候アイコン、気温幅、降水確率）。
  - デスクトップ壁紙に左右されないソリッドダーク背景スタイリング。
- **9分割グリッド配置 & 10段階固定サイズ（WQHD対応）**: ディスプレイを 3×3（TopLeft〜BottomRight）に分割する直感的な配置と、WQHD まで最適化された 10 段階のサイズプリセット（280px〜2060px）。フォントサイズ（時刻・日付）およびベクター天気アイコンがウィンドウ寸法に完全連動し、十分なヘッドルーム設計により文字や天候アイコンの下部見切れを完全防止。
- **2 サーフェス Edit Layout モード**: `Layer::Top` の独立 `Edit Layout Panel`（9分割位置ボタン・10段階サイズ選択ボタン）と `Layer::Bottom` のウィジェット本体が連携。インプレース Layer Command により、チラつきゼロでリアルタイムに変形。
- **独立設定アプリ (`toodle-settings`)**: ネイティブ XDG Toplevel ウィンドウ（720 x 780、ソリッドダーク背景）。4 タブ構成（Appearance, Layout, Weather, Display）、エラー通知付きリアルタイム自動保存、10 テーマプリセット、16 色カラーパレット、9分割配置・10段階サイズセレクタ、クイック都市選択。
- **明確に分離されたビルド・インストール構造**: `cargo build --release` はバイナリ生成のみを担当し、ユーザー環境（`$HOME/.local/bin`）への反映は明示的な `./tools/install-local.sh` に分離。将来のディストリビューションパッケージング（.deb, RPM, AUR 等）とも整合。
- **inotify 設定ホットリロード & マイグレーション**: `~/.config/toodle/config.toml` の変更を 50ms イベント集約（event coalescing）で高速検知し、ウィジェットへ即座に反映。旧形式の設定ファイルは読み込み時に自動マイグレーションされ、新形式でディスクに自動再保存。編集中（Edit モード）の作業内容は予期せぬ外部リロードから保護。
- **完全組み込みフォント & カラーベクター天気アイコン**: Roboto, JetBrains Mono, DejaVu Serif, Open Sans を `include_bytes!` で静的組み込み。天気アイコンは従来のモノクロフォントや Unicode 絵文字に代わり、同梱 Bas Milius Meteocons カラー SVG アセット（`resources/icons/meteocons/`、MIT License）で描画され、外部システム非依存でどの環境・表示スケールでも鮮やかで美しいモダンなアイコンを表示。

---

## クイックスタート

### 前提条件
- Arch Linux（または Rust / Cargo が動作する最新の Linux ディストリビューション）。
- COSMIC Desktop Environment（`cosmic-comp`, `libcosmic` 依存ライブラリ: `wayland`, `libxkbcommon`）。

### ビルド

Toodle はソースコード形式で配布されます。GitHub Releases によるコンパイル済みバイナリの配布は行っていません。ソースからローカル環境でビルドしてください：

```bash
# リポジトリのクローン
git clone https://github.com/wammed/Toodle.git
cd Toodle

# ウィジェット本体および設定アプリのビルド (ビルドのみ、$HOME/.local/bin へのインストールは行われません)
cargo build --release
```

### ローカルインストール

リリースバイナリ（`toodle`, `toodle-settings`）を `$HOME/.local/bin` にインストールする場合：

```bash
./tools/install-local.sh
```

### 起動

```bash
# クロックウィジェット本体の起動 (デスクトップ背景 Layer::Bottom で常駐)
./target/release/toodle &
# またはインストール済みの場合:
# toodle &

# 設定アプリケーションの起動
./target/release/toodle-settings
# またはインストール済みの場合:
# toodle-settings
```

#### 自動起動（Autostart）時の注意点・トラブルシューティング

COSMIC デスクトップ環境の自動起動（設定アプリの「自動起動」や `$HOME/.config/autostart/*.desktop`）に登録する場合、COSMIC のログインセッション初期化と Toodle の Layer Surface / Input Region 初期化のタイミングによって、起動直後に**右クリックが反応しなくなる挙動**が発生する場合があります。現時点では正確な根本原因は未確定ですが、デスクトップ背景等の初期化順序との競合の可能性が考えられます。

現在の実機環境で確認済みの回避策は、XDG Autostart の起動コマンドに 3 秒程度の遅延を設定する方法です（現在の実機環境では 3 秒程度の遅延によって問題が解消することを確認しています。systemd user service 等による起動順序制御は、将来的な改善・代替候補です）：

```desktop
# 実機確認済みワークアラウンド（例: ~/.config/autostart/com.github.wammed.toodle.desktop の Exec 行）
Exec=sh -c "sleep 3 && toodle"
# またはフルパス指定:
# Exec=sh -c "sleep 3 && $HOME/.local/bin/toodle"
```

---

## アーキテクチャ概要

```text
┌─────────────────────────────────────────────────────────────┐
│                       COSMIC Desktop                        │
│                                                             │
│   Layer::Top 独立サーフェス群:                                │
│   ┌───────────────┐ ┌───────────────┐ ┌──────────────────┐  │
│   │ Context Menu  │ │   Calendar    │ │ Weekly Forecast  │  │
│   │   (340x380)   │ │   (680x720)   │ │    (680x720)     │  │
│   └───────▲───────┘ └───────▲───────┘ └────────▲─────────┘  │
│           │                 │                  │            │
│           └─────────────────┼──────────────────┘            │
│                             │ Edit Layout Panel (Layer::Top)│
│                             ▼                               │
│   Layer::Bottom ウィジェット本体:                            │
│   ┌──────────────────────────────────────────────────────┐  │
│   │ Toodle Main Widget                                   │  │
│   │ [12:34:56]  [Monday, Sep 20, 2026]  [Sunny 22°C]     │  │
│   │ (content_bounds 矩形入力領域: 背景クリック透過)         │  │
│   └─────────────────────────▲────────────────────────────┘  │
│                             │ inotify 監視 (~25ms)          │
│               ~/.config/toodle/config.toml                  │
│                             ▲                               │
│   XDG Toplevel ウィンドウ:   │ アトミック保存 (.tmp -> rename)│
│   ┌─────────────────────────┴────────────────────────────┐  │
│   │ toodle-settings (Appearance, Layout, Weather, Disp)  │  │
│   └──────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

---

## テーマプリセット & 組み込みフォント

| テーマ名 | フォントファミリ | デフォルト推奨色 | スタイル解説 |
|---|---|---|---|
| **Modern** | Roboto Bold / Regular | `#FFFFFF` | クリーンで現代的なサンセリフスタイル |
| **Classic** | DejaVu Serif Bold / Regular | `#E2E8F0` | 重厚で上品なタイムピースクラシックスタイル |
| **Digital Mono** | JetBrains Mono Bold / Regular | `#38BDF8` | 開発者向けターミナルライクなモノスペース |
| **Minimal** | Open Sans Bold / Regular | `#94A3B8` | 控えめで洗練されたヒューマニストデザイン |
| **Cyberpunk** | JetBrains Mono Bold / Regular | `#EAB308` | ネオンイエローの高コントラストフューチャースタイル |
| **Nord** | Open Sans Bold / Regular | `#38BDF8` | 北極の冷涼さを感じるアークティックブルースタイル |
| **Warm Sunset** | DejaVu Serif Bold / Regular | `#F59E0B` | 夕暮れの温かみを感じるアンバーゴールド |
| **Forest** | Roboto Bold / Regular | `#10B981` | 自然で爽やかなエメラルドグリーンスタイル |
| **Slate** | JetBrains Mono Bold / Regular | `#94A3B8` | インダストリアルで落ち着いたダークメタリック |
| **Rose Gold** | DejaVu Serif Bold / Regular | `#EC4899` | 上品なパステルローズとバイオレットのアクセント |

---

## 設定ファイル (`config.toml`)

設定は `~/.config/toodle/config.toml` に保存されます：

```toml
[display]
output = "" # 対象 Wayland 出力先（例: "DP-1"、空文字でプライマリ/有効な画面）

[layout]
grid_position = "TopRight" # 9ゾーン: TopLeft, TopCenter, TopRight, MiddleLeft, Center, MiddleRight, BottomLeft, BottomCenter, BottomRight
size_stage = 2            # 10段階: 1 (Compact 280x130) 〜 10 (Max WQHD 2060x920)

[appearance]
theme = "Modern"
color = "#FFFFFF"
text_shadow = true

[weather]
location_name = "Tokyo, Japan"
latitude = 35.6895
longitude = 139.6917
temperature_unit = "Celsius" # Celsius, Fahrenheit
```

> **設定の自動マイグレーション**:
> 旧バージョンの設定ファイル（`anchor`, `margin_x`, `margin_y`, `width`, `height`, `font_scale` 等）が残っている場合でも、読み込み時に自動的かつ確定的に最も近い `grid_position` および `size_stage` へマイグレーションされ、クリーンな新仕様フォーマットでディスクに即時再保存されます。

> **ディスプレイおよびマルチモニター環境の実機検証状況**:
> 単一ディスプレイにおける各解像度（640×360 から WQHD 2560×1440、4K 3840×2160 まで）のジオメトリ計算は自動単体テストにより検証済みです。実機環境においては、Wayland 環境下の同一機種 2560×1440 × 2（`DP-1` および `DP-2`）において、出力先の切り替え、表示位置、表示サイズが正しく動作することを実機にて検証完了しています。異なる解像度の混在（例: 1080p + 4K）、マルチモニターでの mixed DPI、および分数スケーリング（fractional scaling）については実機検証未実施であり、将来の堅牢化課題として位置付けられます。

---

## 基本操作

- **左クリック**: カレンダーの日付切り替えや各種ボタンの操作。
- **右クリック**: コンテキストメニュー（`Calendar`, `Weekly Forecast`, `Edit Layout`, `Settings`, `Quit`）を表示。
- **Edit Layout Mode**: 独立した `Edit Layout Panel` の 3×3 グリッドボタンと 10 段階のサイズボタンにより、Wayland サーフェスを破棄・再生成せずインプレースで瞬時にレイアウトを変更、`Save` または `Cancel`。
- **設定アプリ (`toodle-settings`)**: テーマプリセット、16色パレット、都市プリセットの変更がデスクトップ上のウィジェットへ即座にリアルタイム反映。

---

## 開発について

Toodle は、AIを活用した **Vibe Coding** によって開発されています。アーキテクチャの検討、実装、リファクタリング、テスト、ドキュメント作成、コードレビューなどの開発工程でAIを積極的に活用しています。生成されたコードや提案をそのまま採用するのではなく、実際の動作確認やレビューを行いながら開発しています。

---

## ドキュメント & 設計 Baseline

- **[Toodle Design Document v0.3](docs/Drafts/Toodle-Design-Docs-v0.3.md)**: COSMIC 実機検証結果を反映した公式の設計基準書（Baseline）。
- **[アーキテクチャ設計書](docs/ARCHITECTURE.ja.md)**: サーフェス階層、クロックストリーム、キャッシュ仕様の詳細。
- **[機能仕様書](docs/FEATURES.ja.md)**: UI/UX 機能の詳細解説。
- **[開発引き継ぎサマリー](SESSION_HANDOVER.md)**: 全開発履歴、技術決定事項、既知のギャップ、ロードマップ。

---

## ライセンス

本プロジェクトは [MIT License](LICENSE) の下で公開されています。
同梱の Meteocons カラー SVG アイコン（作者: Bas Milius）は [MIT License](THIRD_PARTY_LICENSES/METEOCONS_LICENSE.txt) の下でライセンスされています。
同梱フォントは各フォント独自の上流ライセンス（SIL Open Font License 1.1 / Bitstream Vera & DejaVu License）が適用されます。
ライセンス体系、ソースコード配布方針、第三者アセット通知の詳細は **[LICENSES.ja.md](LICENSES.ja.md)** および [THIRD_PARTY_LICENSES/README.md](THIRD_PARTY_LICENSES/README.md) を参照してください。

