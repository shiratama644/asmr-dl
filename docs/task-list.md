# タスクリスト (唯一の正本)

> **運用規則** — Qiita「Claude Code／Codex に中〜大規模開発を任せるためのタスク管理」
> (<https://qiita.com/Y-Y-dev/items/d526fb7cdbe35a3f9384>) に基づく運用。
>
> 1. **本ファイルが進捗管理の唯一の正本**。チャット・Issue・AI の完了報告と本ファイルが
>    矛盾する場合は本ファイルを正とする。
> 2. **進行中タスクは原則 1 件**。複数を同時に進めない (独立性の高い調査・テストを除く)。
> 3. **タスク ID は再利用しない**。中止したタスクは行を消さず「対象外」にして理由を残す。
> 4. **作業中に見つけた新問題は新タスクとして登録**し、現在のタスクへ混ぜない
>    (現在の完了条件に必須の場合のみ例外)。
> 5. 完了は **AI の自己申告ではなく証拠で判定**する (テスト件数 / コミット SHA / PR / 実測値)。
>    Sandbox では cargo が実行できないため（AGENTS.md §6.2）、**「実環境検証待ち」状態を適切に使う**。
> 6. 個別タスクの詳細 (目的・変更範囲・禁止事項・完了条件・テスト方法・停止条件) は
>    `docs/planning/*_PLAN.md` (計画書テンプレート `_TEMPLATE.md` 準拠) に書く。
>
> **状態の定義**: `未着手` / `調査中` / `実装中` / `ローカル検証済み` /
> `実環境検証待ち` (実機・実機相当環境での確認が残る) / `完了` / `保留` (外部判断待ち) /
> `対象外` (中止・不採用。理由を残す)

---

## 未完了サマリー

> 完了していないタスクと残作業だけをここに列挙する（全件確認しなくて済むように）。
> 進行中のタスクが無ければ「なし」と書く。

| ID | 状態 | 残作業 |
|---|---|---|
| UI-1 | 実装中 | cargo build/test（Sandbox 制約 §6.2 により未実行）+ 実機でのレイアウト確認（縦 40/60 桁、D1〜D23 の UI 系） |

---

## タスク一覧

> タスクはテーマ（フェーズ・機能・運用）ごとに `###` 見出しで区切り、
> 下記テーブル形式で管理する。行は**追加のみ**（完了しても行は残し状態を更新する）。

### リポジトリ整備 (2026-10-04)

| ID | タスク | 状態 | 進捗 | 依存 | 完了条件 | 証拠 |
|---|---|---|---:|---|---|---|
| INIT-1 | asmr-dl.zip を解凍しルートディレクトリへ配置（src/ / scripts/ / Cargo.toml / Cargo.lock / config.default.toml） | 完了 | 100% | — | 1. zip 内のソースがルートに展開されている 2. 既存 .gitignore と競合しない（`target` 重複は許容） 3. asmr-dl.zip 自体は保持 | コミット: リポジトリ整備のコミット / 確認: `ls src/ scripts/ Cargo.toml config.default.toml` |
| INIT-2 | TEMPLATE_REPO の .agent/ / docs/ / AGENTS.md / README.md を asmr-dl 向けに書き換え適用 | 完了 | 100% | INIT-1 | 1. .agent/ が公式 .claude/ 構成に準拠（settings.json/rules/skills/agents/hooks/commands/output-styles/workflows/logs） 2. docs/ が arch/ + planning/ + research/ + audit/ + ops/ + examples/ + task-list.md 構成 3. AGENTS.md §6 が asmr-dl 固有（Rust/Termux/Sandbox §6.2） 4. README.md が asmr-dl のセットアップ・使い方 5. 代替検証全PASS（toml / sh -n / links / jq） 6. 完了ログ作成（.agent/logs/） | 検証: config.default.toml tomllib OK / scripts sh -n OK / docs links OK / settings.json jq OK / cargo build/test 未実行（§6.2: Sandbox に Rust ツールチェーン無し） |

### UI/UX (2026-10-04)

| ID | タスク | 状態 | 進捗 | 依存 | 完了条件 | 証拠 |
|---|---|---|---:|---|---|---|
| UI-1 | TUI を ratatui のモダンデザインに刷新（Catppuccin Mocha パレット / 角丸 / chrome バー / スピナー） | 実装中 | 90% | — | 1. `src/ui.rs` 全面書き換え（パレット・ヘッダ/フッタバー・ジョブラスト・ポップアップ） 2. `App.tick_count` による決定論的アニメーション 3. cargo build/test PASS（Sandbox 外） 4. 実機で縦 40/60 桁のレイアウト崩れなし（device-testing D1〜D23 の UI 系） | コード: `src/ui.rs`（25.6KB）/ `src/app.rs`（tick_count 追加）/ 計画: `docs/planning/UI_MODERN_TUI_PLAN.md` / build/test 未実行（§6.2: Sandbox に Rust ツールチェーン無し） |

### 品質・CI (将来の候補)

| ID | タスク | 状態 | 進捗 | 依存 | 完了条件 | 証拠 |
|---|---|---|---:|---|---|---|
| CI-1 | GitHub Actions 品質ゲートの導入（fmt/clippy/test/build） | 未着手 | 0% | — | 1. `.github/workflows/ci.yml` が push/PR で 4 ゲートを全PASS 2. `docs/arch/cicd.md` と一致 3. 実機確認不要（Linux runners） | — |
| DEV-1 | 実機(Termux)での動作確認マトリクス実行（device-testing D1〜D23） | 未着手 | 0% | — | device-testing スキルの D1〜D23 を実機で確認し、結果を証拠欄に回填 | — |
