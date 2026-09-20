# Toodle

> **System76 COSMIC Desktop Environment 向け モダン・デジタルクロックウィジェット**

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-2021%20Edition-orange.svg)](https://www.rust-lang.org/)
[![COSMIC](https://img.shields.io/badge/Desktop-COSMIC-purple.svg)](https://github.com/pop-os/cosmic-epoch)
[![Wayland](https://img.shields.io/badge/Protocol-wlr--layer--shell-green.svg)](https://wayland.freedesktop.org/)

[English (英語ドキュメント)](README.md) | [アーキテクチャ設計書](docs/ARCHITECTURE.ja.md) | [機能仕様書](docs/FEATURES.ja.md) | [設計書 v0.3 (Baseline)](docs/Drafts/Toodle-Design-Docs-v0.3.md) | [引き継ぎサマリー](SESSION_HANDOVER.md)

---

## 主な特徴 (Highlights)

- **COSMIC / Wayland ネイティブ**: System76 の公式ツールキット `libcosmic` および `wlr-layer-shell` プロトコルを採用。
- **チラつきゼロのサーフェス制御**: 最背面 `Layer::Bottom` に常駐し、`content_bounds` 矩形入力領域により余白のクリックをデスクトップ壁紙やアイコンへ透過。
- **高精度サブセカンドクロック**: 次の 1 秒のマイクロ秒境界に同調する非同期ストリームにより、CPU 負荷なしに正確な 1 秒更新を実現。
- **2段階キャッシュ付き天気サブシステム**: Open-Meteo REST API を利用し、30分（現在天気）/ 3時間（週間予報）のローカル永続キャッシュ、厳格な座標検証、オフライン自動フォールバックを完備。
- **独立ポップアップサーフェス群 (`Layer::Top`)**:
  - 右クリックコンテキストメニュー（340 x 380、18px 中央配置ボタン）。
  - 月間カレンダー（680 x 720、滑らかな月送り、「今日」ハイライト）。
  - 7日間週間天気予報（680 x 720、天候アイコン、気温幅、降水確率）。
  - デスクトップ壁紙に左右されないソリッドダーク背景スタイリング。
- **2 サーフェス Edit Layout モード**: `Layer::Top` の独立 `Edit Layout Panel` と `Layer::Bottom` のウィジェット本体が連携。サーフェスの再生成を行わないインプレース更新により、チラつきゼロでリアルタイムに変形。
- **独立設定アプリ (`toodle-settings`)**: ネイティブ XDG Toplevel ウィンドウ（720 x 780、ソリッドダーク背景）。4 タブ構成（Appearance, Layout, Weather, Display）、リアルタイム自動保存、10 テーマプリセット、16 色カラーパレット、クイック都市選択。
- **inotify 設定ホットリロード**: `~/.config/toodle/config.toml` の変更を 25ms で高速検知し、ウィジェットへ即座に反映。
- **バイナリ完全組み込みフォント**: Roboto Sans, JetBrains Mono, DejaVu Serif, Open Sans の 4 ファミリ（計8ファイル）を `include_bytes!` で静的組み込み。外部システムフォント非依存。

---

## クイックスタート

### 前提条件
- Arch Linux（または Rust / Cargo が動作する最新の Linux ディストリビューション）。
- COSMIC Desktop Environment（`cosmic-comp`, `libcosmic` 依存ライブラリ: `wayland`, `libxkbcommon`）。

### ビルド

```bash
# リポジトリのクローン
git clone https://github.com/wammed/Toodle.git
cd Toodle

# ウィジェット本体および設定アプリのビルド
cargo build --release
```

### 起動

```bash
# クロックウィジェット本体の起動 (デスクトップ背景 Layer::Bottom で常駐)
./target/release/toodle &

# 設定アプリケーションの起動
./target/release/toodle-settings
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
| **Modern** | Roboto Sans Bold / Regular | `#FFFFFF` | クリーンで現代的なサンセリフスタイル |
| **Classic** | DejaVu Serif Bold / Regular | `#E2E8F0` | 重厚で上品なタイムピースクラシックスタイル |
| **Digital Mono** | JetBrains Mono Bold / Regular | `#38BDF8` | 開発者向けターミナルライクなモノスペース |
| **Minimal** | Open Sans Bold / Regular | `#94A3B8` | 控えめで洗練されたヒューマニストデザイン |
| **Cyberpunk** | JetBrains Mono Bold / Regular | `#EAB308` | ネオンイエローの高コントラストフューチャースタイル |
| **Nord** | Open Sans Bold / Regular | `#38BDF8` | 北極の冷涼さを感じるアークティックブルースタイル |
| **Warm Sunset** | DejaVu Serif Bold / Regular | `#F59E0B` | 夕暮れの温かみを感じるアンバーゴールド |
| **Forest** | Roboto Sans Bold / Regular | `#10B981` | 自然で爽やかなエメラルドグリーンスタイル |
| **Slate** | JetBrains Mono Bold / Regular | `#94A3B8` | インダストリアルで落ち着いたダークメタリック |
| **Rose Gold** | DejaVu Serif Bold / Regular | `#EC4899` | 上品なパステルローズとバイオレットのアクセント |

---

## 設定ファイル (`config.toml`)

設定は `~/.config/toodle/config.toml` に保存されます：

```toml
[display]
output = "" # 対象 Wayland 出力先（例: "DP-1"、空文字でアクティブ出力）

[layout]
anchor = "TopRight" # TopLeft, TopRight, BottomLeft, BottomRight
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
temperature_unit = "Celsius" # Celsius, Fahrenheit
```

---

## 基本操作

- **左クリック**: ウィジェットやボタンのフォーカス・操作。
- **右クリック**: コンテキストメニュー（`Calendar`, `Weekly Forecast`, `Edit Layout`, `Settings`, `Quit`）を表示。
- **Edit Layout Mode**: 独立した `Edit Layout Panel` のスライダーでマージン・サイズ・フォントスケールをリアルタイム調整し、`Save` または `Cancel`。
- **設定アプリ (`toodle-settings`)**: スライダーやテーマ、都市ボタンの操作がデスクトップ上のウィジェットへ即座にリアルタイム反映。

---

## ドキュメント & 設計 Baseline

- **[Toodle Design Document v0.3](docs/Drafts/Toodle-Design-Docs-v0.3.md)**: COSMIC 実機検証結果を反映した公式の設計基準書（Baseline）。
- **[アーキテクチャ設計書](docs/ARCHITECTURE.ja.md)**: サーフェス階層、クロックストリーム、キャッシュ仕様の詳細。
- **[機能仕様書](docs/FEATURES.ja.md)**: UI/UX 機能の詳細解説。
- **[開発引き継ぎサマリー](SESSION_HANDOVER.md)**: 全開発履歴、技術決定事項、既知のギャップ、ロードマップ。

---

## ライセンス

本プロジェクトは [MIT License](LICENSE) の下で公開されています。
