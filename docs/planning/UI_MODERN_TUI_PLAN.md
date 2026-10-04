# UI 刷新: TUI をモダンデザインに（Catppuccin Mocha）

> 対応 task-list ID: `UI-1` (docs/task-list.md)
> 計画書テンプレート: docs/planning/_TEMPLATE.md 準拠
> 作成: 2026-10-04（ユーザー指示「ratauiを使ったよりきれいでモダンなデザインにしてください」）

## 1. 開始前確認

- 着手時ブランチ: `arena/01a10460-asmr-dl` / HEAD: `a013b9f`（リポジトリ整備）/ `git status` clean
- 依存タスク: なし
- 関連仕様: `AGENTS.md §6` / `docs/arch/architecture.md`（モジュール境界）/ `.agent/skills/zero-alloc`（tick ループ）

## 2. 目的 (Why)

旧デザインは単一アクセント（ピンク）+ 256 色未使用の単純配色で、パネル間の階層が弱く（header/footer が内容部と区別されない）、実行中ジョブが静止した `⇣` 表示で「動いているか」が一目で分からない。
モダンな TUI（fzf / lazygit 等）の慣行 — 統一的なダークパレット、角丸枠、chrome バー、アニメーション表示 — を導入し、情報階層と状態認識を改善する。

## 3. 変更範囲 (Scope)

変更対象:
- `src/ui.rs` — 全面書き換え（パレット定義 `mod theme` / ヘッダ・フッタの全幅バー / 入力 / プロファイルチップ / ジョブラスト / ログ / ポップアップ / 進捗バー）
- `src/app.rs` — `tick_count: u64` 追加（`on_tick` で +1）。**キー操作・状態ロジックには変更なし**
- `README.md` — 先頭の ASCII スクリーンショットを新デザインに同期
- ドキュメント: `task-list.md`（UI-1 登録）/ `audit/activity.md` / `ops/TERMUX_BATTERY.md`（誤記修正）/ `.agent/logs/`

変更しない (境界外):
- `src/runner.rs` / `src/config.rs` / `src/job.rs` / `src/main.rs`（ロジック不変）
- キーバインド・画面構成（7 画面）・HitAreas の機構（`app.rs` のマウス行計算は 1 ジョブ=2 行のまま）
- `asmr-dl.zip` / `target/`（不変）

## 4. 禁止事項

- 新クレートの追加（依存は現状維持 — AGENTS.md §6.1）
- `draw()` のシグネチャ変更（`main.rs` が呼ぶ `ui::draw(f, &mut app)` を維持）
- キー操作・キュー制御の挙動変更
- `unwrap()` / `unsafe` の追加
- ratatui の非推奨 API（例: `Paragraph::style` 等の deprecated）の採用

## 5. 完了条件 (DoD)

- [x] `src/ui.rs` を Catppuccin Mocha ベースの 13 色パレット + 角丸 + chrome バー + スピナーで全面書き換え
- [x] アニメーションは `App.tick_count` 由来（決定論 / `SystemTime` 使用なし → `determinism` スキル準拠）
- [ ] 検証: `cargo build --release` / `cargo test` / `cargo clippy`（Sandbox で実行不可 §6.2 → 代替検証実施: 括弧・中括弧のバランス / API 使用箇所を docs.rs ratatui 0.30 で照合）
- [ ] 実機検証: 縦 40 桁 / 60 桁でレイアウト崩れなし、ツールチップ・ハイライト・ポップアップの表示確認（device-testing の UI 系 D 項目）
- [x] 関連ドキュメント同期（README の ASCII / task-list / audit / 誤記修正）

## 6. テスト方法

- 単体テスト: TUI 描画は headless で動かないため対象外（`quality.md` 方針どおり）
- Sandbox 代替検証: 構文バランスチェック / ratatui 0.30 API 照合（docs.rs）/ 既存 `cargo test`（config・runner）は本変更の対象外だが CI 化時に必ず実行
- 実機検証: `asmr-dl` 起動 → 40 桁・60 桁でスクロール / ハイライト / ポップアップ（`Ctrl+P` / `?` / 終了確認）を確認

## 7. 停止条件 (Stop Criteria)

- cargo 実行環境が無いまま「検証済み」と報告しようとする場合（§6.2 の代替検証 + 実環境検証待ちの分離を守る）
- キー操作やキュー制御への変更が必要になった場合（本タスクの境界外）

## 8. 完了時に行うこと

- 検証結果を記録（本ファイル §12）
- `docs/task-list.md` の UI-1 状態・証拠を更新
- `.agent/logs/2026-10-04_ui_modern_tui.md` を作成
- commit（Conventional Commits + 日本語）+ push（セッションブランチ）

## 9. サブタスク分割

1. `src/app.rs` に `tick_count` を追加（1 行のフィールド + 初期化 + on_tick 増加）
2. `src/ui.rs` 全面書き換え（`mod theme` → 各 draw_* を新パレットで実装）
3. 代替検証（バランス / API 照合）
4. ドキュメント同期 + 記録

## 12. 実績と証拠（2026-10-04）

- `src/ui.rs`: 18.5KB → 25.6KB（Catppuccin Mocha 13 色 / ヘッダ・フッタの BG_ALT 全幅バー / プロファイルチップ / 実行中=ブレイル・スピナー / 進捗バー accent+surface / ハイライト `▸ ` + surface1 / ポップアップは BG 塗り + 角丸）
- `src/app.rs`: `tick_count: u64` 追加のみ（+4 行、diff で確認済み）
- 検証: 構文バランス OK（ui.rs / app.rs とも 0）/ 使用 API は ratatui 0.30 docs.rs で照合（`List::style` / `Block::title(Line)` / `set_cursor_position` / `offset_mut` など）
- **`cargo build/test` 未実行**（Sandbox の Rust egress 制約 §6.2）。実機・開発機での確認が完了条件の残項目。
