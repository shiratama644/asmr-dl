# Quality — 品質ゲート・テスト方針

> 正本: `Cargo.toml` / `src/*.rs` の `#[cfg(test)]` / `.agent/skills/ci-quality-gates/` / `AGENTS.md §3`。

## 品質ゲート（commit 前）

| # | コマンド | 目的 | 失敗時 |
|---:|---|---|---|
| G1 | `cargo fmt --all -- --check` | フォーマット（rustfmt 既定） | `cargo fmt --all` で自動修正 |
| G2 | `cargo clippy --all-targets -- -D warnings` | Lint（警告=エラー） | 修正。`#[allow]` は理由コメント必須 |
| G3 | `cargo test` | 単体テスト（純粋関数） | 修正。shallow test 化しない |
| G4 | `cargo build --release` | 本番ビルド | 修正 |

- **Sandbox（cargo 無し、§6.2）では代替検証**（tomllib / `sh -n` / jq / リンクチェック）のみ実施し、コミット報告に「G1〜G4 未実行（§6.2）」を明記。
- CI 未導入のため、G1〜G4 の最終検証は**開発機 or 実機**。将来の CI 構成は [cicd.md](./cicd.md)。

## テスト方針

- **配置**: 同一 `.rs` 文件内に `#[cfg(test)] mod tests`（`_tests_/` ディレクトリは使わない、Rust 慣例）。
- **対象**: 純粋関数（I/O なし）。I/O 関数はロジックを純粋関数に分離してテスト（`.agent/skills/testing/`）。
- **既存テスト**（2026-10-04 時点）:
  | ファイル | テスト | 内容 |
  |---|---|---|
  | `src/config.rs` | `default_config_parses` | 既定設定が parse でき、`generic` が末尾 |
  | `src/config.rs` | `detection` | 各ドメインが正しいプロファイルに判定 |
  | `src/config.rs` | `urls_from_share_text` | 共有テキストから URL を抽出（末尾句読点除去） |
  | `src/runner.rs` | `names` | `name_from_url`（percent decode）/ `is_generic_name` / `sanitize_filename` |
  | `src/runner.rs` | `hms` | `fmt_hms`（0 / 65 / 3725 秒） |
- **必ずテストするもの**: 境界値（0/1/空/最大長）・異常系（不正 URL / 無音）・same-input（決定論）。
- **TUI の描画・キー操作は単体テストの対象外**（headless で動かない）。実機検証（`.agent/skills/device-testing/` の D1〜D23）。

## 壊れたファイル対策（プロダクト品質）

- 変換後検証: 出力を再度 ffprobe し、`has_audio` / **長さ（入力の 90% 以上、-1秒許容）** を確認。
  途中で切れた Opus は失敗扱い（ADR-008）。
- 出力名の衝突は `unique_path` で回避（`名 (2).opus`、保存直前判定で並列ジョブ対策）。

## 報告の証拠

コミット報告には以下を含める（`AGENTS.md §7.2`）:

- 検証コマンドと結果（G1〜G4 の PASS/FAIL + テスト件数）
- Sandbox では代替検証結果 + 未実行ゲートの明示
- 実機検証待ちのタスクは `task-list.md` に `実環境検証待ち` で登録

## 関連

- `.agent/skills/ci-quality-gates/SKILL.md` — ゲートの正本運用
- `.agent/skills/testing/SKILL.md` — テストの書き方
- [cicd.md](./cicd.md) — 将来の CI
- [pipeline.md](./pipeline.md) — 変換後検証の詳細
