---
name: sandbox-constraints
description: Arena Sandbox の恒常的制約（Rust ツールチェーン不可・egress 制限・termux-* 不在・TUI headless 不可）と迂回策。2026-10-04 実測。環境トラブル時に参照。Use when encountering network, build, or permission issues in sandbox.
---

# Sandbox Constraints — 環境制約と迂回策

> このスキルは Arena Sandbox 環境の恒常的制約と、その迂回策をまとめたもの。
> **2026-10-04 に asmr-dl で実測した値**を含む。
> 制約は「乗り越える」のではなく「迂回する」。修正対象ではない。

## 1. 恒常的制約一覧（2026-10-04 実測）

| 制約 | 実測内容 | 影響 | 対処 |
|---|---|---|---|
| **Rust 系ドメイン到達不可** | `curl https://sh.rustup.rs` / `static.rust-lang.org` / `crates.io` 全て `SSL_ERROR_SYSCALL` | rustup によるツールチェーン導入不可 → `cargo build` / `cargo test` / `cargo clippy` / `cargo fmt` 実行不可 | 構文レベルの代替検証（§2）+ 最終検証は CI / 実機へ分離（`device-testing` スキル）。コミット報告に「build/test 未実行（§6.2）」を明記 |
| **apt 不可** | `apt-get update` → `Permission denied`（root なし） | パッケージ管理で rustc 入手不可 | 同上 |
| **`termux-*` ビナリ不在** | Sandbox は Linux x86_64。`termux-wake-lock` 等は未インストール | Termux 連携（通知 / メディアスキャン / wake-lock）の動作確認不可 | 実機検証タスクとして分離。コードは「bin 無しの無音スキップ」方針（`spawn_detached`）を維持 |
| **TUI は headless 実行不可** | 端末（Tty）が無い。`ratatui::init()` は raw mode を要求 | 起動確認・キー操作確認不可（`--help` / `--print-config` もビルド後にのみ確認可能） | `device-testing` スキルのテストマトリクスで実機検証依頼を作成 |
| **GitHub は到達可** | `gh api` / `git clone` / push が正常 | — | fetch/push/PR は通常通り実行可 |
| **python3 / jq は使える** | `python3`（tomllib 付き）+ `jq` が `/usr/bin` に存在 | — | 代替検証の土台として利用（§2） |

## 2. 代替検証パターン（cargo が無い環境）

```bash
# (1) TOML 構文 + プロファイル末尾 generic 検証
python3 -c "import tomllib; tomllib.load(open('config.default.toml','rb')); print('toml OK')"
python3 - <<'PY'
import tomllib
cfg = tomllib.load(open('config.default.toml','rb'))
profiles = cfg.get('profiles', [])
assert profiles and profiles[-1].get('name') == 'generic', "generic must be last"
print(f"profiles OK: {len(profiles)} (last=generic)")
PY

# (2) shell スクリプト構文（POSIX sh）
sh -n scripts/install-termux.sh
sh -n scripts/termux-url-opener

# (3) JSON 構文（settings.json 等）
jq empty .agent/settings.json

# (4) Markdown 内部リンクの実在チェック（docs/ 配下）
cd docs
grep -ohr "](\./[^)]*)" *.md **/*.md 2>/dev/null | sed 's/](\.\///; s/)$//; s/#.*//' | sort -u | while read -r f; do [ -f "$f" ] || echo "BROKEN: $f"; done
cd ..

# (5) hooks の事後検証
sh .agent/hooks/post_edit_verify.sh
```

- Rust ソースの編集後は **構文チェックができない**ことを正直に報告する（clippy/parse が走らない）。
  編集は「既存コードの最小差分」にとどめ、実機 or CI での `cargo build` を残課題として明記する。

## 3. 復旧手順（Sandbox再構築時）

Sandbox再構築を検知したら（`git log` が起点1件のみ / 大量削除+未追跡 / `target/` 無）：

```bash
# 1. リモートの最新をfetch（ブランチ名は git branch --show-current で確認）
git fetch origin <現在のブランチ>

# 2. FETCH_HEADにワークツリーごとリセット（この場合のみ --hard 許可）
git reset --hard FETCH_HEAD

# 3. 依存を再構築（cargo があれば fetch。無ければ記録のみで進行）
bash .agent/hooks/restore-env.sh
```

復旧後は必ず `git log --oneline -5` と、実行可能な検証（§2）で健全性を確認してから作業再開。

## 4. 実機検証依頼の作成（device-testing と併用）

cargo / TUI を実行できないため、実機で確認すべき事項を **再現可能な手順 + 期待結果** で書けるようにする：

- どのビルド（コミット SHA）で、どの手順で、何を観察するか
- 成功時・失敗時のそれぞれで何が起きるか（例: 「Header に `yt-dlp✓ ffmpeg✓ opus✓` が出る」）
- 結果を `docs/task-list.md` の「実環境検証待ち」タスクの証拠欄に回填してもらう運用

## 5. 参考：restore-env.sh の仕組み（asmr-dl 版）

- `command -v cargo` でツールチェーンの有無を検出
- 有: `cargo fetch`（lockfile 準拠の依存取得）+ `cargo test` で健全性確認
- 無: 「ツールチェーン無し（§6.2）」を記録して代替検証へ誘導

## 6. 更新履歴

| 日付 | 内容 |
|---|---|
| 2026-10-04 | asmr-dl 適用時に実測（Rust 系 egress ブロック / apt 不可 / python3+jq 可 / GitHub 可） |
