---
description: 変更内容をレビューし、バグ・セキュリティ・スタイルの問題を指摘する。Use when you want to review changes before merging.
---

# Review Command

変更内容をレビューする手順。`code-reviewer` エージェントを活用する。

## 手順

1. **変更範囲の確認**:
   ```bash
   git diff --stat HEAD~1 HEAD
   git diff HEAD~1 HEAD
   ```

2. **サブエージェントにレビューを依頼**:
   - `code-reviewer` エージェントに差分を渡し、レビューを依頼
   - Must / Should / Nit の3段階で指摘を分類してもらう

3. **レビュー観点**:
   - バグ: ロジックエラー、境界条件、`Option`/`Result` の誤扱い、非同期処理、panic 原因
   - セキュリティ: 機密情報のハードコード（cookies / token）、ファイル名の注入（パス走査）
   - スタイル: 命名規則、重複コード、`AGENTS.md §6` 違反（モジュール境界・エラー処理・プロセス管理）
   - Rust 固有: `unsafe` の正当性、`unwrap()`/`expect()` の乱用、`#[allow]` の理由
   - テスト: 網羅性、エッジケース、shallow test

4. **レポート作成**:
   - 指摘事項を `docs/audit/review-{date}.md` に保存（必要に応じて）
   - 指摘がMustなら修正を促す

## 出力例

```markdown
## レビュー結果

### Must
- `src/runner.rs:45` — キャンセル時に子プロセス（yt-dlp の子 ffmpeg）が kill されない（process group kill 漏れ）

### Should
- `src/app.rs:30` — 履歴バッファに cap がない（長時間実行で増加）

### 良い点
- 純粋関数にロジックを分離し、`#[cfg(test)]` でテストしている
```
