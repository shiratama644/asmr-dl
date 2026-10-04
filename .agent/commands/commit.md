---
description: 変更を検証し、Conventional Commits形式でコミットする。Use when you want to commit changes after verification.
---

# Commit Command

変更を検証し、コミットする手順。

## 手順

1. **現状確認**:
   ```bash
   git status
   git diff --stat
   git log -5 --oneline
   ```

2. **検証** (cargo が使える環境では4種、Sandbox では §6.2 の代替検証):
   ```bash
   # cargo がある環境
   cargo fmt --all -- --check
   cargo clippy --all-targets -- -D warnings
   cargo test
   cargo build --release

   # Sandbox（cargo 無し、AGENTS.md §6.2）
   python3 -c "import tomllib; tomllib.load(open('config.default.toml','rb')); print('OK')"
   sh -n scripts/install-termux.sh
   sh .agent/hooks/post_edit_verify.sh
   ```
   1つでも失敗したら原因を特定して修正し、再検証。

3. **差分確認**:
   ```bash
   git diff
   # 意図しない変更が含まれていないか確認
   git diff --cached --name-only | grep -E "(\.env|cookies|target/)"   # 機密・成果物の混入
   ```

4. **コミット**:
   ```bash
   git add <対象ファイル>
   git commit -m "feat: <日本語で変更内容>"
   ```
   - Conventional Commits形式（`feat:`, `fix:`, `refactor:`, `perf:`, `test:`, `docs:`, `chore:`, `build:`, `ci:`）
   - 日本語で書く
   - 例: `feat(profiles): bilibili の並列フラグメント数を 8 に増加`
   - Sandbox で cargo が実行できなかった場合は、コミット本文に「build/test 未実行（§6.2）」を追記する

5. **プッシュ** (検証PASS後):
   ```bash
   git push origin <現在のブランチ>
   ```

## 禁止事項

- 検証がFAILした状態でコミットしない
- `git reset --hard` / `git push --force` は使わない（§4.1.1 の Sandbox 復旧時の `reset --hard FETCH_HEAD` のみ例外）
- 意図しないファイルをコミットに含めない
- `target/` / `asmr-dl.zip` / 機密ファイル（.env, cookies）をコミットしない
