# 計画書テンプレート (新規計画書は本形式で作成する)

> **形式の出典**: Qiita「Claude Code／Codex に中〜大規模開発を任せるためのタスク管理」
> <https://qiita.com/Y-Y-dev/items/d526fb7cdbe35a3f9384>
>
> - **進捗の正本は `../task-list.md`**。計画書は個別タスクの詳細 (目的・範囲・条件) を担う。
> - 各計画書は本テンプレートの §1〜§9 を必須セクションとし、
>   §10〜§12 (設計詳細・Gotchas・実績) を必要に応じて増減する。
> - 新規タスクを計画する場合は必ず `task-list.md` に行を追加 (ID 採番) してから本テンプレートで詳細化する。

---

```markdown
# <タスク名>: <タイトル>

> 対応 task-list ID: `TASK-ID` (docs/task-list.md)
> 計画書テンプレート: docs/planning/_TEMPLATE.md 準拠

## 1. 開始前確認

- 現在のブランチ / HEAD / `git status` を確認する (未コミット変更があれば停止)
- `docs/task-list.md` で依存タスクの完了を確認する
- 関連仕様 (AGENTS.md §6 / .agent/skills/ / docs/arch/) を読む
- 本計画書の §5 (完了条件) と §7 (停止条件) を再読する

## 2. 目的 (Why)

<!-- 「何を」だけでなく「なぜ」を書く。悪い例: 「○○を直して」。
     良い例: 「yt-dlp が JS チャレンジで落ちる YouTube を安定取得できるようにする。
     原因は JS ランタイム不在で、nodejs + yt-dlp-ejs + --js-runtimes で対処する」 -->

## 3. 変更範囲 (Scope)

<!-- 調査・変更してよいファイル・機能・ドキュメントを列挙。
     「task-list.md と関連ドキュメントの更新」を忘れない。
     変更しないものも明示すると境界が明確になる -->

変更対象:
-

変更しない (境界外):
- `asmr-dl.zip` / `target/`（不変）
- 既存プロファイルの既定値（新規追加のみ）

## 4. 禁止事項

<!-- 推測で埋めてはいけない仕様・破壊的変更・守る不変条件 -->

- 既存テストの削除・スキップ・アサーションの緩和
- `unwrap()` / `unsafe` / `#[allow]` の安易な追加
- `config.default.toml` の `generic` の末尾位置を変更しない
- モジュール境界（import-boundaries）を壊さない

## 5. 完了条件 (DoD)

<!-- 第三者が Yes/No で判定できる条件を列挙。検証方法も必ず書く -->
<!-- Sandbox で cargo が実行できない場合は §6.2 の代替検証 + 実機検証の分離を明記 -->

- [ ]
- [ ] 検証: cargo fmt/clippy/test/build（か §6.2 の代替検証 + 残課題明記）
- [ ] 関連ドキュメント（arch/ / examples/ / README）を同期
- [ ] task-list.md の状態・証拠を更新

## 6. テスト方法

<!-- 単体テスト（#[cfg(test)]）/ 実機検証（device-testing の D番号）/ 代替検証 -->

- 単体テスト:
- 実機検証: device-testing D_
- Sandbox 代替検証:

## 7. 停止条件 (Stop Criteria)

<!-- 続けてはならない状況。迷ったら停止してユーザーに確認 -->

- 完了条件が満たせない
- 推測で埋めなければならない仕様に出会った
- 破壊的変更（既定値変更・新クレート導入）が必要になった
- 他タスクと競合する差分が発生した

## 8. 完了時に行うこと

- 検証結果を記録（コマンド + 結果）
- `docs/task-list.md` の状態・証拠を更新
- `docs/audit/diff-{context}.md`（計画から外れた差分が varsa）
- `.agent/logs/YYYY-MM-DD_<summary>.md` を作成（log-task）
- 知見を `.agent/skills/` へ同期（skills/index.md も更新）
- commit（Conventional Commits + 日本語）+ push（セッションブランチ）

## 9. サブタスク分割

<!-- 1タスク=1論理単位。大きすぎれば分割 -->

1.
2.

## 10. 設計詳細（任意）

<!-- 関数シグネチャ・データ構造・メッセージフローなど -->

## 11. Gotchas（任意）

<!-- 既知の落とし穴・外部依存の挙動 -->

## 12. 実績と証拠（完了時に記入）

<!-- commit SHA / 検証結果 / 実測値 -->
```
