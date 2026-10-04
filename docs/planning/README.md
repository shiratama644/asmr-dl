# Planning Index — asmr-dl

計画書（`docs/planning/`）は、`docs/task-list.md` の各タスクを **どの順で、どの範囲で、何をもって完了とするか** に分解する場所です。仕様そのものの正本は `../arch/` です。完了済み計画は `complete/` に置きます。

## まず読むもの

| 順 | 文書 | いつ読むか | 内容 |
|---:|---|---|---|
| 1 | `../task-list.md` | 常に最初 | 状態・依存・次に着手できるタスクの唯一の正本 |
| 2 | 対象タスクの `*_PLAN.md` | 実装/調査に入る前 | 変更範囲、禁止事項、DoD、停止条件、検証方法 |
| 3 | `../research/README.md` | 外部技術・調査の根拠が必要な時 | 調査の入口 |
| 4 | `../arch/README.md` | 仕様確認が必要な時 | 仕様書一覧 |

## 計画書一覧（asmr-dl）

| 文書 | 対応 ID | 状態 | 役割 |
|---|---|---|---|
| `_TEMPLATE.md` | — | 現用 | 新規計画書の必須形式 |
| `index.md` | — | 現用 | フォルダ説明 |
| `UI_MODERN_TUI_PLAN.md` | UI-1 | 実装中 | TUI のモダンデザイン刷新（Catppuccin Mocha） |
| `complete/README.md` | — | 現用 | 完了済み計画の索引 |

> 進行中: UI-1（実装済み、検証待ち — Sandbox に Rust ツールチェーン無し §6.2）。INIT-1/INIT-2 は完了、CI-1/DEV-1 は未着手。

## 次に着手可能なタスク

| 優先 | ID | 内容 | 事前に読むもの |
|---:|---|---|---|
| 1 | CI-1 | CI 導入（fmt/clippy/test/build） | `../arch/cicd.md` / `.agent/skills/ci-quality-gates/` |
| 2 | DEV-1 | 実機検証マトリクス実行（D1〜D23） | `.agent/skills/device-testing/` |

## 計画書を書く/更新する時のルール

- 新規タスクは先に `../task-list.md` へ ID を追加する
- 新規計画書は `_TEMPLATE.md` の §1〜§9 を最低限満たす
- 実装範囲、禁止事項、DoD、停止条件を必ず書く
- 計画書と `../arch/` が矛盾したら、勝手に片方を正にせずユーザーへ確認する
- Sandbox で cargo が実行できない場合は、§6.2 の代替検証 + 実機検証の分離を §5/§6 に明記する
- 完了後は「実績と証拠」に commit / validation を書く
- ファイル名は短く正確、ハイフン最大1つ

## 完了済み計画の扱い

完了した計画書は `complete/` へ移動。過去の記録は書き換えない。新規は `_TEMPLATE.md` 準拠で作成。
