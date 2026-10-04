# CI/CD — 現状・将来構成

> 正本: `.agent/skills/ci-quality-gates/SKILL.md` / `AGENTS.md §3`。
> **2026-10-04 時点: GitHub Actions は未導入**（`.github/workflows/` なし）。

## 現状

- 品質の担保は以下の3段（`ci-quality-gates` スキル参照）:
  1. **ローカル品質ゲート**（cargo がある環境: 開発機）— G1〜G4
  2. **代替検証**（Sandbox、cargo 無し）— tomllib / `sh -n` / jq / リンクチェック
  3. **実機検証**（Termux）— `device-testing` スキルの D1〜D23
- リリースは**未整備**。`target/release/asmr-dl` を `scripts/install-termux.sh` でインストール、または手動配布（`asmr-dl.zip` 相当）。

## 将来の CI 構成（導入時の正本）

CI を導入する場合は、`.github/workflows/ci.yml` を**本ファイルと同時**に作成する（正本は1つ）。

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
      - uses: dtolnay/rust-toolchain@stable
        with: { components: clippy, rustfmt }
      - uses: Swatinem/rust-cache@v2
      - run: cargo fmt --all -- --check
      - run: cargo clippy --all-targets -- -D warnings
      - run: cargo test
      - run: cargo build --release
```

- **Agent は `.github/workflows/` を直接作成しない**（提案 → 承認フローが基本）。CI 導入はユーザーの明示的指示がある場合のみ（`ci-quality-gates` スキル）。
- workflow の解説は本ファイルに書き、workflow 自体の複製を docs に置かない。
- Linux runners は TUI を起動できないが、G1〜G4（fmt/clippy/test/build）はすべて実行可能（`cargo test` は純粋関数のみで TUI を起動しない）。

## 今後の候補（優先度）

| 候補 | 内容 | 優先度 |
|---|---|---|
| **CI 導入**（上） | push/PR で G1〜G4 を全PASS | 🟡 |
| **実機検証の制度化** | `device-testing` D1〜D23 を `task-list.md` の常設タスク（DEV-1）として運用 | 🟡 |
| **リリース自動化** | GitHub Releases に `asmr-dl`（aarch64-linux-android / x86_64-linux）をアップロード | 🟢 |
| **依存更新**（Renovate 等） | Cargo 依存の自動更新 PR | 🟢 |

## 関連

- `.agent/skills/ci-quality-gates/SKILL.md` — ゲートの正本運用
- [quality.md](./quality.md) — 品質ゲートの設計
- `ops/UPDATING_YTDLP.md` — yt-dlp の手動更新（CI 非依存）
