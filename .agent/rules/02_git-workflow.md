---
paths:
  - ".git/**"
  - ".agent/hooks/**"
  - "AGENTS.md"
---

# Rule 02: Git運用 & 環境復旧ルール

> 優先度: **CRITICAL** — 実行環境消失時の復旧チェックポイントとしてGitを扱う

## 1. 作業開始時の現状把握（必須）

```bash
git status
git branch --show-current
git log -5 --oneline
```

- 未コミット変更があれば勝手に破棄・混入しない。
- セッション型環境（Arena等）ではブランチ名がセッションごとに変わる。必ず `git branch --show-current` で確認。

### Sandbox再構築の検知

以下が全て該当したらSandbox再構築と判定：

- `git log --oneline` が起点コミット1件のみ
- `git status` が「大量の削除 + 大量の未追跡」
- `target/` が無い

→ `.agent/hooks/sandbox-recovery.md` + `restore-env.sh` で復旧。

```bash
git fetch origin <現在のブランチ>
git reset --hard FETCH_HEAD   # この場合のみ --hard 許可（AGENTS.md §4.1.1例外）
bash .agent/hooks/restore-env.sh
```

## 2. コミットルール

- **タイミング**: 検証（fmt/clippy/test/build、実行可能な範囲 — AGENTS.md §6.2）が全PASSした状態でのみコミット。
- **事前チェック**: `git status` / `git diff` で意図しない差分が無いか確認。
- **メッセージ**: Conventional Commits形式 + 日本語
  - `feat:`, `fix:`, `refactor:`, `perf:`, `test:`, `docs:`, `chore:`, `build:`, `ci:`
  - 例: `feat(profiles): bilibili の並列フラグメント数を 8 に増加`

## 3. 厳禁なGit操作（明示的指示がない限り禁止）

- `git reset --hard` / `git clean -fd`（例外はSandbox再構築時の `reset --hard FETCH_HEAD` のみ）
- `git rebase` / `git commit --amend`
- `git push --force` / `--force-with-lease`

## 4. ブランチ運用（セッション型環境）

- **作業ブランチはセッション固定**。Arenaはこのブランチ名でセッションを追跡。
- ブランチ名はセッションごとに変わるため、文書に過去のブランチ名を残さない。
- feature branchは切らない。セッション固定ブランチへ直接commit + pushし、`gh pr create` で `main` 向けPRを作成。
- pushは `git push origin <現在のブランチ>` の明示指定で行う。

## 5. Pushの事前許可（恒久合意）

- Sandboxは予告なく再構築され、ローカルコミットのみだと作業が破棄される。
- 検証が全PASS（§6.2 の代替検証を含む）し意図しない差分が無いことを確認できたら、その場で `git push origin <セッション固定ブランチ>` を実行（pushのたびに確認を取らない）。
- push先はセッション固定ブランチのみ。`main`等への直接push、他ブランチへのpush、force pushは禁止。
- PR作成も許可済み（`gh pr create`）。
