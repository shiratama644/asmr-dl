---
description: テストを実行し、結果を確認する。Use when you want to run tests and check results.
---

# Test Command

テストを実行し、結果を確認する手順。

## 手順

1. **単体テスト**（cargo が使える環境）:
   ```bash
   cargo test            # 全テスト実行（#[cfg(test)]、一発実行）
   cargo test <name>     # 特定テスト
   ```
   - 既存テスト: `src/config.rs`（`default_config_parses` / `detection` / `urls_from_share_text`）、`src/runner.rs`（`names` / `hms`）
   - 純粋関数のテスト（I/O なし）。TUI の描画・キー操作は対象外（`device-testing` スキルの L3）

2. **Lint**（テストコードも含む）:
   ```bash
   cargo clippy --all-targets -- -D warnings
   ```
   - 意味のあるテストか確認（shallow test / `assert!(true)` を避ける）

3. **Sandbox（cargo 無し、§6.2）**:
   ```bash
   python3 -c "import tomllib; tomllib.load(open('config.default.toml','rb')); print('OK')"
   sh -n scripts/install-termux.sh
   sh .agent/hooks/post_edit_verify.sh
   ```
   - `cargo test` が実行できない。代替検証のみ実施し、テスト実行を CI / 実機への**残課題**として明記する
   - 新規に追加した `#[cfg(test)]` テストは「未実行（§6.2）」として報告する

4. **結果の確認**:
   - 失敗したテストがあれば原因を特定して修正
   - フレイキーテストがあれば原因を特定して修正（TUI / 時刻依存テストは特に注意）
   - 意味あるテストが不足している場合は `test-writer` エージェントにテスト追加を依頼

## 品質基準

- 既存テストを壊さない
- テストを通すためだけに `unwrap()` を足したり、アサーションを緩めたりしない
- 境界値・エッジケース・異常系を必ずテスト
- 純粋関数は same-input（同じ入力→同じ出力）を検証する（`determinism` スキル）
