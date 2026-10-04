# Termux — 連携・端末制約

> 正本: `src/main.rs`（`is_termux` / wake-lock）・`src/runner.rs`（`post_actions` / `notify` / `spawn_detached`）・`src/config.rs`（`output_root` / `home_dir`）・`scripts/`。

## Termux 検出（`is_termux`）

`TERMUX_VERSION` 環境変数が存在するか、`PREFIX` に `com.termux` を含むかで判定。

## 連携一覧

| 機能 | コマンド / 機構 | 設定 | 無い場合の挙動 |
|---|---|---|---|
| 画面ロック解除 | `termux-wake-lock`（起動時）/ `termux-wake-unlock`（終了時） | `wake_lock = true` | spawn 失敗を無視（本体は動く） |
| 完了/失敗通知 | `termux-notification --group asmr-dl --title ... --content ...` | `notify = true` | `spawn_detached`（20秒タイムアウトで kill） |
| メディアスキャン | `termux-media-scan <files...>` | `media_scan = true` | 同上 |
| 完了ファイルを開く | `termux-open <file>`（`o` キー） | — | `termux-open 失敗: ...` flash |
| 共有メニュー連携 | `~/bin/termux-url-opener`（`exec asmr-dl "$url"`） | — | 共有メニューから起動できない |

- **Termux:API アプリが無いと `termux-notification` / `termux-media-scan` は固まる**。
  `spawn_detached`（`src/runner.rs`）は `kill_on_drop(true)` + **20秒タイムアウトで kill** する fire-and-forget。
  新しい `termux-*` 呼び出しも必ずこれに乗せる（`memory-leak` R3）。

## 保存先のフォールバック（`Config::output_root`）

1. 既定: `~/storage/shared/Music/ASMR`（`termux-setup-storage` 済みなら Android の `Music/ASMR`）
2. `output_dir` が `~/storage` で始まるが `~/storage` が存在しない（未許可）場合 → **`~/asmr-dl-output`** にフォールバック
3. サムネイル（`.jpg`）は Opus と同じ名前で横に置く（対応プレイヤーでジャケット表示）

- `home_dir()` は `HOME` 環境変数、無ければ `/data/data/com.termux/files/home`。
- `expand_tilde` は `~` / `~/` を展開（設定の `output_dir` / `temp_dir` / `cookies` 等）。

## 共有メニュー連携（`scripts/termux-url-opener`）

- 置き場所: `~/bin/termux-url-opener`（実行権限付き）。`scripts/install-termux.sh` が配置（既存があれば上書きせず案内）。
- ブラウザ / YouTube アプリの「共有 → Termux」で呼ばれ、`exec asmr-dl "$url"` で URL をキューに入れた状態で起動。

## 貼り付け（共有テキストからの URL 抽出）

- 複数 URL を含むテキストを貼り付けると**URL だけ抽出してキューに追加**（`config::extract_urls`）。
  - 例: `この動画おすすめ→https://youtu.be/abc123?si=xx。あと https://example.com/a.m3u8, 以上` → 2 件
  - 末尾の `.,!?:;)]` 等と全角句読点は除去
- 1 件の場合は入力欄に挿入（編集可能）。

## 端末制約・対策

| 制約 | 対策 |
|---|---|
| **Phantom Process Killer**（Android が子プロセスを強制終了） | `process_group(0)` + 独立プロセスグループ。Galaxy の開発者オプション「子プロセスの制限を無効にする」を推奨（`ops/TERMUX_BATTERY.md`） |
| **共有ストレージの FS 制限** | 一時ファイルは内部ストレージ（高速）に書き、完成してから `move_file` で共有へ（`rename` 失敗なら `copy`+`remove`） |
| **使えないファイル名文字** | yt-dlp に `--windows-filenames`。ffmpeg 経路は `sanitize_filename` |
| **メモリ/CPU 限定** | `max_concurrent` 既定 2。`release` は `lto=thin, codegen-units=4, strip` |
| **縦画面（40〜60 桁）** | レイアウトは幅预算で計算（`unicode-width`）。崩れないことを最優先 |

## Galaxy (One UI) のおすすめ設定

長時間の ASMR 配信アーカイブを途中で止めないための設定は `ops/TERMUX_BATTERY.md` を参照:

1. Termux のバッテリー「制限なし」
2. バックグラウンド使用制限から除外
3. Android 14 以降: 開発者オプション「子プロセスの制限を無効にする」ON
4. asmr-dl は起動中に `termux-wake-lock` を自動取得（`wake_lock = false` で無効）

## 関連

- `ops/INSTALL_TERMUX.md` — インストール手順
- `ops/TERMUX_BATTERY.md` — バッテリー設定
- `ops/UPDATING_YTDLP.md` — yt-dlp 更新
- [architecture.md](./architecture.md) — 終了シーケンス（wake-unlock 等）
