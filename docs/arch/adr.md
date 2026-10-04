# ADR — 意思決定ログ

> 「なぜそうしたか」の記録。`Status: Accepted` の ADR に反する実装はせず、変更が必要なら人間に確認する（`arch/README.md` §3）。
> 本ログは 2026-10-04（TEMPLATE_REPO 適用時）に、**現行コードから読み取れる決定**を整理して作成した。

---

## ADR-001: TUI フレームワークは ratatui（crossterm 直接依存しない）

- **Status**: Accepted
- **Context**: ターミナル UI が必要。crossterm を直接使う手もあった。
- **Decision**: `ratatui 0.30`（feature: `unstable-rendered-line-info`）を採用。**crossterm は ratatui が再エクスポートするため直接依存にしない**（`Cargo.toml` のコメント）。
- **Rationale**: 描画の抽象化・レイアウトの簡便性。crossterm のバージョン不一致を防ぐ。

## ADR-002: 非同期ランタイムは tokio、キー入力は専用 OS スレッド

- **Status**: Accepted
- **Context**: 外部プロセスの並行実行と、blocking なキー読み込みの両立が必要。
- **Decision**: `tokio`（rt-multi-thread）。キー入力は `std::thread::spawn` で `event::read()` をループし、`mpsc::unbounded_channel` へ（`src/main.rs`）。
- **Rationale**: crossterm の `event::read` は blocking。tokio のタスクで回すと整个イベントループを詰まらせるため、専用スレッドで読んでチャネルに渡す。

## ADR-003: 設定は `config.toml`（`include_str!` で既定値をバイナリ埋め込み）

- **Status**: Accepted
- **Context**: 初回起動時の設定生成、既定値と実装の同期が必要。
- **Decision**: `config.default.toml` を `include_str!("../config.default.toml")` でバイナリに埋め込み（`config.rs::DEFAULT_CONFIG`）。初回起動時に `~/.config/asmr-dl/config.toml` を作成。
- **Rationale**: 既定値は常にバイナリと同期（外部ファイルを探す必要がない）。ユーザーは生成されたファイルを編集。

## ADR-004: プロファイルは「上から順に判定 + generic が末尾のフォールバック」

- **Status**: Accepted
- **Context**: サイトごとに取得方法・音質が異なる。URL から自動判定したい。
- **Decision**: `[[profiles]]` を上から順に評価（`url_regex` → `domains`）。**`generic` は常に末尾**（`Config::normalize()` が保証）。
- **Rationale**: ユーザーが独自サイトを「generic より前」に挿入すれば自動的に優先される。generic の位置が固定なので、フォールバックが必ず機能する。

## ADR-005: 元が Opus かつ加工指定が無ければ無劣化コピー（`-c:a copy`）

- **Status**: Accepted
- **Context**: YouTube の 251（Opus）等を再エンコードすると無意味に劣化する。
- **Decision**: `copy_if_opus && !needs_processing() && codec == "opus"` のとき `ffmpeg -c:a copy`。それ以外は `libopus` で再エンコード（`src/runner.rs::convert_with_info`）。
- **Rationale**: ASMR 元の音質を壊さない。加工（normalize / filter / channels）が必要なら再エンコード必須。

## ADR-006: 2段構えの取得（backend=auto で yt-dlp → 失敗時 ffmpeg 直接）

- **Status**: Accepted
- **Context**: yt-dlp が対応しないサイト（m3u8 直リンク等）もある。
- **Decision**: `backend = "auto"` のとき `via_ytdlp` → 失敗 & `fallback_to_ffmpeg` なら work フォルダを掃除して `ffmpeg_direct`（`src/runner.rs::run_inner`）。
- **Rationale**: 1つの URL で「できる限り」取得する。m3u8 直リンクは ffmpeg 直接で 1 パスにできる。

## ADR-007: キャンセルは watch チャネル + プロセスグループごと SIGKILL

- **Status**: Accepted
- **Context**: yt-dlp が ffmpeg を起動するため、親だけ kill すると ffmpeg が残る。
- **Decision**: キャンセルは `watch::channel(bool)`（`RunCtx.cancel`）。`run_proc` は unix で `process_group(0)` 起動し、キャンセル時 `kill(-pid, SIGKILL)` + `child.kill()`（`src/runner.rs`）。
- **Rationale**: 孤児プロセスを防ぐ。`CanceledError` で「キャンセル」と「失敗」を区別し、失敗通知を出さない。

## ADR-008: 変換後の長さ検証（途中で切れた出力を失敗扱い）

- **Status**: Accepted
- **Context**: 途中で切れた Opus を保存すると、ユーザーが「壊れたファイル」として認識する。
- **Decision**: 変換後に出力を再度 ffprobe し、`has_audio` / **長さ（入力の 90% 以上、-1秒許容）** を確認。不合格なら失敗（`src/runner.rs::convert_with_info`）。
- **Rationale**: 壊れたファイルを音楽アプリに置かない。ASMR は長時間なので、途中で切れる損失が大きい。

## ADR-009: 一時ファイルは内部ストレージ、完成してから共有へ

- **Status**: Accepted
- **Context**: Android の共有ストレージ（FUSE）は遅く、FS 制限がある。
- **Decision**: 一時ファイルは `~/.cache/asmr-dl/job-*`（内部ストレージ=高速）に書き、完成したら `move_file`（`rename` 失敗なら `copy`+`remove`）で共有へ（`src/runner.rs`）。
- **Rationale**: 変換中の I/O を高速化。別ファイルシステム間の移動に対応。

## ADR-010: 音量正規化は既定 OFF（ASMR は小さい音が持ち味）

- **Status**: Accepted
- **Context**: `loudnorm` で音量を揃えると、ASMR の小さな音（ささやき等）のダイナミクスが潰れる。
- **Decision**: `normalize` の既定は `false`。有効時は `loudnorm=I=-24:TP=-2:LRA=20`（緩め）。ユーザーは `filter` で調整可能（`AudioSettings` 既定値 + `config.default.toml` のコメント）。
- **Rationale**: ASMR の特徴（定位・小さな音）を壊さないことを優先。

## ADR-011: チャンネル数は既定 0（元のまま）

- **Status**: Accepted
- **Context**: バイノーラル/ステレオの定位は ASMR の生命線。
- **Decision**: `channels` の既定は `0`（元のチャンネル数を維持）。`1`/`2` で明示変換（`AudioSettings`）。
- **Rationale**: 定位を崩さない。`needs_processing()` は `channels > 0` を再エンコード必須とする。

## ADR-012: `termux-*` は `spawn_detached`（fire-and-forget + タイムアウト）

- **Status**: Accepted
- **Context**: Termux:API アプリが無いと `termux-notification` 等は固まる。
- **Decision**: `spawn_detached`（`kill_on_drop(true)` + 20秒タイムアウトで kill）（`src/runner.rs`）。
- **Rationale**: Termux 連携が無い環境でも本体は動く。固まったプロセスを残さない。

## ADR-013: 依存の最小化（新規クレートは合意制）

- **Status**: Accepted
- **Context**: 端末（メモリ/CPU 限定）でのビルド時間・バイナリサイズ・サプライチェーン。
- **Decision**: 依存は `ratatui/tokio/serde/toml/anyhow/regex/url/unicode-width/libc` のみ。**新規クレート追加はユーザーの明示的合意が必要**（`AGENTS.md §6.1`）。
- **Rationale**: ビルドが軽く、依存の更新・影響範囲が小さい。

## ADR-014: Sandbox では cargo 実行不可を恒常的制約として扱う（代替検証）

- **Status**: Accepted
- **Context**: Arena Sandbox は `sh.rustup.rs` / `static.rust-lang.org` / `crates.io` に到達できず、apt も不可（2026-10-04 実測）。Rust ツールチェーンを入手できない。
- **Decision**: `cargo build/test/clippy/fmt` は Sandbox で実行しない。代替検証（tomllib / `sh -n` / jq / リンクチェック）を実施し、最終検証は CI（未導入時は開発機/実機）へ分離。コミット報告に「build/test 未実行（§6.2）」を明記（`AGENTS.md §6.2` / `sandbox-constraints` スキル）。
- **Rationale**: 制約を「乗り越える」のではなく「迂回する」。未検証の変更を「完了」として誤認させない。

---

## 変更ルール

- ADR は**追記のみ**。既存の ADR を書き換えない（変更が必要な場合は新 ADR で代替し、旧 ADR に `Superseded by ADR-0XX` を記載）。
- `Status: Accepted` の ADR に反する実装はせず、変更が必要なら人間に確認する。
