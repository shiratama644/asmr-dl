---
paths:
  - "AGENTS.md"
  - ".agent/skills/project-overview/**"
---

# Rule 05: プロジェクト固有の遵守事項（asmr-dl）

> 優先度: **HIGH** — このファイルは `AGENTS.md §6` と対応する asmr-dl 固有ルール。
> TEMPLATE_REPO のテンプレート節（Node/pnpm/Next.js 系）を**全て削除**し、asmr-dl（Rust TUI）に書き換えた。

## 1. 環境・ツールチェーン（§6.1相当）

- **言語**: Rust `edition = "2021"`、stable チャンネル推奨
- **パッケージマネージャ**: Cargo（`package.json` / Node / pnpm は存在しない）
- **主要依存（最小構成を維持）**: ratatui 0.30 / tokio 1 / serde / toml / anyhow / regex / url / unicode-width / libc
- **外部コマンド（実行時依存）**: `yt-dlp`（pip、YouTube 用は `yt-dlp-ejs` + `nodejs`）、`ffmpeg` / `ffprobe`（libopus 必須）
- **ターゲット環境**: Termux (Android)、設計基準は Galaxy S24 Ultra / One UI。デスクトップ Linux 動作も壊さない
- **新規クレート追加はユーザーの明示的合意が必要**

## 2. サンドボックス制約（§6.2相当）

乗り越えず、迂回する。2026-10-04 実測の恒常的制約。

| 制約 | 対処 |
|---|---|
| `sh.rustup.rs` / `static.rust-lang.org` / `crates.io` 到達不可 → Rust ツールチェーン不可 | `cargo build/test/clippy/fmt` は実行不可。代替検証: tomllib トムル解析 / `sh -n` / リンクチェック。最終検証は CI（未導入時は実機）で実施し報告に明記 |
| apt 不可（root なし） | 同上 |
| `termux-*` ビナリ不在 | 実機検証タスクとして分離。コードは「bin 無しの無音スキップ」方針維持 |
| TUI は headless 実行不可 | `--help` / `--print-config` もビルド後にのみ確認可能 |

## 3. リポジトリ固有の Git 制約（§6.3相当）

- `target/` は gitignore 済み。ビルド成果物をコミットしない
- ルートの `asmr-dl.zip` は初期ソース配布物。**編集・削除しない**
- `config.default.toml` は `include_str!` でバイナリ埋め込み。`Config::normalize()` の挙動（generic が末尾）と矛盾させない

## 4. 実装ルール（§6.4相当）

- モジュール依存は下位へのみ: `config/job`（純粋）← `runner`（I/O）← `app`（状態）← `ui`（描画）← `main`（配線）
- エラー処理は anyhow（`?` + `.context()`）。キャンセルは `CanceledError` で区別
- 外部プロセスは `kill_on_drop(true)` +（unix）`process_group(0)`、キャンセルはプロセスグループごと SIGKILL
- UI ホットパス（200ms tick）の不要アロケーションを避ける
- 純粋関数は必ず `#[cfg(test)]` でテスト
- コメント・UI 文案・ログ・コミットメッセージは日本語
- `unwrap()` / `expect()` は「構造的に不可能」な箇所のみ
- 新規プロファイル追加は `config.default.toml` に追記。`generic` は常に最後

## 5. 整形・Lint（§6.5相当）

- `cargo fmt`（rustfmt 既定、`rustfmt.toml` なし）
- `cargo clippy --all-targets -- -D warnings`。`#[allow]` は理由をコメントで必ず書く

## 6. UI 実装ルール（§6.6相当）

- 縦画面（40〜60 桁）で崩れないことを最優先。`unicode-width` で幅を計算
- 描画時に `HitAreas` を記録し `on_mouse` で使う（二重管理しない）
- 状態アイコンは Unicode（`… ⇣ ✔ ✖ ■`）
- ターミナル初期化・復元は `ratatui::init()` / `ratatui::restore()` に任せる

## 7. ドキュメント運用（§6.7相当）

- 構成・命名: `docs/README.md`
- 進捗正本: `docs/task-list.md`
- 仕様: `docs/arch/` / 計画: `docs/planning/` / 調査: `docs/research/` / 監査: `docs/audit/` / 運用: `docs/ops/` / 設定例: `docs/examples/`
- 追加・削除・移動時は `docs/README.md` の目次更新必須
- ファイル名は短く正確、ハイフン最大1つ

## 8. 新規タスク着手時のチェックリスト

- [ ] `AGENTS.md §6` の制約（特に §6.2 Sandbox 制約）を確認
- [ ] 対象タスクが `docs/task-list.md` に ID として登録されている
- [ ] 対象サブシステムの `docs/arch/*.md` と該当スキルを事前読込
- [ ] `config.default.toml` を触る場合は `Config::normalize()` / `Config::default()` との整合を確認
- [ ] 検証計画（cargo が実行可能か、代替検証に留まるか）を事前に確定
