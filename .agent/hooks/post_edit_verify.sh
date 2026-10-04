#!/bin/sh
# post_edit_verify.sh — 編集後の整合性を事後検証する PostToolUse フック
# asmr-dl 用に一般化（TEMPLATE_REPO / PalmIDE の Rule を参考に Rust 向けに調整）。
#
# 検証内容:
#   (A) 機密ファイル（.env / cookies）が追跡対象に含まれていないか
#   (B) docs/ 配下のファイル追加/削除時に docs/README.md の目次が更新されているか（警告のみ）
#   (C) config.default.toml が parse でき、generic が末尾プロファイルであるか
#   (D) Cargo.toml が parse でき、name = "asmr-dl" であるか
#
# 終了コード: 0 = OK, 1 = NG（警告）

set -u
cd "$(dirname "$0")/../.." || exit 1

FAIL=0

echo "=== post_edit_verify ==="

# (A) 機密ファイルの混入チェック
SECRET_HITS=$(git diff --cached --name-only 2>/dev/null; git diff --name-only 2>/dev/null; git ls-files --others --exclude-standard 2>/dev/null | head -n 50)
if echo "$SECRET_HITS" | grep -qE "(\.env$|\.env\.|cookies)"; then
  echo "❌ (A) .env / cookies ファイルが追跡対象に含まれています。gitignore を確認してください。" >&2
  FAIL=1
else
  echo "✅ (A) 機密ファイル: OK"
fi

# (B) docs/ の変更時に README.md の目次更新を促す（警告レベル）
DOCS_CHANGED=$(git status --porcelain 2>/dev/null | grep -E "docs/" | head -n 20)
if [ -n "$DOCS_CHANGED" ]; then
  if git diff --name-only 2>/dev/null | grep -q "docs/README.md"; then
    echo "✅ (B) docs/README.md 更新: OK"
  else
    ADDED_OR_DELETED=$(git status --porcelain 2>/dev/null | grep -E "^(\?\?|A |D |R )" | grep "docs/" || true)
    if [ -n "$ADDED_OR_DELETED" ]; then
      echo "⚠️ (B) docs/ に追加/削除がありますが docs/README.md が未更新です。目次の更新を検討してください。" >&2
    else
      echo "✅ (B) docs/ 変更: OK (README更新不要)"
    fi
  fi
else
  echo "✅ (B) docs/ 変更: なし"
fi

# (C) config.default.toml 整合性（TOML parse + generic 末尾）
if command -v python3 >/dev/null 2>&1 && [ -f config.default.toml ]; then
  if python3 -c "
import tomllib,sys
cfg = tomllib.load(open('config.default.toml','rb'))
profiles = cfg.get('profiles', [])
if not profiles: sys.exit('no profiles')
if profiles[-1].get('name') != 'generic': sys.exit('generic must be last')
names = [p.get('name') for p in profiles]
if len(names) != len(set(names)): sys.exit('duplicate profile name')
" 2>/dev/null; then
    echo "✅ (C) config.default.toml: OK (generic last)"
  else
    echo "❌ (C) config.default.toml: parse 失敗 or generic が末尾でない" >&2
    FAIL=1
  fi
else
  echo "ℹ️ (C) config.default.toml / python3 なし: スキップ"
fi

# (D) Cargo.toml 整合性
if command -v python3 >/dev/null 2>&1 && [ -f Cargo.toml ]; then
  if python3 -c "
import tomllib,sys
d = tomllib.load(open('Cargo.toml','rb'))
if d['package']['name'] != 'asmr-dl': sys.exit('package name != asmr-dl')
" 2>/dev/null; then
    echo "✅ (D) Cargo.toml: OK (name=asmr-dl)"
  else
    echo "❌ (D) Cargo.toml: parse 失敗 or name != asmr-dl" >&2
    FAIL=1
  fi
else
  echo "ℹ️ (D) Cargo.toml / python3 なし: スキップ"
fi

# (E) target/ の成果物がステージングされていないか（gitignore 済みのはず）
if git diff --cached --name-only 2>/dev/null | grep -q "^target/"; then
  echo "❌ (E) target/ のビルド成果物がステージングされています。gitignore を確認してください。" >&2
  FAIL=1
else
  echo "✅ (E) target/: OK (未ステージング)"
fi

if [ "$FAIL" -eq 0 ]; then
  echo "=== 検証完了 ==="
  exit 0
fi

echo "=== 検証失敗（上の ❌ を確認）===" >&2
exit 1
