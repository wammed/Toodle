# 📚 Toodle ドキュメントポータル (Documentation Portal)

Toodle の公式ドキュメントポータルへようこそ。  
Toodle は、System76 COSMIC Desktop Environment および Linux Wayland 環境向けに開発された、高精度・堅牢・高機能なモダン・デジタルクロック＆ウェザーウィジェットです。

本ポータルでは、利用目的に応じて必要な詳細仕様やガイドへ迷わずアクセスできるように各ドキュメントを整理・体系化しています。

---

## 🧭 ドキュメント総合ナビゲーション

| ドキュメント | 主な内容 | 推奨読者 |
| :--- | :--- | :--- |
| **[クイックスタート](../README.ja.md#-クイックスタート)** | 必要環境、ビルド手順、ローカルインストール、起動方法、自動起動設定 | 初めて利用する方、インストールしたい方 |
| **[🖱️ 操作 & インタラクションガイド](SHORTCUTS.ja.md)** | マウス操作、カレンダー、天気予報、Edit Layout モード、設定アプリ | 日常の操作やレイアウト変更をマスターしたい方 |
| **[💡 詳細機能仕様書 (FEATURES)](FEATURES.ja.md)** | 全機能の完全仕様、9分割配置・10段階サイズ、天気サブシステム、テーマ | 機能詳細や設定項目の仕様を知りたい方 |
| **[📐 アーキテクチャ設計書 (ARCHITECTURE)](ARCHITECTURE.ja.md)** | マルチサーフェス構造、動的マルチモニター管理、サブセカンドクロック、非同期設計 | 内部構造や Wayland 連携の設計を知りたい方 |
| **[🛡️ 堅牢性 & セキュリティモデル (SECURITY)](SECURITY.ja.md)** | 2段階天気キャッシュ、アトミック保存、オフライン完全対応、権限・透過設計 | データの安全性やオフライン耐性を確認したい方 |
| **[🛡️ 知的財産コンプライアンス・デューデリジェンス (IP COMPLIANCE)](../IP_COMPLIANCE.ja.md)** | Toodle アイコンの由来、類似性レビュー結果、知財デューデリジェンス記録 | 全ユーザー、コントリビューター、法務・審査担当者 |
| **[🎨 アイコンデザイン・IPレビュー履歴 (ICON DESIGN HISTORY)](../ICON_DESIGN_HISTORY.ja.md)** | 4アプリ横断の AI 生成ログ、類似性監査履歴、デザイン変遷の記録 | 全ユーザー、法務・審査担当者 |
| **[📄 ライセンス & サードパーティ通知](../LICENSES/README.ja.md)** | MIT License、Meteocons、同梱フォント、依存クレート監査記録 | 法的要件やライセンス条項を確認したい方 |

---

## 🎯 目的別ガイド

### 1. まずはインストールして動かしてみたい
- [README.ja.md: クイックスタート](../README.ja.md#-クイックスタート) を参照し、`cargo build --release` でバイナリをビルドしてください。
- `$HOME/.local/bin` への配置は `./tools/install-local.sh` で簡単に行えます。
- COSMIC のログイン時に自動起動したい場合は、自動起動トラブルシューティング（`sleep 3` ワークアラウンド）をご確認ください。

### 2. ウィジェットの操作方法やレイアウト変更を覚えたい
- [SHORTCUTS.ja.md](SHORTCUTS.ja.md) をご覧ください。
- 右クリックコンテキストメニューからのカレンダー・週間天気予報表示、`Edit Layout` モードによる 9 ゾーン位置 & 10 段階サイズのリアルタイム変形、`toodle-settings` アプリでのテーマ・都市設定などを詳しく解説しています。

### 3. 表示サイズやテーマ、天気などの機能仕様を知りたい
- [FEATURES.ja.md](FEATURES.ja.md) では、WQHD 最適化された 10 段階のサイズプリセット寸法表、全 10 種類のテーマと推奨カラー、Open-Meteo REST API を利用した 2 段階キャッシュ仕様、組み込みカラーベクター天気アイコンの詳細を解説しています。

### 4. 内部設計やマルチモニター、Wayland Layer Shell の仕組みを知りたい
- [ARCHITECTURE.ja.md](ARCHITECTURE.ja.md) では、`Layer::Bottom`（ウィジェット本体）と `Layer::Top`（ポップアップ群・Edit パネル）のマルチサーフェス連携、Wayland ジオメトリ確定遅延を考慮した `OutputManager`、マイクロ秒境界同期サブセカンドクロックストリームの設計を解説しています。

### 5. キャッシュの安全性やオフライン動作、プライバシーを確認したい
- [SECURITY.ja.md](SECURITY.ja.md) では、古い非同期天気レスポンスを破棄する世代 ID 機構、クロックスキュー防御、プロセス ID 一時ファイルによるアトミック保存、および完全なオフラインフォールバックとゼロトラッキング方針を解説しています。

### 6. ライセンスやフォント・アイコンの利用規約を確認したい
- [LICENSES/README.ja.md](../LICENSES/README.ja.md) では、Toodle 本体の MIT License、Bas Milius による Meteocons カラー SVG アイコン（MIT）、同梱フォント（Roboto, Open Sans, JetBrains Mono NL, DejaVu Serif）のライセンス、および `cargo-deny` による全依存クレートのライセンス監査記録（[LICENSES/THIRD_PARTY_AUDIT.md](../LICENSES/THIRD_PARTY_AUDIT.md)）を網羅しています。

### 7. 法務・ライセンス・知的財産 (IP) プロヴェナンスを確認したい
- [ライセンス総合案内](../LICENSES/README.ja.md) でソースコード配布モデル、依存クレート管理方針、フォント取り扱いを確認してください。
- [知的財産コンプライアンス記録書](../IP_COMPLIANCE.ja.md) で Toodle アイコンの由来とデューデリジェンス内容を確認してください。
- [アイコンデザイン・IPレビュー履歴](../ICON_DESIGN_HISTORY.ja.md) で Fluffy, Waddle, Rooney, Toodle の 4 アプリを網羅した詳細な時系列監査ログを通読できます。

---

<p align="center">
  <a href="../README.ja.md">← ルート README (日本語) に戻る</a> | <a href="PORTAL.md">English Portal →</a>
</p>
