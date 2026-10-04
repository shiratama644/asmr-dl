---
name: tech-stack
description: Rust/Cargo/ratatui/tokio + 外部ツール（yt-dlp, ffmpeg, ffprobe）の使い方とハマりどころ。progress-template パース、--js-runtimes、libopus、process group kill、Termux ビルド。Use when writing or reviewing Rust code, changing dependencies, or debugging yt-dlp/ffmpeg interactions.
---

# Tech Stack — Rust + 外部ツール + Termux

> 出典: TEMPLATE_REPO の tech-stack スキルを asmr-dl（Rust）向けに全面書き換え
> 正本: `Cargo.toml`、`config.default.toml`、`docs/arch/tech-stack.md`、`AGENTS.md §6`

## 依存（Cargo.toml、最小構成を維持）

| クレート | バージョン | 用途 | 注意 |
|---|---|---|---|
| ratatui | 0.30 | TUI 描画 + crossterm（再エクスポート） | feature: `unstable-rendered-line-info`。**crossterm を直接依存にしない**（バージョン不一致防止） |
| tokio | 1 | 非同期ランタイム | features: rt-multi-thread, macros, process, sync, time, io-util, fs |
| serde | 1 | シリアライズ | derive |
| toml | 0.8 | 設定ファイル | `config.default.toml` は `include_str!` で埋め込み |
| anyhow | 1 | エラー処理 | `?` + `.context()` |
| regex | 1 | URL 抽出・プロファイル url_regex | `OnceLock<Regex>` で1回コンパイル |
| url | 2 | URL 解析（host 抽出） | |
| unicode-width | 0.2 | 描画幅計算（全角対応） | 縦画面レイアウトの幅预算に使う |
| libc | 0.2 | process group kill / localtime_r | unix のみ（process_group / SIGKILL） |

**新規クレート追加はユーザーの明示的合意が必要**（AGENTS.md §6.1）。

## Release プロファイル（Cargo.toml）

```toml
[profile.release]
opt-level = 3
lto = "thin"
codegen-units = 4
strip = true
panic = "unwind"
```

- S24 Ultra で `cargo build --release` は数分かかることを前提にしている。
- `panic = "unwind"` は Terminal パニック時の画面復元フック（`ratatui::init`）を踏まないため。`abort` にしない。

## 外部コマンド（実行時依存）

| コマンド | 用途 | 検出 |
|---|---|---|
| `yt-dlp` | 取得（サイト対応が広い） | 起動時バックグラウンドで `--version` |
| `ffmpeg` | 変換（libopus） | `-version` / `-encoders` に `libopus` があるか |
| `ffprobe` | 解析（codec / duration / title / 音声有無） | `-version` |
| `termux-wake-lock` / `termux-notification` / `termux-media-scan` / `termux-open` | Termux 連携 | **bin が無い場合は無音でスキップ**（本体は動く） |

- パスは `config.toml` の `ytdlp_path` / `ffmpeg_path` / `ffprobe_path` で指定可（既定は PATH 上の名前）。
- yt-dlp は**頻繁に更新が必要**（サイト側の変更対応）。取れなくなったら `pip install -U yt-dlp yt-dlp-ejs` を第一候補にする。

## ハマりどころ（実測・既存コード由来）

- **YouTube は JS ランタイムが必要**: youtube プロファイルは `ytdlp_args = ["--js-runtimes", "node", "--remote-components", "ejs:github"]` を付けている（`config.default.toml`）。`No supported JavaScript runtime` エラーが出たら `pkg install nodejs` + `pip install -U yt-dlp yt-dlp-ejs`。
- **progress-template の固定フォーマット**: yt-dlp に
  `--progress-template "download:[PROG]%(progress._percent_str)s|%(progress._speed_str)s|%(progress._eta_str)s|%(info.playlist_index)s|%(info.n_entries)s|%(info.title)s"`
  を渡し、`[PROG]` プレフィックス行を 6 フィールド `|` 分割してパースしている（`src/runner.rs::via_ytdlp`）。**yt-dlp のバージョンアップで出力が変わったらここで壊れる**（`NA` / 空値は `clean_na` で吸収済み）。
- **`NA` / `N/A` / `Unknown` 値**: yt-dlp は未定値を `NA` 等で出力する。`clean_na()`（`src/runner.rs`）で空文字に変換してから使う。
- **libopus が無い ffmpeg**: `-encoders` に `libopus` が無ければ「opus✗」表示。`pkg reinstall ffmpeg` で直す。
- **`--windows-filenames`**: Android 共有ストレージで使えないファイル名文字を yt-dlp 側に避けさせる（`-o` テンプレートとの併用）。
- **process group kill**: `run_proc` は unix で `process_group(0)` で起動し、キャンセル時は `kill(-pid, SIGKILL)` で**yt-dlp が起動した ffmpeg までまとめて停止**する（`src/runner.rs::run_proc`）。このパターンを変更すると、キャンセル後に ffmpeg が残存する。
- **kill_on_drop(true)**: 全外部プロセスで必須。task が drop される瞬間に kill が届く。
- **read_lines の UTF-8 切断対策**: 8192 バイトのスタックバッファで読み、`\n`/`\r` 毎にフラッシュ。チャンク境界でマルチバイト文字が切れても `from_utf8_lossy` で行単位に復元する（`src/runner.rs::read_lines`）。行単位のパースを変更する時はこの不変条件（行=文字列として扱える）を壊さない。
- **termux-api が無い環境**: `termux-notification` / `termux-media-scan` は Termux:API アプリが無いと**固まる**。`spawn_detached`（`src/runner.rs`）は 20 秒タイムアウトで kill する fire-and-forget パターン。新しい termux-* 呼び出しも必ずこれに乗せる。
- **`move_file` のクロスファイルシステム対策**: `rename` 失敗（EXDEV）なら `copy` + `remove`（Termux の内部ストレージ → 共有ストレージは別 FS）。
- **`-reconnect` フラグ**: ffmpeg 直接経路で m3u8/mpd 以外に付ける（`src/runner.rs::convert_with_info`）。m3u8 に付けると HLS の再取得が狂う。
- **トランスコード後検証**: 出力を再度 ffprobe し、入力の 90% 未満（-1秒許容）なら「途中で切れている」として失敗にする（`src/runner.rs::convert_with_info`）。このガードを外さない。

## Termux ビルド

```bash
pkg install rust ffmpeg python nodejs termux-api
pip install -U yt-dlp yt-dlp-ejs mutagen
cargo build --release
```

- Termux の `rust` パッケージは stable。rustup は不要（動かない）。
- `scripts/install-termux.sh` が一括実行する（`docs/ops/INSTALL_TERMUX.md` 参照）。
- 共有ストレージは `termux-setup-storage` で許可を取るまで `~/storage/shared` が無い。`Config::output_root()`（`src/config.rs`）は `~/storage` が無いとき `~/asmr-dl-output` にフォールバックする。

## 関連

- `docs/arch/tech-stack.md` — 詳細なバージョン方針
- `docs/arch/termux.md` — Termux 連携の詳細
- `sandbox-constraints/SKILL.md` — Sandbox での制約
