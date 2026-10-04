# リポジトリ整備（asmr-dl.zip 解凍 + TEMPLATE_REPO 適用）

> Date: 2026-10-04(JST) / Commit: a013b9f / Branch: arena/01a10460-asmr-dl

## 1. 指示内容 (Task Summary)

asmr-dl.zip を解凍してルートディレクトリに置き、TEMPLATE_REPO（shiratama644/TEMPLATE_REPO）の `.agent/` / `docs/` / `AGENTS.md` / `README.md` をコピーしてこのリポジトリ（Rust / Termux）用にそれぞれ書き換える。

## 2. 実行内容 (Executed Actions)

| 内容 | 対象 |
|---|---|
| zip 解凍・ルート配置 | `src/`（6ファイル）/ `scripts/` / `Cargo.toml` / `Cargo.lock` / `config.default.toml`（`asmr-dl.zip` 自体は保持） |
| Agent 構成の全面書き換え | `.agent/` 42ファイル（rules 5 / skills 14 / agents 5 / hooks 8 / commands 3 / output-styles 2 / workflow 1 / settings.json / logs） |
| スキルの転用 | `e2e` → `device-testing`（実機 D1〜D23）/ `deep-dive-setup` → `site-onboarding` |
| ドキュメントの全面書き換え | `docs/` 30ファイル（arch 13 / planning / research / audit / ops 4 / examples 2 / task-list） |
| 規約の置換 | `AGENTS.md` §6 を asmr-dl 固有に（§6.2 = Sandbox 制約・代替検証ルール） |
| README | zip 内の日本語 README（12KB）を正に採用 + 開発規約の参照追加 |
| 代替検証 | tomllib（3 TOML）/ sh -n（4 shell）/ jq（settings.json）/ md リンク 103 件 — 全 PASS |

## 3. 気づいたこと・知見 (Insights & Lessons Learned)

- **Sandbox の Rust egress ブロックは恒常的**: `sh.rustup.rs` / `static.rust-lang.org` / `crates.io` が到達不可（2026-10-04 実測）。cargo のインストールは再試行しない（`sandbox-constraints` スキルに記録）。
- **zip の README に書かれていない事実**: 本リポジトリの `src/` は 6 ファイルのみ（`cli.rs` / `util.rs` / `main_ui.rs` は存在しない）。ドキュメントを書く時は**現物ソースを正**にする（当初の棚卸し記録で誤記 → activity.md で訂正済み）。
- **テンプレートの TS 特有の仕様書は削除してよいか判断**: `bootstrap` / `detector` / `cache` は cod-web 由来で本プロジェクトに該当しないため arch から削除し、代わりに `pipeline` / `profiles` を新設した（docs/README.md にその旨を明記）。
- **write_file の `fetch failed` は一過性**: 同じ内容で 1 回再試行すると成功する（docs/task-list.md で確認）。

## 4. 次にすべきこと (Next Actions)

- `cargo build/test` は未実行（§6.2）。CI 導入（CI-1）または開発機・実機での実行が必要。
- 実機検証マトリクス（DEV-1 / device-testing D1〜D23）の実施。
- `docs/planning/complete/` は空（INIT-1/2 は計画書なしで完了）。次回の新規タスクは `_TEMPLATE.md` 準拠の計画書を作成すること。
