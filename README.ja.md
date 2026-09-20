# Toodle

> **System76 COSMIC Desktop Environment 向け モダン・デジタルクロックウィジェット**

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-2021%20Edition-orange.svg)](https://www.rust-lang.org/)
[![COSMIC](https://img.shields.io/badge/Desktop-COSMIC-purple.svg)](https://github.com/pop-os/cosmic-epoch)
[![Wayland](https://img.shields.io/badge/Protocol-wlr--layer--shell-green.svg)](https://wayland.freedesktop.org/)

[English Documentation](README.md) | [アーキテクチャ設計](docs/ARCHITECTURE.ja.md) | [機能仕様](docs/FEATURES.ja.md) | [引き継ぎサマリー](SESSION_HANDOVER.md)

---

## 主な特徴 (Highlights)

- **Wayland ネイティブ対応**: System76 の次世代デスクトップ COSMIC 向けに、`libcosmic` および `wlr-layer-shell` を用いて開発。
- **チラつきゼロのサーフェス制御**: ウィジェット本体は常に通常ウィンドウの背面（`Layer::Bottom`）に常駐。非矩形入力領域（`set_input_zone`）により、時計・文字以外の透明領域のクリックはデスクトップ壁紙やアイコンへ透過。
- **高精度サブセカンド同期時計**: 次の1秒のマイクロ秒境界に高精度同調する非同期ストリームにより、滑らかで正確な時刻更新を実現。
- **2段階キャッシュ付き天気予報**: Open-Meteo REST API（無料・登録不要）と連携し、現在天気と7日間予報を表示。30分（現在天気）/ 3時間（予報）の永続JSONキャッシュとオフライン時自動フォールバックを完備。
- **インタラクティブなポップアップ（`Layer::Top`）**:
  - 右クリックコンテキストメニュー（340 x 380、18pxフォント、中央配置）。
  - 月間カレンダー（680 x 720、スムーズな月送りナビゲーション、「今日」ハイライト）。
  - 7日間週間天気予報（680 x 720、天候アイコン、最高/最低気温幅、降水確率）。
- **デスクトップ上での直接レイアウト編集 (Edit Layout Mode)**: 右クリックメニューから直接編集モードへ移行し、デスクトップ上でバウンディング枠を見ながらマージン・サイズ・フォント倍率を直感的に調整可能。
- **独立した専用設定アプリ (`toodle-settings`)**: XDG Toplevel ウィンドウ（720 x 780）として動作する設定アプリ。Appearance、Layout、Weather、Display の4タブを備え、スライダー操作やカラー選択が即座にデスクトップウィジェットへリアルタイム反映。
- **inotify 連動ホットリロード**: `~/.config/toodle/config.toml` の変更をバックグラウンドスレッドで常時監視（応答速度 25ms）。設定アプリからの変更をウィジェットの再起動なしにリアルタイム反映。
- **100% バイナリ組み込みフォント**: Roboto Sans, JetBrains Mono, DejaVu Serif, Open Sans の 4 フォントファミリ（計8ファイル）を `include_bytes!` でバイナリ内へ静的組み込み。外部システムフォントの有無に依存せず、常に同一の正確なデザインで描画。

---

## クイックスタート

### 前提要件
- Arch Linux（または Rust / Cargo が動作する最新の Linux 環境）。
- COSMIC Desktop Environment（`cosmic-comp`, `libcosmic`, `wayland`, `libxkbcommon`）。

### ビルド

```bash
# リポジトリのクローン
git clone https://github.com/wammed/Toodle.git
cd Toodle

# toodle ウィジェットおよび toodle-settings のビルド
cargo build --release
```

### 起動方法

```bash
# デスクトップクロックウィジェット本体の起動 (デスクトップ背景 Layer::Bottom に常駐)
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
│   Layer::Top ポップアップ:                                  │
│   ┌───────────────┐ ┌───────────────┐ ┌──────────────────┐  │
│   │ Context Menu  │ │   Calendar    │ │ Weekly Forecast  │  │
│   │   (340x380)   │ │   (680x720)   │ │    (680x720)     │  │
│   └───────▲───────┘ └───────▲───────┘ └────────▲─────────┘  │
│           │                 │                  │            │
│   Layer::Bottom ウィジェット:│                  │            │
│   ┌─────────────────────────┴──────────────────┴─────────┐  │
│   │ Toodle Main Widget                                   │  │
│   │ [12:34:56]  [Monday, Sep 20, 2026]  [Sunny 22°C]     │  │
│   │ (非矩形入力領域: 背景クリックは壁紙へ透過)             │  │
│   └─────────────────────────▲────────────────────────────┘  │
│                             │ inotify 監視 (~25ms)          │
│               ~/.config/toodle/config.toml                  │
│                             ▲                               │
│   XDG Toplevel 設定ウィンドウ:│ アトミック保存 (.tmp -> rename)│
│   ┌─────────────────────────┴────────────────────────────┐  │
│   │ toodle-settings (Appearance, Layout, Weather, Disp)  │  │
│   └──────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

---

## テーマプリセットとフォント一覧

| テーマ名 | 採用フォント | デフォルト色 | スタイル概要 |
|---|---|---|---|
| **Modern** | Roboto Sans Bold / Regular | `#FFFFFF` | クリーンで現代的なフラットデザイン |
| **Classic** | DejaVu Serif Bold / Regular | `#E2E8F0` | 高級時計のような重厚でクラシックな佇まい |
| **Digital Mono** | JetBrains Mono Bold / Regular | `#38BDF8` | 開発者に馴染むシャープなモノスペース端末風 |
| **Minimal** | Open Sans Bold / Regular | `#94A3B8` | 控えめで邪魔にならないヒューマニストサンセリフ |
| **Cyberpunk** | JetBrains Mono Bold / Regular | `#EAB308` | 高コントラストで未来的なサイバーネオン風 |
| **Nord** | Open Sans Bold / Regular | `#38BDF8` | 北極の冷涼なブルーとスレートの組み合わせ |
| **Warm Sunset** | DejaVu Serif Bold / Regular | `#F59E0B` | 夕暮れの温かみを感じるアンバーゴールド |
| **Forest** | Roboto Sans Bold / Regular | `#10B981` | 自然で爽やかなエメラルドグリーンスタイル |
| **Slate** | JetBrains Mono Bold / Regular | `#94A3B8` | インダストリアルで落ち着いたダークメタリック |
| **Rose Gold** | DejaVu Serif Bold / Regular | `#EC4899` | 上品なパステルローズとバイオレットのアクセント |

---

## 設定ファイル (`config.toml`)

設定は `~/.config/toodle/config.toml` に保存されます：

```toml
[display]
output = "" # 空文字でプライマリ/アクティブディスプレイ、または "DP-1" 等を指定

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

## 操作方法

- **左クリック**: ウィジェットやボタンのフォーカス・操作。
- **右クリック**: コンテキストメニュー（`Calendar`, `Weekly Forecast`, `Edit Layout`, `Settings`, `Quit`）を表示。
- **Edit Layout Mode**: スライダーでマージン・サイズを直接ドラッグ調整し、`Save` または `Cancel`。
- **設定アプリ (`toodle-settings`)**: スライダーやテーマ、都市ボタンの操作がデスクトップ上のウィジェットへ即座にリアルタイム反映。

---

## ライセンス

本プロジェクトは [MIT License](LICENSE) の下で公開されています。
