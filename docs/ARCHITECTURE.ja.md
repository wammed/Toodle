# Toodle アーキテクチャ & 技術設計書

本ドキュメントは、System76 COSMIC Desktop Environment 向けデジタルクロックウィジェット **Toodle** のシステムアーキテクチャ、Wayland layer-shell プロトコル制御、高精度クロックパイプライン、天気2段階キャッシュ、inotify 設定同期、および組み込みタイポグラフィ基盤に関する詳細な技術解説を提供します。

---

## 1. システム全体構成

Toodle は、Wayland コンポジタ `cosmic-comp` 上で動作するよう設計されており、System76 の公式ツールキット `libcosmic`（Wayland layer-shell 拡張を備えた `iced` ベース）を採用しています。

システムは以下の 2 つの独立バイナリで構成されます：
1. **`toodle` (ウィジェットデーモン & Layer Shell ポップアップ)**:
   - デスクトップ背景上の `Layer::Bottom` サーフェスとして常駐する軽量デーモン。
   - 必要に応じて `Layer::Top` ポップアップ（コンテキストメニュー、月間カレンダー、7日間週間予報、Edit Layout モード）を最前面へ展開。
   - `~/.config/toodle/config.toml` を inotify で常時監視し、チラつきゼロでリアルタイムホットリロードを実行。
2. **`toodle-settings` (XDG Toplevel 設定アプリ)**:
   - 独立して、またはウィジェットの右クリックメニューから起動可能なデスクトップ設定アプリケーション。
   - Appearance、Layout、Weather、Display の 4 カテゴリタブを提供。
   - スライダー操作や色選択をアトミック保存し、起動中の `toodle` ウィジェットへリアルタイム同期。

---

## 2. Wayland サーフェス階層アーキテクチャ (`wlr-layer-shell`)

```text
┌───────────────────────────────────────────────────────────────┐
│ Wayland Output (例: DP-1: 2560 x 1440)                        │
│                                                               │
│ [ Layer::Overlay ]                                            │
│   (画面ロック、スクリーンキャプチャオーバーレイ等)               │
│                                                               │
│ [ Layer::Top ]                                                │
│   - Toodle Context Menu (340 x 380)                           │
│   - Toodle Monthly Calendar (680 x 720)                       │
│   - Toodle Weekly Forecast (680 x 720)                        │
│   - Toodle Edit Layout Mode オーバーレイ操作枠                 │
│                                                               │
│ [ 通常ウィンドウ層 (XDG Toplevel) ]                            │
│   - ブラウザ、端末、ファイルマネージャ等                        │
│   - toodle-settings 設定ウィンドウ (720 x 780)                 │
│                                                               │
│ [ Layer::Bottom ]                                             │
│   - Toodle クロックウィジェット本体 (280 x 140)                │
│     * アンカー: TopRight (設定可能)                            │
│     * 非矩形入力領域 (透明部分はデスクトップへクリック透過)      │
│                                                               │
│ [ Layer::Background ]                                         │
│   - COSMIC 壁紙                                               │
└───────────────────────────────────────────────────────────────┘
```

### 2.1 レイヤー階層の選定理由
- **ウィジェット本体 (`Layer::Bottom`)**: 通常のアプリケーションウィンドウ（ブラウザやエディタ等）を最大化または重ねた際にウィジェットの上に被さるため、作業の邪魔になりません。空のワークスペースでは常に壁紙上に整然と表示されます。
- **ポップアップ群 (`Layer::Top`)**: 右クリックメニューやカレンダー、週間予報は、一般のアプリケーションウィンドウよりも手前の最前面に浮き上がり、操作が遮られないよう配置されます。
- **排他領域の非確保 (`exclusive_zone(0)`)**: ウィジェットは排他領域を持たない（0px）ため、パネルを押し退けたり他のアプリのワークスペースを狭めたりしません。

### 2.2 非矩形入力領域（クリック透過）の実装
Wayland デスクトップウィジェットで頻発する問題が「透明なウィンドウ領域がクリックを遮断し、背後の壁紙メニューやデスクトップアイコンがクリックできなくなる」現象です。

Toodle は `cosmic::iced::wayland::layer_cmd::set_input_zone` を用いてこれを完全に解決しています：
```rust
let padding = 12.0;
let estimated_w = (180.0 * config.appearance.font_scale + padding * 2.0).min(config.layout.width as f32);
let estimated_h = (90.0 * config.appearance.font_scale + padding * 2.0).min(config.layout.height as f32);
Rectangle { x: 0.0, y: 0.0, width: estimated_w, height: estimated_h }
```
文字が表示されている実描画領域のみに入力判定（クリック受付）を限定し、それ以外の透明マージン部分はコンポジタによって下の壁紙やデスクトップへ素通し（パススルー）されます。

### 2.3 チラつきゼロのインプレースサーフェス更新
設定変更やレイアウト編集モードの開始・終了時にサーフェスを破棄して作り直すと、Wayland コンポジタ側で画面のちらつき（フリッカー）が発生します。
Toodle ではサーフェスを破棄せず、同一サーフェスに対して差分リクエストを発行します：
```rust
tasks.push(layer_cmd::set_anchor(surface_id, anchor));
tasks.push(layer_cmd::set_margin(surface_id, top, right, bottom, left));
tasks.push(layer_cmd::set_size(surface_id, Some(width), Some(height)));
tasks.push(layer_cmd::set_input_zone(surface_id, Some(bounds)));
```
これにより、画面が瞬くことなく 0ms で滑らかにサイズやマージンが更新されます。

---

## 3. クロック同期パイプライン

毎秒の更新で秒飛びやタイマードリフトが発生しないよう、次の 1 秒のマイクロ秒境界を逆算して待機する `clock_tick_stream` を実装しています：

```rust
pub fn clock_tick_stream() -> impl futures::Stream<Item = DateTime<Local>> {
    futures::stream::unfold((), |_| async {
        let now = Local::now();
        let dur = duration_until_next_second(now);
        tokio::time::sleep(dur).await;
        let next_now = Local::now();
        Some((next_now, ()))
    })
}
```

ここで `duration_until_next_second` は以下のように計算されます：
```rust
let nanos = now.timestamp_subsec_nanos();
let remaining_nanos = 1_000_000_000u32 - nanos;
Duration::from_nanos(remaining_nanos as u64)
```
CPU を高負荷に浪費するスピンループを行わず、常に物理時計の `XX:XX:XX.000000` の瞬間に目覚めて正確な描画更新を行います。

---

## 4. 天気サブシステムと2段階キャッシュ

```text
┌─────────────────┐       キャッシュヒット (<30分)
│  WeatherService ├──────────────────────────────► [ キャッシュデータを返却 ]
└────────┬────────┘
         │
         │ キャッシュ切れ / 未取得 / 座標変更
         ▼
┌──────────────────────┐  HTTP 200 (JSON)   ┌───────────────────────────┐
│  OpenMeteoProvider   ├───────────────────►│ Open-Meteo REST API       │
│  (reqwest + tokio)   │                    │ (api.open-meteo.com)      │
└────────┬─────────────┘                    └───────────────────────────┘
         │
         │ ネットワーク障害 / オフライン
         ▼
┌───────────────────────────┐
│ Stale キャッシュフォールバック│
│ (前回の有効データを継続表示) │
└───────────────────────────┘
```

### 4.1 厳格な座標バリデーション
東京からロンドンなど別の都市に切り替えた際に古いキャッシュを誤って返さないよう、`CachedWeather` は緯度・経度を記録し、一致判定を行います：
```rust
pub fn is_location_match(&self, lat: f64, lon: f64) -> bool {
    (self.latitude - lat).abs() < 0.02 && (self.longitude - lon).abs() < 0.02
}
```
座標が一致しない場合は即座にキャッシュをバイパスし、新都市の最新データを取得します。

### 4.2 キャッシュ保持ポリシー（設計第13項準拠）
- **現在天気**: 30分間有効。
- **7日間週間予報**: 3時間有効。
- **オフラインフォールバック**: ネットワーク未接続時は最後の有効キャッシュを安全に継続表示します。

---

## 5. inotify 設定ホットリロード

`toodle-settings` と `toodle` は `~/.config/toodle/config.toml` を介して疎結合に連携します：

1. **`toodle-settings` のアトミック書き込み**:
   一時ファイル `.tmp` に書き出し後、アトミックな `rename` により設定ファイルを更新。
2. **`toodle` デーモンの inotify 検知**:
   `notify::recommended_watcher` がリネームイベントを検知し、25ms 後に新設定を読み込んで反映。スライダーのドラッグ中もウィジェットが滑らかに追従します。

---

## 6. バイナリ組み込みタイポグラフィ

外部システムフォントの有無によって文字のメトリクスが崩れることを防ぐため（設計第10項）、Roboto、JetBrains Mono、DejaVu Serif、Open Sans の計 8 ファイルを `include_bytes!` でバイナリ内へ静的組み込みしています。どの Linux ディストリビューションでも完全に同一の美しいフォントで描画されます。
