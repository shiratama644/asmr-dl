#!/usr/bin/env bash
# restore-env.sh
# Sandbox 再構築後の環境復旧（AGENTS.md §4.1.1）。sandbox-recovery.md から呼出。
#
# やること:
#   1. cargo（Rust ツールチェーン）の存在を検出
#   2. 有: cargo fetch（lockfile 準拠の依存取得）+ cargo test で健全性確認
#   3. 無: AGENTS.md §6.2 の Sandbox 制約（Rust 系 egress ブロック・apt 不可）を記録し、
#          代替検証（tomllib / sh -n / jq / リンクチェック）へ誘導
#
# 注意: 本 Sandbox では sh.rustup.rs / static.rust-lang.org / crates.io が到達不可
#       （2026-10-04 実測, SSL_ERROR_SYSCALL）ため、ツールチェーンの「導入」はできない。
#       復旧は「git 状態の復旧」までを担保し、ビルド検証は CI / 実機へ分離する。

set -euo pipefail
cd "$(dirname "$0")/../.." || exit 1

echo "[restore-env] checking Rust toolchain ..."
if command -v cargo >/dev/null 2>&1; then
  echo "[restore-env] cargo found: $(cargo --version)"
  echo "[restore-env] fetching dependencies (cargo fetch) ..."
  cargo fetch
  echo "[restore-env] running tests (cargo test) ..."
  if cargo test --quiet; then
    echo "[restore-env] done. tests passed."
  else
    echo "[restore-env] WARN: cargo test failed. inspect before continuing." >&2
    exit 1
  fi
else
  echo "[restore-env] cargo NOT found (AGENTS.md §6.2: no Rust toolchain in this sandbox)."
  echo "[restore-env] cannot install (sh.rustup.rs / static.rust-lang.org / crates.io unreachable, apt disabled)."
  echo "[restore-env] falling back to syntax-level verification ..."
  python3 -c "import tomllib; tomllib.load(open('config.default.toml','rb')); print('[restore-env] config.default.toml: toml OK')"
  python3 -c "import tomllib; d=tomllib.load(open('Cargo.toml','rb')); assert d['package']['name']=='asmr-dl'; print('[restore-env] Cargo.toml: OK')"
  sh -n scripts/install-termux.sh && sh -n scripts/termux-url-opener && echo "[restore-env] scripts: sh -n OK"
  echo "[restore-env] NOTE: cargo build/test/clippy must be run on dev machine or CI (docs/arch/cicd.md)."
  echo "[restore-env] done (partial: syntax-level only)."
fi
