#!/bin/sh
# pre_edit_guard.sh — 編集禁止領域への変更をブロックする PreToolUse フック
# asmr-dl 用に調整（TEMPLATE_REPO の PalmIDE 設計ベース）。
#
# ブロック対象:
#   .git/ 以下（Git 内部状態への直接書き込み禁止）
#   target/ 以下（ビルド成果物、gitignore 済み）
#   *.zip / *.tgz（asmr-dl.zip 等の配布物アーティファクト。編集・再打包禁止）
#
# 終了コード:
#   0 = 許可, 2 = ブロック

set -u

TARGET=""

# 1) 引数運び（手動実行用）
if [ $# -ge 1 ]; then
  TARGET="$1"
else
  # 2) Claude Code hooks（stdin JSON）から tool_input.file_path を抜く
  INPUT=$(cat 2>/dev/null || true)
  TARGET=$(printf '%s' "$INPUT" \
    | sed -n 's/.*"file_path"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' \
    | head -n1)
fi

# 空なら許可（ファイルパスが取れないツール呼び出し）
case "$TARGET" in
  "") exit 0 ;;
esac

# 正規化: ./ プレフィックス除去
case "$TARGET" in
  ./*) TARGET="${TARGET#./}" ;;
esac

# .agent/logs/ は新規作成は許可（log-task 経由）、既存の上書きは禁止
# ここでは簡易的に許可し、post_edit で「追加のみ」を検証する
case "$TARGET" in
  */.agent/logs/*) : ;;
esac

is_protected() {
  case "$1" in
    .git/*|.git) return 0 ;;
    target/*|target) return 0 ;;
    *.tgz|*.zip) return 0 ;;   # asmr-dl.zip 等のアーティファクト
    *) return 1 ;;
  esac
}

if is_protected "$TARGET"; then
  {
    echo "⛔ [pre_edit_guard] 編集禁止領域です: $TARGET"
    echo "   .git/, target/, *.zip/*.tgz への直接編集は禁止されています。"
    echo "   asmr-dl.zip は初期ソース配布物（アーティファクト）として保持します（AGENTS.md §6.3）。"
  } >&2
  exit 2
fi

exit 0
