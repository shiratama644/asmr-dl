---
name: import-boundaries
description: モジュール間の依存方向（config/job → runner → app → ui → main）を守り、逆参照・循環参照・責務混在を防ぐスキル。Rust の use / mod 境界。Use when adding a new module, moving code between files, or reviewing a refactor that touches module boundaries.
---

# Import Boundaries — モジュール境界を守るスキル

> 出典: TEMPLATE_REPO の `import-boundaries/SKILL.md`（TS import 層）を **Rust モジュール構造向けに全面書き換え**
> 正本: `src/main.rs`（mod 宣言）、`docs/arch/architecture.md`、`AGENTS.md §6.4`

## 原則

- モジュール間の依存は**下位へのみ**。上位が下位を `use` してよい。下位が上位を `use` しない
- 循環参照（`a -> b -> a`）を作らない
- I/O（外部プロセス・ファイル・端末）は `runner`（および `main` の起動処理）に集約し、純粋層（`config` / `job`）に混入させない
- UI 特有の状態（`HitAreas` / `picker_idx` / `log_scroll`）は `app` / `ui` に置き、`runner` へ漏らさない

## モジュールの層（asmr-dl）

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

### 禁止例

```rust
// ❌ L0 (config) が L1 (runner) を参照
// src/config.rs
use crate::runner::RunCtx;   // 逆参照。config は純粋層なので runner を読まない

// ❌ L0 (job) が L3 (ui) を参照
// src/job.rs
use crate::ui::draw;         // 逆参照

// ❌ L1 (runner) が L3 (ui) を参照
// src/runner.rs
use crate::ui::HitAreas;     // runner は描画を知らない
```

### 許可例

```rust
// ✅ L4 (main) が全てを参照して配線
// src/main.rs
mod app; mod config; mod job; mod runner; mod ui;
use app::{App, ToolStatus};
use config::Config;
use ratatui::crossterm::event::...;

// ✅ L2 (app) が L0 + L1 を参照
// src/app.rs
use crate::config::{extract_urls, Config};
use crate::job::{Job, JobEvent, JobMsg, Status};
use crate::runner::{self, RunCtx};

// ✅ L3 (ui) が L2 (app) を参照して描画
// src/ui.rs
use crate::app::App;

// ✅ L1 (runner) が L0 (config / job) を参照
// src/runner.rs
use crate::config::{expand_tilde, host_of, Backend, Config, Profile};
use crate::job::{JobEvent, JobMsg};
```

## `crate::AppMsg` の扱い（例外として許容するもの）

- `AppMsg` は `main.rs` で定義され、`app` / `runner` / `ui` から参照される（`crate::AppMsg`）。
- これは**メッセージの型（純粋データ）**なので、L1/L2/L3 から読むのは許容する。
- ただし `AppMsg` の**内容を実装するロジック**（ジョブ進捗の更新等）は `app::on_msg` に集約し、`runner` は `send` するのみ（`RunCtx::send`）。

## 境界を壊しやすい典型的な変更

| 変更 | リスク | 確認事項 |
|---|---|---|
| 新モジュール追加 | 層の位置を誤る | 新モジュールがどの層か決める。`use` が下位だけか確認 |
| `App` のフィールド追加 | UI 状態が `runner` へ漏れる | `RunCtx` に UI 状態を持たせない（`RunCtx` は `id/url/profile/bitrate/cfg/tx/cancel` のみ） |
| 外部コマンドの追加 | I/O が `config` / `job` に混入 | 新しい外部コマンド呼び出しは `runner`（または `main` の起動処理）に置く |
| ヘルパー関数の移動 | 純粋層に I/O が混入 | 関数が `std::fs` / `tokio::process` を使うなら `runner` / `main` へ |

## 確認コマンド（依存の向きを確認）

```bash
# 各ファイルがどの crate 内部モジュールを use しているか
grep -n "^use crate::" src/*.rs

# 逆参照の疑い: config/job が runner/app/ui を参照していないか
grep -n "use crate::\(runner\|app\|ui\)" src/config.rs src/job.rs || echo "OK: no up-reference"

# runner が ui を参照していないか
grep -n "use crate::ui" src/runner.rs || echo "OK: runner does not use ui"
```

## やってはいけないこと

- 境界を「今だけ」と無視して `use` しない（一度突破すると拡張される）
- 循環参照を回避するために `pub` を全開にして「中身を見せる」代わりに境界を壊さない
- 境界変更は `docs/arch/architecture.md` と本スキルを**同時に**更新する

## 関連

- `docs/arch/architecture.md` — モジュール構造・メッセージフローの正本
- `determinism/SKILL.md` — 純粋層の決定論
- `memory-leak/SKILL.md` — 境界を跨ぐリソース解放
