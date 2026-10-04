# Pipeline — ダウンロード/変換パイプライン

> 正本: `src/runner.rs`。本ファイルはジョブ1本（1 URL）が「開始 → 取得 → 変換 → 検証 → 保存 → 後処理」までで何をするかの仕様。

## 全体フロー

```
URL + Profile + bitrate
        │
        ▼
  work = ~/.cache/asmr-dl/job-<id>-<ts>/   （Termux 内部ストレージ=高速）
  out_root = output_dir[/profile.subdir]   （共有ストレージ=保存先）
        │
        ├── backend = "ffmpeg" ────────────▶ ffmpeg_direct
        ├── backend = "ytdlp" ─────────────▶ via_ytdlp
        └── backend = "auto" ──▶ via_ytdlp
                                    │ 失敗 & fallback_to_ffmpeg
                                    └──▶ (work 掃除) ffmpeg_direct
        │
        ▼
  各ファイル: ffprobe → ffmpeg (Opus) → 検証 → out_root へ移動
        │
        ▼
  post_actions: termux-media-scan + termux-notification
```

一時ファイルは **Termux の内部領域（高速）** に書き、完成してから**共有ストレージへ移す**（`move_file`）。

## 経路の詳細

### 1. yt-dlp 経路（`via_ytdlp`）

1. `yt-dlp` を起動（work フォルダを `-P`）。主な引数:
   - `--newline --progress --color never`
   - `--progress-template "download:[PROG]%(progress._percent_str)s|%(progress._speed_str)s|%(progress._eta_str)s|%(info.playlist_index)s|%(info.n_entries)s|%(info.title)s"` — **固定フォーマットで進捗をパースするためのもの**
   - `-o <profile.filename>` / `--windows-filenames` / `--no-mtime`
   - `--retries 10 --fragment-retries 20`
   - `--no-playlist` / `--yes-playlist`（`profile.playlist`）
   - `-f <profile.format>` / `-S <profile.format_sort>` / `-N <concurrent_fragments>`
   - `--embed-metadata`（`profile.embed_metadata`）/ `--write-thumbnail --convert-thumbnails jpg`
   - `--split-chapters`（`profile.split_chapters`、`-o chapter:_chapters/<chapter_filename>`）
   - `--add-header Referer/UA/...` / `--cookies <path>`
   - `profile.ytdlp_args`（例: youtube の `--js-runtimes node --remote-components ejs:github`）
   - `-- <url>`
2. stdout/stderr を行ごとに解析（`run_proc` の `on_line`）:
   - `[PROG]` 行 → 6 フィールド `|` 分割 → 進捗（pct/speed/eta）+  playlist index + title
   - `ERROR:` → 最後に受信したエラーを保持
   - `[Merger]` / `[Metadata]` / `[ThumbnailsConvertor]` → stage「後処理」
   - `[SplitChapters]` → stage「チャプター分割」
3. 終了後 `collect_media(work)` でメディアファイルを再帰収集（`NON_MEDIA_EXT` と `__out_` / `.part-Frag` / `.temp.` を除外）
   - `status` が非 success でも**ファイルがあれば**「一部失敗」扱いで変換を続行
4. チャプター分割した場合は `_chapters/` 配下のみを対象
5. 各ファイルを `convert` → 保存

### 2. ffmpeg 直接経路（`ffmpeg_direct`）

1. `ffprobe` で URL を解析（`ProbeInfo`: codec/duration/title/has_audio）
   - プロファイルの HTTP ヘッダ（`-user_agent` / `-headers`）を付ける
   - `-rw_timeout 30000000`
2. タイトル（無い場合は URL のファイル名から `name_from_url`、汎用名なら `ホスト_YYYYmmdd-HHMMSS`）
3. `convert_with_info` で **DL + 変換を1パスで** `.opus` に
   - m3u8/mpd でない場合のみ `-reconnect 1 -reconnect_streamed 1 -reconnect_delay_max 10`
4. `out_root/<name>.opus` へ移動

## 変換（`convert` / `convert_with_info`）

### 入力の判定

- `ffprobe` で `has_audio` が false → **`音声トラックがありません` で失敗**（変換前に検出）

### コピー vs 再エンコード

| 条件 | 動作 |
|---|---|
| `copy_if_opus` && 加工指定なし && codec == `opus` | **`-c:a copy`（無劣化コピー）** |
| それ以外 | `libopus` で再エンコード |

### 再エンコード時の ffmpeg 引数

```
-c:a libopus -b:a <bitrate>k -vbr <vbr> -compression_level <min(10, level)>
-application audio -frame_duration <frame_duration> -ar 48000
[-ac <channels>]              # channels > 0 の場合
[-af <filters>]               # normalize / filter の場合
-map 0:a:0 -vn -sn -dn -map_metadata 0
-progress pipe:1 -nostats -f opus <out>
```

- **`normalize = true`** なら `loudnorm=I=-24:TP=-2:LRA=20`（ASMR 向けに緩め、ダイナミクスを潰しすぎない）
- **`filter`** があれば追加（例: `highpass=f=20`）

### 進捗の計算

- `ffmpeg -progress pipe:1` の `speed` / `out_time_us` を解析
- `pct = out_time / duration`（duration が分かれば）
- `eta = (duration - out_time) / speed`
- duration が分からない場合は `out_time` を表示

## 検証（`convert_with_info` 末尾）— 壊れたファイル対策

変換が成功した場合でも、**出力を再度 ffprobe** して検証する:

1. `out_info.has_audio` が false → **`出力に音声がありません` で失敗**
2. `out_info.duration < in_duration * 0.9 - 1.0` → **`出力が途中で切れています` で失敗**

> このガードを外さない（ADR-008）。途中で切れた Opus を音楽アプリに置くと、ユーザーが「壊れたファイル」として認識するため。

## ファイル命名（`collect_media` → 保存）

1. yt-dlp の出力名（`profile.filename`）を基準に `rel` を計算
2. `is_generic_name`（`index` / `master` / `chunklist_` 等）なら、
   **埋め込みタイトル**（`ProbeInfo.title`）→ 無ければ **`ホスト_YYYYmmdd-HHMMSS`** の順で置き換え
3. `sanitize_filename`（`/ \ : * ? " < > |` と制御文字を `_`、200 バイトに制限、空なら `audio`）
4. `unique_path` で同名衝突を回避（`名 (2).opus` 形式。並列ジョブ対策として**保存直前に判定**）

## 保存（`move_file`）

- `tokio::fs::rename` → 失敗（EXDEV、別ファイルシステム）なら `copy` + `remove`
- 出力フォルダが作れない場合はエラー（`termux-setup-storage は実行済み?`）
- `output_dir` が `~/storage` で `~/storage` が無い場合、`Config::output_root()` が `~/asmr-dl-output` にフォールバック

## 後処理（`post_actions`、Termux 連携）

- `media_scan = true` なら `termux-media-scan <files...>`（音楽アプリにすぐ表示）
- `notify = true` なら `termux-notification --group asmr-dl --title "ASMR DL: 完了" --content <name>`
- 両方とも `spawn_detached`（`kill_on_drop` + 20秒タイムアウト）— Termux:API 不在でも本体は止まらない

## キャンセル

- `RunCtx.cancel`（`watch::Receiver<bool>`）が `true` になったら `run_proc` が:
  - unix で `kill(-pid, SIGKILL)`（**yt-dlp が起動した ffmpeg も含むプロセスグループごと**）
  - `child.kill()`
  - `CanceledError` を返す
- `run` は `CanceledError` を検出したら `JobEvent::Canceled` を送信（失敗通知を出さない）
- work フォルダは `keep_temp = false` なら削除

## 関連

- [profiles.md](./profiles.md) — プロファイルごとの設定
- [engineering.md](./engineering.md) — プロセス管理・エラー処理の詳細
- [adr.md](./adr.md) — ADR-005（無劣化コピー）/ ADR-006（2段構え）/ ADR-008（長さ検証）
- `.agent/skills/tech-stack/SKILL.md` — yt-dlp/ffmpeg のハマりどころ
