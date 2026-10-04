# Hook: Verify Before Commit（commit 直前検証）

> **トリガー**: 実装が終わり、Git Commit する直前。
> **目的**: プロジェクトで定義された検証を必ず全 pass させてから commit する。
> 途中の検証失敗で次へ進んではならない。

## 検証（順に実行、1 つでも失敗したら原因特定→修正→再全検証）

cargo が使える環境（開発機 / 将来のCI）:

```bash
cargo fmt --all -- --check              # フォーマット確認
cargo clippy --all-targets -- -D warnings   # Lint（警告=エラー）
cargo test                              # 単体テスト
cargo build --release                   # 本番ビルド
```

※ 検証コマンドは `AGENTS.md §3.1` / `ci-quality-gates` スキルを正本とする。

### 各コマンドの注意

- **fmt**: 差分があれば `cargo fmt --all` で自動修正し、再実行。
- **clippy**: `--all-targets` でテストコードも対象。`-D warnings` で警告ゼロ。
- **test**: 一発実行（watch 相当なし）。純粋関数のテスト（`#[cfg(test)]`）。
- **build**: `--release` は `opt-level=3, lto=thin, codegen-units=4, strip=true`。

## Sandbox（cargo 無し）での代替検証（AGENTS.md §6.2）

cargo が実行できない環境では、以下を実施し、コミット報告に「build/test 未実行（§6.2）」を明記する:

```bash
python3 -c "import tomllib; tomllib.load(open('config.default.toml','rb')); print('toml OK')"
python3 - <<'PY'
import tomllib
cfg = tomllib.load(open('config.default.toml','rb'))
profiles = cfg.get('profiles', [])
assert profiles and profiles[-1].get('name') == 'generic', "generic must be last"
print(f"profiles OK: {len(profiles)} (last=generic)")
PY
python3 -c "import tomllib; d=tomllib.load(open('Cargo.toml','rb')); assert d['package']['name']=='asmr-dl'; print('Cargo OK')"
sh -n scripts/install-termux.sh && sh -n scripts/termux-url-opener
jq empty .agent/settings.json
sh .agent/hooks/post_edit_verify.sh
```

- Rust ソースの編集は構文チェックできないため、**最小差分**にとどめ、`cargo build/test` を CI / 実機への残課題として明記する。

## 追加確認（commit 前）

```bash
git status
git diff                       # 意図しないファイル/差分が無いか
git diff --cached --name-only | grep -E "(\.env|cookies|target/)"   # 機密・成果物の混入
```

- リポジトリ固有の不変パス（`asmr-dl.zip` / `target/`）に差分が無いことを確認。
- `config.default.toml` を変えたら `docs/arch/profiles.md` / `docs/examples/` / `README.md` の同期を確認。

## 検証失敗時の原則（AGENTS.md §3.2）

- テストを通すためだけの**不正な修正厳禁**（テスト削除/`#[ignore]`・アサーション緩和・安易な `unwrap`/`unsafe`・Lint 無効化・エラー握り潰し）。
- 既存テストが落ちたら「テストが間違っている」と即断せず、**既存仕様を壊していないか**先に確認。

## 完了後

検証 all pass + 意図しない差分なし を確認 → commit（Conventional Commits 形式 + 日本語）→ `git push origin <session-branch>`。
