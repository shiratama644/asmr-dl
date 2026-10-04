# Architecture — モジュール構造・依存規則・メッセージフロー

> 正本: `src/main.rs`（mod 宣言）・各 `src/*.rs`。本ファイルは「どうなっているか」の仕様として記録する。

## モジュール構成

```
src/
├── main.rs    # 起動・端末初期化・イベントループ・ツール検出（配線のみ）
├── app.rs     # アプリ状態・キュー制御・キー/マウス/貼り付け処理
├── ui.rs      # Ratatui 描画（縦画面向けレイアウト・ポップアップ・HitAreas 記録）
├── runner.rs  # yt-dlp / ffprobe / ffmpeg の実行、進捗解析、出力検証、保存
├── config.rs  # 設定・プロファイル・URL抽出・cookies.txt読み込み（純粋中心）
└── job.rs     # ジョブの状態とメッセージ（純粋データ）
```

## 依存規則（層）

依存は**下位へのみ**。上位が下位を `use` してよい。下位が上位を `use` しない。循環参照禁止。

```
L0: config / job      — 純粋データ・設定・状態（I/O なし）
       ↑
L1: runner            — yt-dlp / ffprobe / ffmpeg の実行・解析・保存（I/O 層）
       ↑
L2: app               — アプリ状態・キュー制御・入力イベント（I/O なし、状態のみ）
       ↑
L3: ui                — 描画のみ（HitAreas を更新する副作用を含む）
       ↑
L4: main              — 起動・端末初期化・イベントループの配線（副作用・I/O）
```

詳細・禁止例・確認コマンドは `.agent/skills/import-boundaries/SKILL.md`。

### 各モジュールの責務

| モジュール | 責務 | 持たないもの |
|---|---|---|
| `config` | 設定の読み込み・正規化・プロファイル判定・URL 抽出・cookies.txt 解析 | 描画・I/O（ファイル読み込みは設定のみ）・ジョブ制御 |
| `job` | ジョブの状態・メッセージ・ログリングバッファ（cap 付き） | 描画・I/O・スケジューリング |
| `runner` | 外部プロセスの起動・行解析・変換・検証・保存・Termux 後処理 | アプリ状態の所有（`RunCtx` に必要なものだけ）・描画 |
| `app` | 状態管理・入力イベント→状態変化・キューのスケジューリング・メッセージ処理 | 描画（ui  delegating）・外部プロセスの直接起動（runner 委譲） |
| `ui` | 状態をラテン文字/枠線で描画 + `HitAreas` を記録 | 状態の変更（描画副作用のみ）・I/O |
| `main` | 起動・引数処理・ツール検出・端末初期化・イベントループ・終了処理 | 個別のロジック（各モジュールに委譲） |

## メッセージフロー

### 全体像

```
                    ┌─────────────────────────────────────────┐
                    │                main                      │
                    │  tokio::select! ループ（200ms tick）      │
                    │                                          │
  crossterm event ──┤  erx (mpsc) ──▶ app.on_event             │
  (専用スレッド)     │  rx  (mpsc) ──▶ app.on_msg               │
                    │  tick         ──▶ app.on_tick            │
                    │                                          │
                    │  app.schedule() → 空きがあれば            │
                    │    tokio::spawn(runner::run(RunCtx))     │
                    │  terminal.draw(ui::draw)                 │
                    └───────────────┬─────────────────────────┘
                                    │ RunCtx.tx (mpsc)
                                    ▼
                    ┌─────────────────────────────────────────┐
                    │            runner (1ジョブ=1タスク)       │
                    │  yt-dlp / ffprobe / ffmpeg を起動        │
                    │  行解析 → JobEvent (Stage/Progress/      │
                    │  Title/Log/Done/Failed/Canceled)         │
                    │  キャンセルは watch::Receiver<bool>       │
                    └─────────────────────────────────────────┘
```

### メッセージの種類

| 型 | 方向 | 内容 |
|---|---|---|
| `AppMsg::Job(JobMsg{id, ev})` | runner → main → app | ジョブイベント（`JobEvent`） |
| `AppMsg::Tools(ToolStatus)` | main（バックグラウンド）→ app | ツール検出結果（yt-dlp/ffmpeg/ffprobe/libopus） |
| `crossterm::Event` | 専用スレッド → main → app | キー/貼り付け/マウス |
| `watch::Receiver<bool>` | app → runner | キャンセル信号（`true` でキャンセル） |

### `AppMsg` の扱い（境界の例外として許容）

- `AppMsg` は `main.rs` で定義され、`app` / `runner` / `ui` から参照される（`crate::AppMsg`）。
- これは**メッセージの型（純粋データ）**なので L1/L2/L3 から読むのは許容。
- ただし `AppMsg` の**内容を実装するロジック**（ジョブ進捗の更新等）は `app::on_msg` に集約。`runner` は `RunCtx::send` で送信するのみ。

## 並行性・スレッドモデル

| 要素 | 方式 | 備考 |
|---|---|---|
| キー入力 | **専用 OS スレッド**で `event::read()` をループし `mpsc::unbounded_channel` へ | blocking な `read` を tokio の外で回す（`src/main.rs`） |
| アプリループ | `#[tokio::main]` のタスク。`tokio::select!` で event/msg/tick を待機 | 1フレームにつき `schedule()` → `draw()` |
| ジョブ | ジョブごとに `tokio::spawn(runner::run)` | 同時実行数は `max_concurrent`（既定 2）で制御 |
| ツール検出 | 起動時に `tokio::spawn` でバックグラウンド（python の起動が遅いため） | 結果は `AppMsg::Tools` で後から届く |
| 外部プロセス | `tokio::process::Command`（stdin null / stdout・stderr piped） | stdout/stderr は `read_lines` で行単位に `mpsc` |

## 状態の所有

- `App` は `main` が所有。`cfg` は `Arc<Config>` で `RunCtx` に共有。
- `Job` は `App.jobs: Vec<Job>`。`runner` は `RunCtx`（id/url/profile/bitrate/cfg/tx/cancel）を所有し、ジョブ状態そのものは持たない（イベントで通知）。
- ターミナル状態（raw mode / 代替画面 / mouse / paste）は `ratatui::init/restore` に委譲。

## 終了シーケンス

1. `should_quit`（`q` / `Ctrl+C`。実行中なら `ConfirmQuit` で確認）
2. `app.cancel_all()`（全ジョブに watch `true`）
3. `DisableMouseCapture` / `DisableBracketedPaste` / `ratatui::restore()`
4. `termux-wake-unlock`（Termux かつ `wake_lock`）
5. `sleep(300ms)`（子プロセスに kill が届くのを待つ — `memory-leak` R6）
6. 完了件数を表示し `main` を返す

## 関連

- `.agent/skills/import-boundaries/SKILL.md` — 境界の詳細・確認コマンド
- [pipeline.md](./pipeline.md) — ジョブ内の取得/変換の流れ
- [engineering.md](./engineering.md) — プロセス管理・エラー処理
- `../audit/` — 差分・バグの時点記録
