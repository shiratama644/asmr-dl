---
name: verify-doc-integrity
description: ドキュメント・設定の整合性（リンク実在・目次更新・機密情報混入なし・TOML構文）を機械検証する手順。docs/ や config.default.toml 変更後・コミット前に必須。Use when docs or config are changed or before committing.
---

# Skill: ドキュメント・設定整合性の機械検証

TEMPLATE_REPO の `verify-doc-integrity` を asmr-dl（Rust）向けに一般化した機械検証手順。
`package.json` 整合性の代わりに **`Cargo.toml` / `config.default.toml` 整合性**を使う。

## いつ使うか

- `docs/` のファイルを1行でも編集した直後
- `config.default.toml` を編集した直後（`include_str!` でバイナリに埋め込まれるため特に重要）
- PRをopen/updateする直前
- 「この変更でドキュメントが壊れていないか」と聞かれたとき

## 手順（順序厳守）

### Step 1. 機密情報の混入チェック

```bash
# .env / cookies が追跡対象になっていないか
git diff --cached --name-only | grep -E "(\.env|cookies)"
# 期待: 出力なし

# .env.* がステージングされていないか
git diff --cached --name-only | grep "^\.env\."
# 期待: 出力なし
```

> cookies.txt（Netscape 形式）は**ユーザーの個人ファイル**（`~/cookies/...`）。リポジトリに含めてはならない。

### Step 2. 内部リンク検証

```bash
# docs/ 内の相対リンクが実在するか
cd docs
grep -ohr "](\./[^)]*)" *.md **/*.md 2>/dev/null | sed 's/](\.\///; s/)$//; s/#.*//' | sort -u | while read -r f; do [ -f "$f" ] || echo "BROKEN: $f"; done
cd ..
# 期待: BROKEN 0件
```

### Step 3. 目次更新チェック

```bash
# docs/ に追加・削除があるのに README が未更新なら警告
git status --porcelain | grep "docs/" | grep -E "^(\?\?|A |D |R )"
# あれば docs/README.md の更新を検討
git diff --name-only | grep "docs/README.md"
```

### Step 4. 変更狙いの見読み

```bash
git status --short
git diff --stat
# 狙いではないファイルが変更されていないか目視
```

### Step 5. 設定整合性（Rust 向け）

```bash
# config.default.toml が parse でき、generic が末尾にあるか
python3 - <<'PY'
import tomllib
cfg = tomllib.load(open('config.default.toml','rb'))
profiles = cfg.get('profiles', [])
assert profiles and profiles[-1].get('name') == 'generic', "generic must be last"
names = [p.get('name') for p in profiles]
assert len(names) == len(set(names)), f"duplicate profile name: {names}"
print(f"config OK: {len(profiles)} profiles (last=generic)")
PY

# Cargo.toml が parse できるか（[package] name = "asmr-dl"）
python3 -c "import tomllib; d=tomllib.load(open('Cargo.toml','rb')); assert d['package']['name']=='asmr-dl'; print('Cargo.toml OK')"

# .agent/settings.json が JSON か
jq empty .agent/settings.json
```

## 完了報告のフォーマット

```
- (A) 機密情報: OK (0件)
- (B) リンク: OK (BROKEN 0件)
- (C) 目次: OK / 要更新
- (D) 変更狙い: 意図した N ファイルのみ
- (E) 設定: config OK (N profiles) / Cargo.toml OK / settings.json OK
```

どれか1つでもNGが出たら「完了」宣言はしない。先に直す。

## 自動化

`.agent/hooks/post_edit_verify.sh` がこの検証の一部（A / B / E）を自動実行する。手動でも上記コマンドで確認可能。
