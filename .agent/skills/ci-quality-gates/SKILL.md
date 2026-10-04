---
name: ci-quality-gates
description: 品質ゲート（cargo fmt / clippy / test / build）を唯一の正本として扱い、CI 未導入（2026-10-04 時点）の現状でローカルゲート + 代替検証 + 実機検証の3段で運用するスキル。将来の GitHub Actions 構成は docs/arch/cicd.md。
---

# CI Quality Gates — 品質ゲートを正本として扱うスキル

> 出典: TEMPLATE_REPO の `ci-quality-gates/SKILL.md`（pnpm/Playwright CI 前提）を asmr-dl 向けに書き換え
> 正本: `docs/arch/cicd.md`（将来構成）・`docs/arch/quality.md`・`AGENTS.md §6`

## 現状（2026-10-04）

- **GitHub Actions は未導入**（`.github/workflows/` なし）。
- 品質の担保は以下の3段:
  1. **ローカル品質ゲート**（cargo がある環境: 開発機 / 将来のCI）
  2. **代替検証**（Sandbox、cargo 無し — `sandbox-constraints` スキル）
  3. **実機検証**（Termux — `device-testing` スキル）

## ローカル品質ゲート（正本）

```bash
cargo fmt --all -- --check               # (G1) フォーマット
cargo clippy --all-targets -- -D warnings  # (G2) Lint（警告=エラー）
cargo test                               # (G3) 単体テスト
cargo build --release                    # (G4) 本番ビルド
```

| ゲート | 目的 | 失敗時の扱い |
|---|---|---|
| G1 fmt | コードの一貫性 | `cargo fmt --all` で自動修正。修正後は再実行 |
| G2 clippy | 潜在バグ・コード品質 | `-D warnings` で警告をエラー扱い。`#[allow]` は理由コメント必須 |
| G3 test | ロジックの回帰検出 | 意味あるアサーションの確認（`testing` スキル）。shallow test 化しない |
| G4 build | 本番ビルドの成功保証 | `opt-level=3, lto=thin, strip=true`。S24 Ultra で数分 |

- **commit 前に G1〜G4 を全PASSさせる**（cargo が使える環境では必須、AGENTS.md §3.1）。
- Sandbox（cargo 無し）では代替検証（tomllib / `sh -n` / jq / リンク）を実施し、コミット報告に「G1〜G4 未実行（§6.2）」を明記する。

## Sandbox での代替検証（cargo 無し）

```bash
python3 -c "import tomllib; tomllib.load(open('config.default.toml','rb')); print('toml OK')"
python3 -c "import tomllib; d=tomllib.load(open('Cargo.toml','rb')); assert d['package']['name']=='asmr-dl'; print('Cargo OK')"
sh -n scripts/install-termux.sh && sh -n scripts/termux-url-opener
jq empty .agent/settings.json
sh .agent/hooks/post_edit_verify.sh
```

- Rust ソースの編集は**構文チェックできない**ため最小差分にとどめ、G1〜G4 を残課題として明記する。

## 将来の CI 構成（docs/arch/cicd.md 準拠）

CI を導入する場合は以下を正本とする（導入時に `docs/arch/cicd.md` と `.github/workflows/ci.yml` を**同時に**作成する）:

```yaml
name: ci
on:
  push: { branches: [main, 'arena/**'] }
  pull_request: { branches: [main] }

jobs:
  quality:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable   # stable + clippy + rustfmt
        with: { components: clippy, rustfmt }
      - uses: Swatinem/rust-cache@v2
      - run: cargo fmt --all -- --check
      - run: cargo clippy --all-targets -- -D warnings
      - run: cargo test
      - run: cargo build --release
```

- **Agent は `.github/workflows/` を直接作成しない**（提案 → 承認フローが基本）。CI 導入はユーザーの明示的指示がある場合のみ。
- workflow の解説は `docs/arch/cicd.md` に書き、workflow 自体の複製を docs に置かない（正本は1つ）。

## 運用ルール

- ゲート定義（コマンド・フラグ）を変える時は `AGENTS.md §3.1` / `docs/arch/quality.md` / `docs/arch/cicd.md` / 本スキル を**同時に**更新する（正本の散逸防止）。
- `cargo test` のテスト件数はコミット報告の証拠に含める（例: `12 passed`）。
- 新規クレートを導入したら G4（`cargo build --release`）の所要時間を再計測し、`docs/arch/milestones.md` に記録する。

## 関連

- `docs/arch/cicd.md` — 将来の CI 構成
- `docs/arch/quality.md` — 品質ゲートの設計
- `sandbox-constraints/SKILL.md` — Sandbox 制約
- `device-testing/SKILL.md` — 実機検証
