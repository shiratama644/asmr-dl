---
name: memory-leak
description: 子プロセス・watch/mpsc チャネル・tokio タスク・ログリングバッファのリソース解放を守るスキル。kill_on_drop、process group kill、VecDeque cap、drop 時の kill、チャネルの close。Use when spawning external processes, adding channels/tokio tasks, or reviewing cancellation/cleanup paths.
---

# Memory Leak / Resource Leak — リソース解放を守るスキル

> 出典: TEMPLATE_REPO の `memory-leak/SKILL.md`（JS GC / Room 前提）を **Rust のリソース管理向けに全面書き換え**
> 正本: `src/runner.rs`（`run_proc` / `read_lines` / `spawn_detached`）、`src/app.rs`（`cancel_all`）、`docs/arch/engineering.md`、`AGENTS.md §6.4`

## なぜ重要か（Rust での意味）

Rust は GC なしで所有権で解放するが、**外部リソース（子プロセス・ファイルディスクリプタ・ターミナル状態）**と**長期生存するバッファ**は、所有権だけで正しく解放されない:

- 子プロセスは `kill_on_drop` なしだと drop 時に kill されない（孤児になる）
- yt-dlp が起動した ffmpeg は**プロセスグループごと kill** しないと同じく孤児になる
- `VecDeque` のログは cap なしで無制限に増える
- `termux-notification` 等は Termux:API が無いと**固まる**（タイムアウトで kill が必要）
- チャネル（`mpsc` / `watch`）の送信側を drop しないと受信側が閉じない

## 潜在リーク箇所と既定の実装パターン（asmr-dl 既存）

| # | 場所 | リスク | 既定のパターン（維持すること） |
|---|---|---|---|
| R1 | 外部プロセス（yt-dlp/ffmpeg/ffprobe） | 孤児プロセス | `cmd.kill_on_drop(true)`（`run_proc`） |
| R2 | yt-dlp が起動した ffmpeg | キャンセル後も ffmpeg が残存 | unix で `process_group(0)` 起動 + キャンセル時 `kill(-pid, SIGKILL)`（`run_proc`） |
| R3 | `termux-*`（notification / media-scan / open） | Termux:API 不在で固まる | `spawn_detached`: `kill_on_drop(true)` + 20秒タイムアウトで kill（`runner::spawn_detached`） |
| R4 | `Job.logs`（`VecDeque<String>`） | 長時間実行で無制限増加 | `push_log` で `MAX_LOG_LINES=400` 超過時は `pop_front`（`job::Job::push_log`） |
| R5 | キャンセルの伝播 | キャンセルが届かないジョブ | `watch::channel(false)`（`RunCtx.cancel`）。`cancel_all`（`app`）で全ジョブに `send(true)` |
| R6 | 終了時の子プロセス | 終了時に kill が届く前にプロセスが死んでメッセージが欠落 | `main` は `cancel_all()` 後 `sleep(300ms)` して kill が届くのを待つ |
| R7 | ターミナル状態（raw mode / 代替画面 / mouse / paste） | panic 時・終了時に端末が壊れる | `ratatui::init()` / `ratatui::restore()` に任せる。`main` は exit 時に `DisableMouseCapture` / `DisableBracketedPaste` を明示実行 |
| R8 | mpsc チャネル（`erx` / `rx`） | 送信側 drop 無しで受信側が閉じない | `main` のイベントループは `tx` を `App` / `RunCtx` に渡して所有。終了時自然に drop |

## 修正パターン（新コードで必ず踏襲する）

### 外部プロセスの起動

```rust
let mut cmd = tokio::process::Command::new(&path);
cmd.stdin(Stdio::null())
   .stdout(Stdio::piped())
   .stderr(Stdio::piped())
   .kill_on_drop(true);          // R1: drop 時に kill
#[cfg(unix)]
cmd.process_group(0);            // R2: 独立プロセスグループ
```

- キャンセル時は:
  ```rust
  #[cfg(unix)]
  if let Some(pid) = child.id() {
      unsafe { libc::kill(-(pid as libc::pid_t), libc::SIGKILL) };  // グループごと
  }
  let _ = child.kill().await;
  ```
- **新しい外部コマンドを起動する際は必ずこのパターンにする**（`kill_on_drop` + unix `process_group(0)`）。

### fire-and-forget の termux-*

```rust
fn spawn_detached(mut cmd: Command) {
    cmd.stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).kill_on_drop(true);
    if let Ok(mut child) = cmd.spawn() {
        tokio::spawn(async move {
            if tokio::time::timeout(Duration::from_secs(20), child.wait()).await.is_err() {
                let _ = child.kill().await;   // R3: 固まったら kill
            }
        });
    }
}
```

### リングバッファ（ログ）

```rust
pub fn push_log(&mut self, s: impl Into<String>) {
    if self.logs.len() >= MAX_LOG_LINES {
        self.logs.pop_front();
    }
    self.logs.push_back(s.into());
}
```

- **新的な「履歴・キュー」状態を追加する時は必ず cap を付ける**（`VecDeque` + 上限 + `pop_front`）。
- 上限の値は用途に応じて（ログは 400 行）。

### チャネル

- キャンセル伝播は `watch::channel(bool)`（`RunCtx.cancel`）。ジョブ完了/キャンセルで `cancel = None` にして watch 送信側を drop する（`on_job_msg` の `Done` / `Failed` / `Canceled` 参照）。
- 進捗は `mpsc::unbounded_channel`（`tx`）。ジョブが死ぬと `RunCtx` が drop され `tx` が閉じる（`rx` が `None` を返す）。

## 監査コマンド

```bash
# 外部プロセスを spawn するのに kill_on_drop がない箇所
grep -n "Command::new" src/*.rs
# 各 spawn 箇所で kill_on_drop(true) があるか目視（run_proc / spawn_detached / main の termux-*）

# 履歴状態（VecDeque / Vec）に cap があるか
grep -n "VecDeque\|push_log\|MAX_LOG" src/job.rs

# process group kill が unix で書かれているか
grep -n "process_group\|libc::kill" src/runner.rs
```

## やってはいけないこと

- 子プロセスを `kill_on_drop(false)` で放置しない（孤児）
- キャンセル時に親プロセスだけ kill して yt-dlp の子 ffmpeg を残さない（process group kill 必須）
- 履歴・キューに cap を付けずに `push` しない
- `let _ = child.kill()` の失敗を無視しすぎない（重要箇所でログする）
- ターミナルの raw mode / 代替画面を手動で操作して `ratatui::init/restore` をバイパスしない（panic 時フックを壊す）

## 関連

- `docs/arch/engineering.md` — プロセス管理・エラー処理の設計
- `src/runner.rs` — `run_proc` / `read_lines` / `spawn_detached` の実装
- `determinism/SKILL.md` — 純粋層との境界
