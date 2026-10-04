---
name: testing
description: Rust 単体テスト（#[cfg(test)]）で意味あるアサーションを書くスキル。純粋関数の分離、境界値・異常系、same-input、既存テストの拡張、shallow test 回避。Use when writing tests, improving coverage, or adding a new pure function.
---

# Testing — Rust 単体テストで意味ある検証をするスキル

> 出典: TEMPLATE_REPO の `testing/SKILL.md`（Vitest/Playwright 前提）を **Rust 向けに全面書き換え**
> 正本: `docs/arch/quality.md`、`src/config.rs` / `src/runner.rs` の `#[cfg(test)]`、`AGENTS.md §3`

## 原則

- 数字稼ぎの shallow test（呼ぶだけ / 存在確認だけ）ではなく、**壊れるとプロダクトが壊れる経路**を優先
- assertion 弱体化をしない（`assert!(true)` / 空の `#[test]` は禁止）
- テストを通すためだけに `unwrap()` を実装へ移す・アサーションを緩めない
- `cargo test` は一発実行（watch 相当なし）。commit 前検証に `cargo test --watch` を使わない

## テスト配置（Rust 慣例）

- テストは**同一 `.rs` 文件内に `#[cfg(test)] mod tests`** で配置する。
  - 既存: `src/config.rs`（`default_config_parses` / `detection` / `urls_from_share_text`）、`src/runner.rs`（`names` / `hms`）
- 本プロジェクトには `_tests_/` ディレクトリは存在しない（Node 系テンプレートの慣例をそのまま持たない）。
- テスト名は英語 `snake_case`。何をテストしているか明確に（例: `urls_from_share_text`）。

## 純粋関数の分離（テスト容易性の鍵）

I/O を伴う関数は直接テストしにくい。ロジックを**純粋関数（I/O なし）に切り出して**テストする。

| 関数 | 種別 | テスト可能 |
|---|---|---|
| `config::extract_urls` | 純粋（regex + url パース） | ✅ 文字列入 → URL 列 |
| `config::Config::detect_profile` | ほぼ純粋（設定+URL → idx） | ✅ 設定を固定して URL 判定 |
| `config::Profile::matches` | 純粋（url_regex / domains） | ✅ |
| `runner::sanitize_filename` | 純粋 | ✅ 全角・制御文字・長さ |
| `runner::is_generic_name` | 純粋 | ✅ `index` / `chunklist_` / 日本語タイトル |
| `runner::name_from_url` | ほぼ純粋 | ✅ percent decode / 汎用名 fallback |
| `runner::fmt_hms` | 純粋 | ✅ 0 / 65 / 3725 秒 |
| `runner::clean_na` | 純粋 | ✅ `NA` / `N/A` / 空 |
| `runner::percent_decode` | 純粋 | ✅ `%E5%A3%B0` → 声 |
| `runner::probe` / `convert` / `run_proc` | I/O（外部プロセス） | ❌ 直接は不可（モックは最小限） |

**新しいロジックを書く時は、I/O とロジックを分離して純粋関数に寄せる**（例: yt-dlp の progress 行パースは、行文字列 → 構造体 にする純粋関数に切り出せばテスト可能になる）。

## 意味あるテストの例（既存コードから）

- `urls_from_share_text`: 共有テキスト（`この動画おすすめ→https://youtu.be/abc123?si=xx。あと https://example.com/a.m3u8, 以上`）から URL を**正確に2件**抽出し、末尾の句読点・カンマが除去されることを検証
- `detection`: 各ドメイン（youtube / niconico / hls-direct / file-direct / x / generic）が**正しいプロファイル**に判定されることを検証
- `default_config_parses`: 既定設定が parse でき、**`generic` が末尾**であることを検証（`normalize()` の前提を守る回帰テスト）
- `names`: percent decode（`%E5%A3%B0` → `声`）、汎用名（`index [index]`）の判定、`sanitize_filename`（`a/b:c?` → `a_b_c_`）

## 境界値・異常系を必ずテスト

- **境界値**: 0 / 1 / 空文字 / 最大長（`sanitize_filename` の 200 バイト制約は文字境界を守るため必ずテスト）
- **異常系**: 不正 URL（`http://` なし / 閉じ括弧付き）/ 無音トラック / 途中で切れた出力
- **same-input**: 純粋関数は同じ入力に同じ出力（`determinism` スキル）

## 実行・監査

```bash
cargo test            # 全テスト実行（純粋関数。headless 可）
cargo test <name>     # 特定テスト
cargo clippy --all-targets -- -D warnings   # テストコードもLint対象
```

- Sandbox（§6.2）では `cargo test` が実行できない。その場合は**代替検証のみ**とし、テスト実行を CI / 実機への残課題として明記する（`device-testing` スキルの L1）。
- TUI の描画・キー操作は L1 の対象外（`device-testing` スキルの L3 で実機検証）。

## よくある失敗

- `unwrap()` 濫用でテストを短くする → 禁止。`expect("reason")` も最小限に
- I/O 関数をモックして数字だけ稼ぐ → 禁止。ロジックを純粋関数に切り出す
- 既存テストを `#[ignore]` にして通す → 厳禁（AGENTS.md §3.2）
- snapshot 的な「出力をそのまま assert_eq」で実装詳細に依存する → 意味あるアサーションを優先

## 関連

- `docs/arch/quality.md` — テスト方針
- `determinism/SKILL.md` — 純粋関数の決定論
- `device-testing/SKILL.md` — 3層検証（L1/L2/L3）
- `AGENTS.md §3` — テスト・品質保証ルール
