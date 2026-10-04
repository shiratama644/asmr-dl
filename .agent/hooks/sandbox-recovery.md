# Hook: Sandbox Rebuild Recovery

> **トリガー**: Sandbox 再構築を検知した時。AGENTS.md §4.1.1 の手順実体版。
> **検知ヒント**: `git log --oneline` が起点コミット 1 件のみ / `git status` が「大量の削除 + 大量の未追跡」/ `target/` が無い。

## 背景

サンドボックス型の作業環境（Arena 等）は再構築されることがあり、その場合ワークツリーは
「起点コミットのファイル」＋「push 済みコミットで追加されたファイルの未追跡バージョン」が混在した状態で立ち上がる。
ファイルは破損していないので、以下で確実に復旧する。

## 手順

```bash
# 1. リモートの最新を fetch（※ ブランチ名は git branch --show-current で確認）
git fetch origin <session-branch>

# 2. FETCH_HEAD にワークツリーごとリセット
#    （この場合の --hard は §4.3 厳禁ルールの例外 = Sandbox 再構築後の初回のみ許可。
#     未コミット変更は元々存在しない状態のため安全）
git reset --hard FETCH_HEAD

# 3. 依存を再構築（下記スクリプト、または手動）
bash .agent/hooks/restore-env.sh
```

## 復旧後の健全性確認

```bash
git log --oneline -5          # push 済みコミットが見えること
# cargo が使える環境では:
cargo test                    # テストが通ること
# Sandbox（cargo 無し）では代替検証:
python3 -c "import tomllib; tomllib.load(open('config.default.toml','rb')); print('toml OK')"
sh -n scripts/install-termux.sh
```

→ 問題なければ作業再開。

## 注意

- `git reset --hard` は**この例外場景以外では厳禁**（AGENTS.md §4.3）。誤用に注意。
- 復旧スクリプト `restore-env.sh` は cargo の有無を検出する。
  Sandbox（cargo 無し・egress 制限）では「ツールチェーン無し（§6.2）」を記録し、代替検証へ誘導する。
