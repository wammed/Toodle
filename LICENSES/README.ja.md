# ライセンスおよびサードパーティ通知 (Licensing & Third-Party Notice)

<p align="center">
  <a href="README.md">English</a> | <strong>日本語</strong> | <a href="../README.ja.md">← ルート README</a>
</p>

---

本書は、**Toodle** プロジェクトにおけるライセンスポリシー、ソースコード配布モデル、サードパーティ製依存関係の管理方針、および同梱フォント・アイコンアセットのライセンスについて包括的に説明するドキュメントです。本ディレクトリ（`LICENSES/`）には、プロジェクトで利用・同梱している各コンポーネントのライセンス本文および監査記録が集約されています。

---

## 1. プロジェクトライセンス (Toodle)

**Toodle** 本体のソースコードは、**MIT License** のもとで公開されています。

- ライセンス全文はリポジトリルートの [LICENSE](../LICENSE)（同一内容: [LICENSE-MIT.txt](LICENSE-MIT.txt)）に記載されています。
- Copyright (c) 2026 wammed
- 著作権表示および許諾表示を保持する限り、商用・非商用を問わず、複製、改変、再配布、サブライセンス付与、および派生著作物の作成を自由に行うことができます。

---

## 2. 配布およびビルドモデル (Distribution & Build Model)

### ソースコード配布とビルド方針

Toodle は、本リポジトリを通じて**ソースコード形式**で配布されます。

- **GitHub 上でのコンパイル済みバイナリ非配布**: 本プロジェクトでは、GitHub を通じたコンパイル済み実行可能バイナリ、静的ライブラリ、およびインストーラーパッケージの配布を行いません。ユーザーおよび各ディストリビューションのパッケージメンテナーが、それぞれの環境に合わせて Toodle をビルド・パッケージングすることを想定しています。
- **意図的な配布方針の背景**: これはプロジェクト運用上の意図的な方針です。あらかじめコンパイルしたバイナリを配布する場合、バイナリに含まれる第三者ライブラリ、埋め込みフォント、ベクターアイコン、その他各種アセットについて、再配布に必要なライセンス表示や条件をプロジェクト側で個別に管理・提示する必要があります。GitHub 上ではソースコードの提供に徹することで、これらライセンス表示や再配布条件の管理が不必要に煩雑化することを避け、下流ディストリビューターやパッケージャーが自身の配布環境に適した形でパッケージングとライセンス要件を満たせるようにしています。
- **方針の位置付け**: これは Toodle プロジェクト自身の配布ポリシーであり、MIT License や Toodle が採用している第三者コンポーネントのライセンスによってバイナリ配布が法的に禁止されていることを意味するものではありません。
- **ローカルビルドとローカルインストール**:
  - **コンパイル**: 利用者は標準の Cargo コマンドを用いてローカル環境でソースコードからビルドします：
    ```bash
    cargo build --release
    ```
    これにより、ユーザーのシステムやホームディレクトリを変更することなく、`target/release/` 配下に `toodle` および `toodle-settings` バイナリが生成されます。
  - **ローカルインストール補助ツール**: ビルドされたバイナリを `$HOME/.local/bin` に配置したいユーザーのために、補助スクリプト [`tools/install-local.sh`](../tools/install-local.sh) を提供しています。このスクリプトはユーザー自身のローカル環境でのインストールを補助するものであり、GitHub Releases 等のバイナリ配布機構ではありません。
- **公式レジストリからの直接取得**: ビルド時、すべての Rust 依存クレートは公式パッケージレジストリ（[crates.io](https://crates.io/)）または指定の公式 Git リポジトリ（`libcosmic` 等）から直接ダウンロードされます。本リポジトリ内に外部クレートのソースコードをベンダー同梱（Vendor）して再配布することはありません。

---

## 3. サードパーティ製依存関係とライセンス適合性 (Third-Party Dependencies & Compliance)

### パーミッシブおよびウィークコピーレフト方針
下流でのビルドや利用におけるライセンス上の制約（不要な感染性）を排除するため、Toodle の全依存クレートは [`deny.toml`](../deny.toml) にて厳格に管理されています：

- **許可されているライセンス**: パーミッシブ（寛容型）オープンソースライセンス、および動的リンク境界が明確なウィークコピーレフトライセンス（COSMIC デスクトップライブラリの MPL-2.0 など）のみを許可しています：
  - `0BSD`
  - `Apache-2.0` / `Apache-2.0 WITH LLVM-exception`
  - `BSD-2-Clause` / `BSD-3-Clause`
  - `BSL-1.0` (Boost Software License)
  - `CC0-1.0`
  - `ISC`
  - `MIT`
  - `MPL-2.0` (Mozilla Public License 2.0)
  - `Unicode-3.0`
  - `Unlicense`
  - `Zlib`
- **強力なコピーレフトライセンスの排除**: GPL-2.0、GPL-3.0、AGPL などの強力なコピーレフトライセンスを持つクレートは、依存ツリーから意図的に排除されています。

### 自動ライセンス監査
Toodle では [`cargo-deny`](https://github.com/EmbarkStudios/cargo-deny) を用いてライセンスの自動監査を行っています：
- ローカル検証コマンド:
  ```bash
  cargo deny check licenses
  ```
- 上流パッケージのマニフェストメタデータに不備があるクレートについては、その調査結果と根拠を [`THIRD_PARTY_AUDIT.md`](THIRD_PARTY_AUDIT.md) に記録・管理しています。

### システム共有ライブラリの動的リンク
Toodle は COSMIC / Wayland ネイティブウィジェットとして動作し、ホスト OS のシステム共有ライブラリを利用します：
- `wayland-client` や `libxkbcommon` などのライブラリは、ユーザーのホスト OS が提供する共有ライブラリ（`.so`）として実行時に動的リンクされます。
- これらを本リポジトリ内に同梱・静的リンクすることはありません。

---

## 4. フォントおよびビジュアルアセット方針 (Fonts & Visual Assets)

一般的なシステムフォントのみに依存するアプリケーションとは異なり、Toodle ではあらゆる Linux ディストリビューション上でフォントメトリクスやデザインの完全な同一性と再現性を担保するため、高品質なフォントおよび天候アイコンアセットを Rust の `include_bytes!` によりバイナリへ静的組み込みしています。

同梱されている各大手第三者アセットは、それぞれの元ライセンスを完全に維持しています：

### 同梱フォント (`resources/fonts/`)

| フォント名 | ファイル | 著作者 / 著作権表示 | 上流ライセンス | ライセンス文書 |
| :--- | :--- | :--- | :--- | :--- |
| **Roboto** | `Roboto-Regular.ttf`<br>`Roboto-Bold.ttf` | Christian Robertson, Google LLC | SIL Open Font License 1.1 | [ROBOTO_OFL.txt](ROBOTO_OFL.txt) |
| **Open Sans** | `OpenSans-Regular.ttf`<br>`OpenSans-Bold.ttf` | Steve Matteson, Monotype Design Team | SIL Open Font License 1.1 | [OPEN_SANS_OFL.txt](OPEN_SANS_OFL.txt) |
| **JetBrains Mono NL** | `JetBrainsMono-Regular.ttf`<br>`JetBrainsMono-Bold.ttf` | JetBrains s.r.o., Philipp Nurullin, Konstantin Bulenkov | SIL Open Font License 1.1 | [JETBRAINS_MONO_NL_OFL.txt](JETBRAINS_MONO_NL_OFL.txt) |
| **DejaVu Serif** | `DejaVuSerif-Regular.ttf`<br>`DejaVuSerif-Bold.ttf` | DejaVu fonts team, Bitstream Inc., Tavmjong Bah | Bitstream Vera / DejaVu Fonts License | [DEJAVU_LICENSE.txt](DEJAVU_LICENSE.txt) |

### 天候アイコンおよびビジュアルアセット

- **Meteocons (天候カラー SVG アイコン)**:
  - 格納先: `resources/icons/meteocons/`
  - 作者: Bas Milius
  - ライセンス: **MIT License** (Copyright (c) 2020-present Bas Milius)
  - ライセンス文書: [METEOCONS_LICENSE.txt](METEOCONS_LICENSE.txt)
  - *備考*: Toodle 本体と Meteocons はともに MIT License ですが、異なる著作者による個別の著作物です。
- **アプリアイコン & デスクトップエントリ**:
  - 格納先: `resources/icons/toodle*.png`, `resources/icons/*.svg`, `resources/*.desktop`
  - Toodle 用に独自作成されたオリジナルアセットおよびメタデータであり、本プロジェクトの **MIT License** が適用されます。

---

## 5. ライセンス関連ファイル一覧 (`LICENSES/`)

本ディレクトリに集約されているライセンス文書の一覧です：

```text
LICENSES/
├── README.ja.md                  # 本ドキュメント (日本語版総合案内)
├── README.md                     # 英語版総合案内 (English Notice)
├── LICENSE-MIT.txt               # Toodle 本体 MIT License 全文
├── DEJAVU_LICENSE.txt            # Bitstream Vera / DejaVu Fonts License 本文
├── JETBRAINS_MONO_NL_OFL.txt     # SIL Open Font License 1.1 本文
├── METEOCONS_LICENSE.txt         # MIT License (Bas Milius) 本文
├── OPEN_SANS_OFL.txt             # SIL Open Font License 1.1 本文
├── ROBOTO_OFL.txt                # SIL Open Font License 1.1 本文
└── THIRD_PARTY_AUDIT.md          # 上流クレートのライセンス調査・監査記録 (cargo-deny)
```

---

## 6. パッケージ作成時の依存ライセンス抽出方法 (How to Extract Dependency Licenses for Packaging)

利用者が Toodle を再配布可能なパッケージ（Arch Linux AUR PKGBUILD, Debian `.deb`, Fedora `.rpm`, Flatpak 等）としてビルドし、サードパーティ製ライセンス一覧を一括同梱する必要がある場合は、以下のオープンソースツールを利用できます：

```bash
# cargo-about を使用して HTML / Markdown 形式の統合ライセンスレポートを生成
cargo install cargo-about
cargo about generate about.hbs > THIRD_PARTY_LICENSES_RUST.html

# または、すべての生のライセンスファイルを一括収集する場合:
cargo install cargo-bundle-licenses
cargo bundle-licenses --format yaml --output THIRDPARTY.yaml
```

---

## サマリー (Summary)

| 構成要素 | ライセンス / 適用方針 | 補足 |
| :--- | :--- | :--- |
| **Toodle 本体のソースコード** | MIT License | [LICENSE](../LICENSE) / [LICENSE-MIT.txt](LICENSE-MIT.txt) 参照 |
| **Rust 依存クレート** | パーミッシブ / ウィークコピーレフト | [deny.toml](../deny.toml) で強制検証 |
| **システム共有ライブラリ** | システム提供 (Wayland, libxkbcommon) | ユーザーのホスト OS より動的リンク |
| **Meteocons** | MIT License (Bas Milius) | [METEOCONS_LICENSE.txt](METEOCONS_LICENSE.txt) |
| **Roboto** | SIL Open Font License 1.1 | [ROBOTO_OFL.txt](ROBOTO_OFL.txt) |
| **Open Sans** | SIL Open Font License 1.1 | [OPEN_SANS_OFL.txt](OPEN_SANS_OFL.txt) |
| **JetBrains Mono NL** | SIL Open Font License 1.1 | [JETBRAINS_MONO_NL_OFL.txt](JETBRAINS_MONO_NL_OFL.txt) |
| **DejaVu Serif** | Bitstream Vera / DejaVu License | [DEJAVU_LICENSE.txt](DEJAVU_LICENSE.txt) |
| **アプリアイコン & アセット** | MIT License | `resources/icons/`, `resources/*.desktop` |

---

<p align="center">
  <a href="../README.ja.md">← ルート README (日本語) に戻る</a> | <a href="README.md">English Notice →</a>
</p>
