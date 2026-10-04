# Skills Index — Agentのスキル集

> このファイルは `.agent/skills/` の**入口**。タスク着手時に本ファイルだけ読み、
> 必要なスキルだけをピンポイントで読み込む（コンテキストの無駄遣いを防ぐ）。
>
> ここにあるのは **Agent自身のスキル** — 「このプロジェクトで何をどうやるとうまくいくか」
> という実践的なノウハウ・テクニック・手順・パターン・コードベース知識・実測知見。
> 設計仕様の正本ではない。設計の正本は `docs/`（arch/ + planning/ + research/ + audit/ + ops/ + examples/ + task-list.md）。
> 作業規約は `AGENTS.md` と `.agent/rules/`。
>
> 構成はClaude Code公式準拠: 各スキルは `<name>/SKILL.md`（YAML frontmatterに `name` / `description`）。
> 出典: TEMPLATE_REPO を asmr-dl（Rust TUI）向けに書き換え。`e2e/` → `device-testing/`、
> `deep-dive-setup/` → `site-onboarding/` に転用した。

## 読み方ガイド（どの状況でどのスキルを使うか）

| 状況 | 使うスキル |
| :--- | :--- |
| 初回 / 全体把握 | [`project-overview/SKILL.md`](./project-overview/SKILL.md) |
| Rust/Cargo/外部ツール（yt-dlp, ffmpeg）の使い方・ハマりどころ | [`tech-stack/SKILL.md`](./tech-stack/SKILL.md) |
| 「動かない / 検証できない / 外部に接続できない」環境トラブル | [`sandbox-constraints/SKILL.md`](./sandbox-constraints/SKILL.md) |
| 実機(Termux)でのTUI検証・テストマトリクス・実環境検証待ちタスクの切り分け | [`device-testing/SKILL.md`](./device-testing/SKILL.md) |
| 「サイト X が落ちない / 新サイトに対応したい」（プロファイル追加） | [`site-onboarding/SKILL.md`](./site-onboarding/SKILL.md) |
| ドキュメント変更後の整合性検証 | [`verify-doc-integrity/SKILL.md`](./verify-doc-integrity/SKILL.md) |
| ドキュメント整理・URL検証 | [`docs-maintenance/SKILL.md`](./docs-maintenance/SKILL.md) |
| 品質ゲート（fmt/clippy/test/build）・CI 未導入時の運用 | [`ci-quality-gates/SKILL.md`](./ci-quality-gates/SKILL.md) |
| Rust 単体テスト・意味あるアサーションの書き方 | [`testing/SKILL.md`](./testing/SKILL.md) |
| モジュール境界・依存方向の違反がないか確認 | [`import-boundaries/SKILL.md`](./import-boundaries/SKILL.md) |
| 純粋関数の決定論・same-inputテスト | [`determinism/SKILL.md`](./determinism/SKILL.md) |
| プロセス・チャネル・タスクのリソース解放（kill / drop / cap） | [`memory-leak/SKILL.md`](./memory-leak/SKILL.md) |
| ホットパス（tick / 行解析）のアロケーション削減 | [`zero-alloc/SKILL.md`](./zero-alloc/SKILL.md) |
| 3ファイル以上の変更・判断を伴う変更のレビューレポート作成 | [`diff-review-report/SKILL.md`](./diff-review-report/SKILL.md) |
| 設計の正本（全体アーキテクチャ・パイプライン・プロファイル） | `docs/arch/*` + `docs/README.md` |
| 進捗の正本 | `docs/task-list.md` |
| 計画書 | `docs/planning/*_PLAN.md` |
| 調査 | `docs/research/*` |
| 監査 | `docs/audit/*` |
| 設定例 | `docs/examples/` |

## スキル一覧

| スキル | できるようになること（Agentの能力） | 最終更新 | 出典 |
| :--- | :--- | :--- | :--- |
| [project-overview/SKILL.md](./project-overview/SKILL.md) | asmr-dl の製品概要・モジュール構成・ドキュメント構成・進捗管理を素早く把握する | 2026-10-04 | TEMPLATE_REPO → asmr-dl 書き換え |
| [tech-stack/SKILL.md](./tech-stack/SKILL.md) | Rust/Cargo/ratatui/tokio + yt-dlp/ffmpeg の正しい使い方と実測ハマり（--js-runtimes, libopus, process group kill）を回避できる | 2026-10-04 | 同上 |
| [sandbox-constraints/SKILL.md](./sandbox-constraints/SKILL.md) | Sandbox の egress ブロック（Rust 系到達不可・apt 不可）と termux-* 不在を迂回して検証・復旧できる | 2026-10-04 | 同上 + 2026-10-04 実測 |
| [device-testing/SKILL.md](./device-testing/SKILL.md) | TUI の実機検証マトリクスを作成し、「実環境検証待ち」をtask-listで管理できる（Playwright 前提の e2e を転用） | 2026-10-04 | TEMPLATE_REPO `e2e/` → 転用 |
| [site-onboarding/SKILL.md](./site-onboarding/SKILL.md) | 「サイト X が落ちない」要求を 調査→ヘッダ/cookie特定→[[profiles]]追加→docs更新→検証依頼 まで一気通貫で実行できる | 2026-10-04 | TEMPLATE_REPO `deep-dive-setup/` → 転用 |
| [verify-doc-integrity/SKILL.md](./verify-doc-integrity/SKILL.md) | ドキュメントの整合性（リンク実在・目次更新・機密情報混入なし）を機械検証できる | 2026-10-04 | 同上 |
| [diff-review-report/SKILL.md](./diff-review-report/SKILL.md) | 仕様書変更の差分を人間がレビュー可能なレポートにまとめることができる | 2026-10-04 | 同上 |
| [docs-maintenance/SKILL.md](./docs-maintenance/SKILL.md) | ドキュメント整理・内部リンク整合性・外部URL有効性・ミラー排除を確実にできる | 2026-10-04 | 同上 |
| [ci-quality-gates/SKILL.md](./ci-quality-gates/SKILL.md) | 品質ゲート（fmt/clippy/test/build）を正本として扱い、CI 未導入時に代替検証で運用できる | 2026-10-04 | 同上 |
| [testing/SKILL.md](./testing/SKILL.md) | Rust 単体テストで意味あるアサーションを書き、純粋関数/I-O関数を分離してテスト容易性を確保できる | 2026-10-04 | 同上（Rust 向け全面書き換え） |
| [import-boundaries/SKILL.md](./import-boundaries/SKILL.md) | モジュール間の依存方向（config/job → runner → app → ui → main）を守り、逆参照と責務混在を防ぐ | 2026-10-04 | 同上（Rust 向け全面書き換え） |
| [determinism/SKILL.md](./determinism/SKILL.md) | 純粋関数の決定論を守り、禁止API（時刻/乱数/I-O）検出・same-inputテストを実装できる | 2026-10-04 | 同上（Rust 向け全面書き換え） |
| [memory-leak/SKILL.md](./memory-leak/SKILL.md) | 子プロセス・watch/mpsc チャネル・tokio タスク・ログリングバッファの解放（kill_on_drop / cap / drop）を守れる | 2026-10-04 | 同上（Rust 向け全面書き換え） |
| [zero-alloc/SKILL.md](./zero-alloc/SKILL.md) | 200ms tick と行解析のホットパスで不要アロケーション（逐次 String 生成 / 配列確保）を削減できる | 2026-10-04 | 同上（Rust 向け全面書き換え） |

## 設計仕様の正本（スキルではなくdocs/）

| 文書 | 内容 |
| :--- | :--- |
| [docs/README.md](../../docs/README.md) | 全ドキュメントの目次 |
| [docs/task-list.md](../../docs/task-list.md) | タスク管理の唯一の正本 |
| [docs/arch/README.md](../../docs/arch/README.md) | 仕様書一覧（どう作るか） |
| [docs/planning/README.md](../../docs/planning/README.md) | 計画書索引 |
| [docs/planning/_TEMPLATE.md](../../docs/planning/_TEMPLATE.md) | 計画書テンプレート |
| [AGENTS.md](../../AGENTS.md) | 開発規約（汎用§1〜§5 + プロジェクト固有§6） |
| [.agent/rules/](../../.agent/rules/) | トピック別ルール（pathsで発火条件） |

> 実装テクニック・実測ハマりどころは **skills** に貯め、設計の事実は **docs/** を正本とする。

## 運用ルール

- 新しいノウハウ（特に**実測で判明した**制約・挙動）を得たらスキルとして追加/更新し、本indexの「最終更新」も更新する。
- 新スキル追加時は「読み方ガイド」と「一覧」の両方に追記する。
- スキルは実践的なやり方・コードパターン・回避策を書く。設計の正本はdocs/。
- AGENTS.mdと重複する作業規約はスキルに書かずAGENTS.md / rules/を正とする。
- 各スキルは `<kebab-case>/SKILL.md`（YAML frontmatterに `name` / `description` 必須）。`description` は自動発火の判断材料なので具体的に書く。

## 公式構成との対応

- 公式 `.claude/skills/<name>/SKILL.md` と同型。`name` はディレクトリ名と一致させる。
- `commands/*.md` は旧形式だが互換性のため残す。新規は `skills/` を使う。
- スキルは `description` の具体性が重要。Claudeが自動で発火するかどうかをこの説明で判断する。
