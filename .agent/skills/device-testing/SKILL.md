---
name: device-testing
description: TUI(asmr-dl)の検証戦略。Sandbox では cargo/TUI が実行できないため、単体テスト（純粋関数）/ CLI 確認 / 実機(Termux)テストマトリクスの3層で検証し、「実環境検証待ち」タスクを docs/task-list.md で管理する。TEMPLATE_REPO の e2e (Playwright) を TUI 向けに転用。Use when changes need runtime verification, device testing is required, or you must split a task into testable + device-verification parts.
---

# Device Testing — TUI の3層検証戦略

> 出典: TEMPLATE_REPO の `e2e/SKILL.md`（Playwright 前提）を、**ブラウザが無い TUI プロジェクト向け**に転用
> 正本: `docs/arch/quality.md`、`AGENTS.md §6.2`、`docs/task-list.md`

## なぜ3層か

asmr-dl は headless では動かない（TUI + 外部コマンド + Android 連携）。検証を1層にすると
「Sandbox で確認できない」がデフォルトになり、未検証の変更が溜まる。そのため検証を3層に分け、
**どこで検証したか・どこに残課題があるかを task-list で明示する**。

| 層 | 場所 | 対象 | Sandbox で実行可 |
|---|---|---|---|
| L1: 単体テスト | `cargo test`（`#[cfg(test)]`） | 純粋関数（URL 抽出 / 名前 sanitize / プロファイル判定 / 時間書式 / progress パース） | ❌（ツールチェーン無し、§6.2）— CI / 実機で実行 |
| L2: CLI / 構文確認 | `cargo build --release` 後 `asmr-dl --help` / `--print-config` / tomllib 解析 / `sh -n` | 起動・ヘルプ・設定生成・TOML 構文 | 一部 ✅（tomllib / sh -n）。`--help` はビルド後のみ |
| L3: 実機(Termux) | Galaxy S24 Ultra（or 同等 Android） | TUI 描画・キー操作・ダウンロード・変換・Termux 連携 | ❌ — ユーザー環境で実行 |

## L1: 単体テスト（純粋関数）

- テストは `#[cfg(test)] mod tests`（同一 `.rs` 文件内）。既存テストは `src/config.rs` / `src/runner.rs`。
- 純粋関数（I/O なし）をテストする。既存のテスト対象:
  - `config.rs`: `default_config_parses`（TOML パース + generic 末尾）、`detection`（プロファイル自動判定）、`urls_from_share_text`（共有テキストからの URL 抽出）
  - `runner.rs`: `names`（name_from_url / is_generic_name / sanitize_filename）、`hms`（fmt_hms）
- **新しいロジックは純粋関数に切り出してテストする**（`testing/SKILL.md` 参照）。
- L1 は「壊れたロジックを検出」する層。描画・I/O は対象外。

## L2: CLI / 構文確認

```bash
# Sandbox で実行可（cargo 無しでも）
python3 -c "import tomllib; tomllib.load(open('config.default.toml','rb')); print('OK')"
sh -n scripts/install-termux.sh
sh -n scripts/termux-url-opener
jq empty .agent/settings.json

# cargo がある環境では追加
cargo build --release
./target/release/asmr-dl --help        # 使い方表示、exit 0
./target/release/asmr-dl --print-config  # config.default.toml 内容を出力、exit 0
```

- `--help` / `--print-config` は TUI を起動しない（`src/main.rs` で `ratatui::init()` より前に return）。
  したがって「headless でも動く」少数のコマンドであり、ビルド後の smoke として有効。

## L3: 実機(Termux)テストマトリクス

### 3.1 起動・表示

| # | 手順 | 期待結果 |
|---|---|---|
| D1 | `asmr-dl` で起動 | Header に `♪ ASMR→Opus` + ツール状態（`yt-dlp✓ ffmpeg✓ opus✓` 等）。`?` でヘルプ表示 |
| D2 | 起動直後 | `yt-dlp✗` / `ffmpeg✗` / `opus✗` があれば理由（未インストール / libopus 無し）が status に表示 |
| D3 | 縦画面（40〜60 桁）で起動 | レイアウトが崩れない。行が切れない |

### 3.2 入力・キュー

| # | 手順 | 期待結果 |
|---|---|---|
| D4 | URL を入力 → Enter | `N 件をキューに追加`。ジョブラストに `… 待機中` 表示 |
| D5 | 共有テキスト（`見て→https://... です`）を貼り付け | URL だけ抽出されてキューに追加 |
| D6 | 複数 URL を一度に貼り付け | 全てキューに追加（一括） |
| D7 | `Tab` / `Shift+Tab` | プロファイルが切り替わる（`プロファイル: <name>` flash） |
| D8 | `Ctrl+B` | ビットレートが preset 順に切替。一周で既定に戻す |
| D9 | 引数で起動: `asmr-dl https://youtu.be/xxx` | 起動と同時にキューへ（termux-url-opener 連携） |

### 3.3 ダウンロード・変換

| # | 手順 | 期待結果 |
|---|---|---|
| D10 | YouTube（Opus 251）を DL | `Opusのままコピー(無劣化)` ログ。`.opus` が `Music/ASMR/YouTube/...` に保存 |
| D11 | m3u8 直リンク（`hls-direct`）を DL | ffmpeg 直接経路で DL+変換。progress（速度/ETA）表示 |
| D12 | 複数ファイルを DL（`playlist = true`） | 各ファイルが変換され、`DL i/n` 表示 |
| D13 | 完了時 | 終了後に `完了 N 件 → <path>` 表示。`termux-media-scan` / 通知（Termux:API ありの場合） |

### 3.4 キャンセル・再試行

| # | 手順 | 期待結果 |
|---|---|---|
| D14 | 実行中に `c` | キャンセル。yt-dlp が起動した ffmpeg も停止（残存しない） |
| D15 | 失敗ジョブで `r` / 全失敗で `R` | 再試行。`──── 再試行 ────` ログ |
| D16 | `q`（実行中） | 確認ダイアログ。`y` で終了、else で継続 |

### 3.5 Termux 連携

| # | 手順 | 期待結果 |
|---|---|---|
| D17 | 他アプリから「共有 → Termux」 | `termux-url-opener` 経由で URL をキューに入れた状態で起動 |
| D18 | `o`（完了ファイル） | Android のアプリで開く（`termux-open`） |
| D19 | 画面 OFF で長時間 DL | `termux-wake-lock` により停止しない（Galaxy 設定も要） |

### 3.6 異常系

| # | 手順 | 期待結果 |
|---|---|---|
| D20 | 音声トラック無しの動画を DL | `音声トラックがありません` で失敗（変換前に検出） |
| D21 | 途中で切れた出力 | 長さ検証で失敗扱い（`出力が途中で切れています`） |
| D22 | 403 の m3u8 | プロファイルに `referer` / `headers` を足すよう誘導 |
| D23 | `termux-setup-storage` 未実行 | 出力先が `~/asmr-dl-output` にフォールバック |

## 「実環境検証待ち」タスクの管理

- L3 が必要だが Sandbox で実行できない変更は、`docs/task-list.md` に **`実環境検証待ち`** 状態で登録する（完了させない）。
- 証拠欄に「コミット SHA + 上記マトリクスの該当 #（例: D10, D14）」を書く。
- ユーザーが実機で確認し「OK」をもらったら `完了` に更新し、証拠に「実機確認 YYYY-MM-DD（ユーザー）」を回填。
- **L1/L2 が通っていても L3 未確認の変更は「完了」にしない**（AGENTS.md §5 の停止条件と整合）。

## 禁止事項

- 「Sandbox で動かなかった」を「動かない」断定にしない（制約と挙動の混同）。必ず §6.2 の制約を根拠に明記する。
- TUI の描画・キー操作を Sandbox で無理に emu しない（`tmux` / pseudo-tty で代用すると誤った結論が出る）。
- 実機検証手順を「〜を確認してください」の一文で終わらせず、**手順 + 期待結果**（上記マトリクス形式）で書く。

## 関連

- `docs/arch/quality.md` — 品質ゲートの設計
- `sandbox-constraints/SKILL.md` — Sandbox 制約の根拠
- `docs/task-list.md` — 進捗・「実環境検証待ち」の正本
