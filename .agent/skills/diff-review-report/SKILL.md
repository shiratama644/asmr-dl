---
name: diff-review-report
description: 仕様書・設定変更の差分を、人間がレビュー可能なレポートにまとめる手順。Use when changes involve 3+ files or require human review.
---

# Skill: 差分レビューレポートの作成

TEMPLATE_REPO の `diff-review-report` を引き継ぐ。変更が3ファイル以上または判断を伴うとき、必ず作成するレビュー用レポートの手順。

## 形式（MERGE_DIFF_REPORT.md 準拠）

```markdown
# 差分レビュー報告書 — X → Y

> 作成日: YYYY-MM-DD / 対象: （旧→新の範囲）
> 目的: 何をレビューしてほしいか1行で

## 1. 変更の方針        — 数化できない判断の宣言（箇条書き5点程度）
## 2. 全体サマリ        — 総行数、ファイル数、変更種別の表
## 3. 行数変化（ファイル別）— 旧/新/差分/主な変化理由の4列表
## 4. 意図的な変更点     — (1) 本文改変 (2) 削除 (3) 圧縮・簡素化 (4) リンク修正
## 5. 機械検証結果       — grep/リンク/TOML解析の検証結果をコマンドと期待値つきで
## 6. レビューポイント    — 人間判断を仰ぐ事項を表で。理由と懸念を必ずセット
## 7. まとめ
```

## 作成手順

1. `wc -l` で旧新の全ファイル行数を取る
2. `git diff --stat` で変更範囲を確定
3. 差分を「追加 / 削除 / 圧縮 / リンク修正」の4分類に仕分け
4. 人間判断を仰ぐ事項は表に集約し、理由と懸念を必ずセットで書く
5. `config.default.toml` の変更がある場合は、**影響を受けるプロファイル一覧**（which profiles changed, どの audio 項目か）を必ず §4 に含める

## 品質基準

- 数値は `wc` / `grep` の実測値のみ使う（推定で書かない）
- 「変えたつもり」ではなく「実際に変えた場所」を `git diff` で確認してから書く
- レポート本文は「削除した」か「まとめた」かを正確に区別して書く
- **既定値の変更**（`config.default.toml` の numeric / boolean）は必ず「旧値 → 新値 + 理由」の表で示す

## 保存場所

- `docs/audit/diff-{context}.md` に保存
- 時点記録のため、後から書き換えない。修正は新レポートで。

## テンプレート例

```bash
# 行数取得
wc -l docs/**/*.md src/*.rs

# 変更範囲
git diff --stat HEAD~1 HEAD

# プロファイル数・名前の確認（config.default.toml 変更時）
python3 -c "import tomllib; print([p['name'] for p in tomllib.load(open('config.default.toml','rb')).get('profiles',[])])"
```
