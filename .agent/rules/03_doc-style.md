---
paths:
  - "docs/**/*.md"
  - "README.md"
  - "AGENTS.md"
  - "config.default.toml"
  - "src/**/*.rs"
---

# Rule 03: ドキュメント記述スタイル

> 優先度: **HIGH** — 可読性と再利用性の一貫性を保つ

## 1. 言語と表記

- 本文は**日本語**。コード識別子・技術固有名詞は英語のまま。
- 「〜です/ます」調。計画書の完了条件などは断定形「〜する」。
- 絵文字は既存ファイルのパターンに従う。新規の飾り絵文字は増やさない（可読性優先）。
- コード内のコメント・UI 文案・ログも日本語（`src/` 既存コードの運用に従う）。

## 2. ファイル構成テンプレート

### 仕様書 (`docs/arch/`)

- `product.md` — プロダクト定義・用語
- `architecture.md` — モジュール構造・依存規則・メッセージフロー
- `pipeline.md` — ダウンロード/変換パイプライン（yt-dlp 経路 / ffmpeg 経路）
- `profiles.md` — サイト別プロファイルと自動判定
- `tech-stack.md` — 技術スタック（Rust + 外部ツール）
- `termux.md` — Termux 連携・端末制約
- `quality.md` — 品質ゲート・テスト方針
- `security.md` — セキュリティ・プライバシー
- `engineering.md` — エンジニアリング規約（エラー処理・プロセス管理）
- `cicd.md` — CI/CD（現状: 未導入、ローカルゲート中心）
- `adr.md` — 意思決定ログ
- `milestones.md` — フェーズと完了条件

### 計画書 (`docs/planning/{TOPIC}_PLAN.md`)

`docs/planning/_TEMPLATE.md` 準拠。必須セクション: 開始前確認, 目的, 変更範囲, 禁止事項, 完了条件, テスト方法, 停止条件, 完了時に行うこと, サブタスク分割, 設計詳細, リスク, 実績と証拠

### 調査 (`docs/research/`)

- `{TOPIC}_RESEARCH.md` — 調査結果

### 監査 (`docs/audit/`)

- `diff-{context}.md` — 差分レポート
- `issues-{context}.md` — バグリスト

### 完了済み計画 (`docs/planning/complete/`)

- `{TOPIC}_COMPLETE.md` — 完了レポート

### 設定例 (`docs/examples/`)

- `profile-examples.toml` — プロファイル定義例（`config.default.toml` の `[[profiles]]` 形式）
- `cookies.example.txt` — Netscape 形式 cookies.txt の例

### 運用 (`docs/ops/`)

- `INSTALL_TERMUX.md` — Termux インストール手順
- `TERMUX_BATTERY.md` — Galaxy/One UI のバッテリー設定
- `UPDATING_YTDLP.md` — yt-dlp 更新手順

## 3. 命名規約（ハイフン最大1つ、短く正確）

| 種類 | 命名規則 | 例 |
|---|---|---|
| タスクリスト | `docs/task-list.md`（固定・唯一の正本） | — |
| 計画書テンプレート | `_TEMPLATE.md`（固定） | — |
| 計画書 | `{TOPIC}_PLAN.md`（ハイフン最大1つ） | `YOUTUBE_PLAN.md` |
| 仕様書 | `kebab-case.md`（ハイフン最大1つ） | `tech-stack.md` |
| 調査 | `{TOPIC}_RESEARCH.md` | `TERMUX_RESEARCH.md` |
| 監査（差分） | `diff-{context}.md` | `diff-pipeline.md` |
| 監査（バグ） | `issues-{context}.md` | `issues-cancellation.md` |
| 完了レポート | `{TOPIC}_COMPLETE.md` | `TEMPLATE_APPLY_COMPLETE.md` |
| 設定例 | `kebab-case.example.*` / `*.examples.toml` | `profile-examples.toml` |
| 運用 | 大文字スネークケース | `INSTALL_TERMUX.md` |
| スキル | `kebab-case/SKILL.md` | `project-overview/SKILL.md` |
| ルール | `kebab-case.md`（ハイフン最大1つ） | `project-template.md` |
| フック手順 | `kebab-case.md`（ハイフン最大1つ） | `verify-commit.md` |
| フックスクリプト | `kebab-case.sh`（ハイフン最大1つ） | `restore-env.sh` |
| Rust モジュール | `snake_case.rs`（Rust 慣例） | `runner.rs` |

## 4. リンク規約

- `docs/` 内の相互リンクは相対パス（`./planning/`, `../README.md` 等）
- 存在しないファイルへのリンクを残さない。移動時は参照も更新。
- 外部リンクは可能な限り公式ソースを優先。
  - ffmpeg: https://ffmpeg.org/documentation.html
  - yt-dlp: https://github.com/yt-dlp/yt-dlp
  - ratatui: https://ratatui.rs / https://docs.rs/ratatui
  - tokio: https://tokio.rs / https://docs.rs/tokio
  - Termux: https://termux.com

## 5. 運用ルール

- ドキュメントを追加・削除・移動したら**必ず `docs/README.md` の目次を更新**
- 削除済みファイルを指す参照を残さない
- タスクIDと進捗は `docs/task-list.md`（正本）にのみ記録、計画書・完了レポートには「対応 task-list ID」を書いて相互参照
- 仕様は `docs/arch/` が正本、計画は `docs/planning/`、調査は `docs/research/`、監査は `docs/audit/`、運用は `docs/ops/`
- ファイル名は短く正確、ハイフン最大1つ

## 6. TOML / Rust コメント規約

- `config.default.toml` のコメントは日本語。ユーザーが直接編集することを前提に、各項目の役割と既定値の理由を書く。
- `[[profiles]]` の末尾に `generic`（フォールバック）が常に存在すること。`Config::normalize()`（`src/config.rs`）がこれを保証する。
- Rust の doc comment（`///`）は日本語でよい。公開 API（`pub`）には必ず doc comment を付与する。
