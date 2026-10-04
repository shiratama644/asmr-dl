# Activity Log — 重要変更の追記式記録

> 最新を上。加えてよい、**削除・上書きはしない**（時点記録）。
> 各記録は `YYYY-MM-DD` / 変更内容 / 根拠 / 結論（採用・不採用・要確認）の順で書く。

---

## 2026-10-04 TUI モダンデザイン刷新 (UI-1)

- **内容**: `src/ui.rs` を全面書き換えし、Catppuccin Mocha ベースのモダンテーマに刷新。`src/app.rs` に `tick_count: u64` を追加（スピナー等のアニメーションフレーム用 / `on_tick` で +1）。
- **変更点**: 13 色パレット（`mod theme`）/ ヘッダ・フッタの `BG_ALT` 全幅バー / 角丸枠 / プロファイル名チップ（surface 背景）/ 実行中ジョブのブレイル・スピナー（200ms tick 連動・決定論）/ 進捗バー（accent + surface）/ 選択行 `▸ ` + surface1 / ポップアップは BG 塗り + 角丸。キー操作・画面構成・HitAreas 機構は不変。
- **根拠**: ユーザー指示「ratauiを使ったよりきれいでモダンなデザインにしてください」。
- **結果**: `cargo build/test` 未実行（§6.2: Sandbox の Rust egress 制約）。代替検証（構文バランス / ratatui 0.30 API 照合）のみ実施。
- **結論**: 要確認（実機・開発機でのビルド + 縦 40/60 桁のレイアウト確認が完了条件の残項目。計画: `docs/planning/UI_MODERN_TUI_PLAN.md`）

---

## 2026-10-04 TEMPLATE_REPO 適用（リポジトリ整備）

- **内容**: `shiratama644/TEMPLATE_REPO`（TS 製テンプレート）の `.agent/` / `docs/` / `AGENTS.md` / `README.md` を本リポジトリに導入し、**asmr-dl（Rust / Termux）向けに全面書き換え**した。
- **対応範囲**:
  - `asmr-dl.zip` を解凍し、ソース一式（`src/` / `Cargo.toml` / `scripts/` / `config.default.toml` / `LICENSE` / 等）をルートに配置（INIT-1）
  - `.agent/`（rules 5 / skills 14 / agents 5 / hooks 8 / commands 3 / output-styles 2 / workflow 1 / settings.json）
  - `docs/`（arch 13 / planning / research / audit / ops / examples / task-list）
  - ルート `README.md`（zip 内の日本語 README を正に採用）
  - `AGENTS.md` §6 を asmr-dl 固有の規約に置換
- **根拠**: 現行コード（`src/*.rs` / `Cargo.toml` / `config.default.toml` / `scripts/install-termux.sh`）の読み取り + Sandbox 制約の実測（`sh.rustup.rs` / `crates.io` 到達不可 → Rust ツールチェーン取得不可）
- **結果**: build / test は Sandbox 制約により未実行（AGENTS.md §6.2 の代替検証のみ実施：tomllib / `sh -n` / jq / リンクチェック）。最終検証は開発機・実機へ分離（`task-list.md` CI-1 / DEV-1）
- **結論**: 採用（完成）

---

## 2026-10-04 v0.1.0 時点の機能棚卸し

- **内容**: `src/` の全6ファイル（`main.rs` / `app.rs` / `ui.rs` / `runner.rs` / `config.rs` / `job.rs` ※本リポジトリにはこれ以外のソースファイルは存在しない。当初記録の `cli.rs` / `util.rs` / `main_ui.rs` は誤りとして訂正）の機能整理
- **根拠**: ソースコード直接の読み取り
- **結果**: 取得（yt-dlp / ffmpeg 直接 / auto 2段構え）/ 変換（Opus / 無劣化コピー / loudnorm 任意）/ TUI（7画面）/ 制御（キュー / キャンセル / process group kill）/ Termux 連携（wake-lock / 通知 / media-scan / termux-open）を確定
- **結論**: 採用（`docs/arch/*.md` に反映済み）
