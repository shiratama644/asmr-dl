---
paths:
  - "Cargo.toml"
  - "Cargo.lock"
  - "src/**"
  - "config.default.toml"
  - "scripts/**"
  - ".agent/hooks/**"
---

# Rule 04: 検証・品質保証ルール

> 優先度: **HIGH** — 「動いた」で終わらせず、機械的に品質を担保する

## 1. 検証コマンドの実行（必須）

本プロジェクトの検証コマンドは以下。commit前に原則として全てPASSさせる。
**Sandbox では Rust ツールチェーンが無いため実行できないことがある（AGENTS.md §6.2）**。
その場合は §2 の代替検証を実施し、コミット報告に「build/test 未実行（§6.2）」を明記する。

```bash
cargo fmt --all -- --check              # フォーマット確認 (rustfmt)
cargo clippy --all-targets -- -D warnings   # Lint（警告をエラー扱い）
cargo test                              # 単体テスト（#[cfg(test)]、watchモードなし）
cargo build --release                   # 本番ビルド
```

### 各コマンドの注意

- **fmt**: `rustfmt.toml` が無い場合は rustfmt 既定設定。フォーマット差分は `cargo fmt --all` で自動修正可能。
- **clippy**: `--all-targets` でテスト・ベンチも対象にする。`-D warnings` で警告ゼロを強制。
- **test**: `cargo test` は一発実行（watch 相当なし）。TUI の描画は headless で動かないが、`#[cfg(test)]` の純粋関数テストは実行可能。
- **build**: `--release` は `opt-level=3, lto=thin, codegen-units=4, strip=true`。環境起因の既知エラーが exit 0 で出ても成功扱い。

## 2. Sandbox 代替検証（§6.2 で cargo 実行できない時）

```bash
# TOML 構文（config.default.toml は include_str! でバイナリに埋め込まれる）
python3 -c "import tomllib; tomllib.load(open('config.default.toml','rb')); print('toml OK')"

# プロファイルが generic を末尾に持つこと（normalize の前提）
python3 - <<'PY'
import tomllib
cfg = tomllib.load(open('config.default.toml','rb'))
profiles = cfg.get('profiles', [])
assert profiles and profiles[-1].get('name') == 'generic', "generic must be last"
print(f"profiles OK: {len(profiles)} (last=generic)")
PY

# shell スクリプト構文（POSIX sh）
sh -n scripts/install-termux.sh
sh -n scripts/termux-url-opener

# Markdown 内部リンクの実在チェック
sh .agent/hooks/post_edit_verify.sh
```

## 3. エラー対応と品質維持

- エラー発生時はエラーメッセージやスタックトレースから根本原因を特定し、最小限の範囲で修正する。
- **テストを通すためだけの不正な修正は厳禁**：
  - テストの削除・`#[ignore]`・アサーションの緩和
  - 型エラーを回避するための安易な `unwrap()` / `expect()` / `unsafe`
  - Lint ルールの勝手な無効化（`#[allow]`）・エラーの握りつぶし（`let _ =` で失敗を捨てる）
- **既存仕様の尊重**：既存テストが落ちた場合、「テストが間違っている」と即断せず、既存仕様を壊していないか確認する。

## 4. 既存バグの扱い

- **今回のタスクを妨げるバグ**：必要最小限の修正を行う。
- **無関係な既存バグ**：勝手に修正せず、ユーザーに報告する。
- バグ修正時は、可能であれば再発防止の回帰テスト（Regression Test）を追加する。

## 5. 完了前の機械検証チェックリスト

### (A) 検証コマンド全PASS（実行可能な範囲）

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo build --release
```

### (B) 意図しない差分の確認

```bash
git status --short
git diff --stat
# 狙いではないファイルが含まれていないかを目視
```

### (C) 機密情報の混入チェック

```bash
git diff --cached --name-only | grep -E "(\.env|cookies)"
# → 出力が0件であること（cookies.txt はユーザー個人ファイル。リポジトリに含めない）
```

### (D) ドキュメント整合性

- `docs/` に追加・削除があれば `docs/README.md` の目次が更新されているか
- `config.default.toml` を変えたら `docs/examples/` と `docs/arch/profiles.md` が同期しているか
- タスクリストの証拠（コミットSHA / テスト件数 / 実測値）が記録されているか

## 6. テスト配置規則（Rust 慣例）

- テストはテスト対象の**同一 `.rs` 文件内に `#[cfg(test)] mod tests`** で配置する。
  - 例: `src/config.rs` の末尾に `mod tests`、`src/runner.rs` の末尾に `mod tests`
- 本プロジェクトには `_tests_/` ディレクトリは存在しない（Rust の慣例に従う）。
- 純粋関数（I/O なし）をテスト対象にする。I/O 関数はモック・注入を最小限に。
- テスト名は英語の `snake_case`（Rust 慣例）。何をテストしているか明確に（例: `default_config_parses` / `urls_from_share_text`）。

## 7. 報告のテンプレート

作業完了時は以下で簡潔に報告：

1. **変更ファイル一覧**（パスのみ）
2. **検証結果**（fmt/clippy/test/build の PASS/FAIL + 件数。Sandbox では §2 の代替検証結果）
3. **意図的に変えた点**（本当に変えたつもりの場所だけ）
4. **残課題・人間判断を求める事項**（あれば）
