# .agent/ — Agent設定ディレクトリ

> 本ディレクトリはClaude Code公式の `.claude/` ディレクトリ構成に準拠しつつ、ディレクトリ名は `.agent/` のまま維持しています。
> 公式仕様: https://code.claude.com/docs/en/claude-directory.md
> 出典: `shiratama644/TEMPLATE_REPO` を asmr-dl（Rust + Ratatui TUI）向けに書き換えたもの。

## ディレクトリ構成

```
.agent/
├── settings.json              # チーム共有設定（permissions, hooks, env）— コミット対象
├── settings.local.json        # 個人オーバーライド（gitignore）— 個人のみ
├── rules/                     # トピック別ルール（pathsで発火条件を絞れる）
│   ├── 01_information-hierarchy.md
│   ├── 02_git-workflow.md
│   ├── 03_doc-style.md
│   ├── 04_verification.md
│   └── project-template.md    # asmr-dl 固有の遵守事項（AGENTS.md §6 と対応）
├── skills/<name>/SKILL.md     # 再利用プロンプト（/nameで呼び出し、自動発火も可能）
│   ├── project-overview/      # 製品概要・構成把握
│   ├── tech-stack/            # Rust/Cargo/外部ツール・ハマりどころ
│   ├── sandbox-constraints/   # Sandbox制約・迂回策（2026-10-04実測）
│   ├── verify-doc-integrity/  # ドキュメント整合性検証
│   ├── diff-review-report/    # 差分レビューレポート
│   ├── docs-maintenance/      # ドキュメント整理・URL検証
│   ├── ci-quality-gates/      # 品質ゲート・CI（現状: ローカルゲートのみ）
│   ├── testing/               # Rust 単体テスト・意味あるアサーション
│   ├── import-boundaries/     # モジュール境界・依存方向
│   ├── determinism/           # 純粋関数・決定論・same-inputテスト
│   ├── memory-leak/           # プロセス・チャネル・タスクのリソース解放
│   ├── zero-alloc/            # ホットパスのアロケーション削減
│   ├── device-testing/        # 実機(Termux)でのTUI検証・テストマトリクス
│   └── site-onboarding/       # 新サイト対応（プロファイル追加）の一気通貫手順
├── agents/                    # サブエージェント定義（name, description, tools, model等）
│   ├── explore.md
│   ├── plan.md
│   ├── doc-editor.md
│   ├── code-reviewer.md
│   └── test-writer.md
├── hooks/                     # フック手順(.md) + 実行スクリプト(.sh)
│   ├── index.md               # 索引
│   ├── pre-task.md            # タスク開始時
│   ├── verify-commit.md       # commit直前
│   ├── log-task.md            # タスク完了時
│   ├── sandbox-recovery.md
│   ├── restore-env.sh
│   ├── pre_edit_guard.sh      # PreToolUse: 編集禁止領域ブロック
│   └── post_edit_verify.sh    # PostToolUse: 事後検証
├── commands/                  # 旧commands互換（新しくはskills/を使う）
│   ├── commit.md
│   ├── review.md
│   └── test.md
├── output-styles/             # 出力スタイル（concise, detailed等）
│   ├── concise.md
│   └── detailed.md
├── workflows/                 # 動的ワークフロー（複数サブエージェントを束ねる）
│   └── implement-task.js
├── agent-memory/              # サブエージェント永続メモリ（自動生成、gitignore）
└── logs/                      # タスク実行ログ（追加のみ、書き換え禁止）
    └── YYYY-MM-DD_<summary>.md
```

## 各ファイルの役割（公式準拠）

| ファイル | Scope | Commit | 役割 | 参考 |
|---|---|---|---|---|
| `settings.json` | Project | ✓ | Permissions, hooks, env | [Settings](https://code.claude.com/docs/en/settings-reference.md) |
| `settings.local.json` | Project |  | 個人オーバーライド、gitignore | [Settings scopes](https://code.claude.com/docs/en/settings-reference.md) |
| `rules/*.md` | Project | ✓ | トピック別ルール、pathsで発火条件 | [Rules](https://code.claude.com/docs/en/memory#organize-rules-with-claude/rules/) |
| `skills/<name>/SKILL.md` | Project | ✓ | 再利用プロンプト、/nameで呼び出し | [Skills](https://code.claude.com/docs/en/skills.md) |
| `commands/*.md` | Project | ✓ | 単一ファイルプロンプト、skillsと同じ機構 | [Skills](https://code.claude.com/docs/en/skills.md) |
| `output-styles/*.md` | Project | ✓ | 出力スタイルのカスタマイザ | [Output styles](https://code.claude.com/docs/en/output-styles.md) |
| `agents/*.md` | Project | ✓ | サブエージェント定義 | [Subagents](https://code.claude.com/docs/en/sub-agents.md) |
| `workflows/*.js` | Project | ✓ | 動的ワークフロー | [Workflows](https://code.claude.com/docs/en/workflows.md) |
| `agent-memory/<name>/` | Project |  | サブエージェント永続メモリ（gitignore） | [Persistent memory](https://code.claude.com/docs/en/sub-agents#enable-persistent-memory) |
| `logs/` | Project | ✓ | タスク実行ログ | テンプレート独自 |

## 運用ルール

- **settings.json** はチーム共有。permissions, hooks, envを定義。個人の上書きは `settings.local.json` へ。
- **rules/** はトピック別に分割。`paths` フロントマターで発火条件を絞る。例: `paths: ["docs/**"]` ならdocs編集時のみ読まれる。
- **skills/** は `<name>/SKILL.md` 形式。frontmatterに `name`, `description` 必須。`description` は自動発火の判断材料なので具体的に書く。
- **agents/** は `name`, `description`, `tools`, `model` 等のfrontmatter + システムプロンプト本文。
- **hooks/** は手順(.md)と実行スクリプト(.sh)の両方を置く。実行登録は `settings.json` の `hooks` で行う。
- **logs/** は追加のみ。過去ログを書き換えない。

## asmr-dl への適用時の変更点（TEMPLATE_REPO からの差分）

- テクノロジー特化の記述を Rust/Cargo/Termux 向けに書き換え（pnpm/Node/Playwright 関連は全て削除）。
- スキル差し替え 2 件:
  - `e2e/`（Playwright）→ `device-testing/`（TUI はブラウザ E2E が無いため実機検証マトリクスへ転用）
  - `deep-dive-setup/`（pnpm setup 主導の一気通貫）→ `site-onboarding/`（「サイト X が落ちない」→ 調査 → プロファイル追加）
- hooks の検証対象を `package.json`/`node_modules` 系から `Cargo.toml`/`target/` 系へ置換。
- `settings.json` の permissions を cargo 系コマンドへ置換。
