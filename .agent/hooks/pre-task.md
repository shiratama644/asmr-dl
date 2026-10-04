# Hook: Pre-Task（タスク開始時）

> **トリガー**: ユーザーから指示を受け、作業を開始する直前。`settings.json` の `UserPromptSubmit` でも自動実行。
> **目的**: 現状を把握し、必要な知識だけを読み込み、スコープ違い/履歴破壊を防ぐ。

## 手順

### 1. 現状把握（AGENTS.md §4.1 + rules/02_git-workflow.md）

```bash
git status
git branch --show-current
git log -5 --oneline
```

- ※ セッション型環境ではブランチ名がセッションごとに変わる。必ず `git branch --show-current` で確認する（AGENTS.md §4.4）。
- 未コミット変更があれば勝手に破棄・混入しない。
- ログが起点 1 件のみ / `git status` が大量の削除+未追跡 / `target/` 無 → **Sandbox 再構築**。→ [`sandbox-recovery.md`](./sandbox-recovery.md)。

### 2. 知識のピンポイント読込（本 hook の核心）

[`../skills/index.md`](../skills/index.md) の「読み方ガイド」で**該当スキルだけ**を読む。
- 全スキルを常に読まない（コンテキスト浪費）。
- 初回/全体把握が必要な時だけ `project-overview/SKILL.md` を読む。
- 例: 環境トラブル → `sandbox-constraints/SKILL.md`、実機検証 → `device-testing/SKILL.md`、新サイト → `site-onboarding/SKILL.md`、ドキュメント編集 → `verify-doc-integrity/SKILL.md`
- ルールは `rules/` を参照。`paths` フロントマターで発火条件を絞っているため、該当ファイル編集時のみ自動で読まれる。

### 3. リポジトリ固有の制約の確認

- **Sandbox では Rust ツールチェーンが無い**（AGENTS.md §6.2）。`cargo build/test/clippy/fmt` は実行できない。
  - 代替検証: tomllib（TOML）/ `sh -n`（shell）/ jq（JSON）/ リンクチェック
  - 最終検証は CI（未導入時は実機）へ分離
- `asmr-dl.zip` / `target/` は不変。`config.default.toml` は `include_str!` で埋め込み（`Config::normalize()` と矛盾させない）。
- 不変ディレクトリ・書き込み不可領域・恒常的な環境制約など、リポジトリ固有の制約が `AGENTS.md` §6 にあれば作業前から意識しておく。

### 4. ドキュメントと実コードの優先順位

- 計画書や仕様書と実コードが矛盾する場合は、プロジェクトの定めた優先順位に従う（AGENTS.md §6）。
- 定めがない場合は実コードを最終確認し、判断に迷えばユーザーに質問する。

### 5. タスク粒度の確認（AGENTS.md §1.2）

1 タスク = 1 つの意味のある論理的単位。「ついでに」スコープを広げない。

## 完了後

→ 実装 → [`verify-commit.md`](./verify-commit.md) で検証 → commit/push → [`log-task.md`](./log-task.md) でログ記録。
