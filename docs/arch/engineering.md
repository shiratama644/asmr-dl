# Engineering — エンジニアリング規約

> 正本: `src/*.rs` / `AGENTS.md §6` / `.agent/skills/`（determinism / memory-leak / zero-alloc / import-boundaries）。

## 言語・コード規約

- **コメント・UI 文案・ログ・コミットメッセージは日本語**。コード識別子は英語。
- **エラー処理は anyhow**: `?` + `.context()` / `.with_context()` で文脈を付与。
  - `bail!` は明らかな失敗（`音声トラックがありません` 等）。
  - **キャンセルは `CanceledError`**（`src/runner.rs`）で区別し、失敗通知に含めない。
- **`unwrap()` / `expect()` は「構造的に不可能」な箇所のみ**許容。理由をコメントで示す
  （例: `DEFAULT_CONFIG` の parse は `include_str!` の固定文字列のため `expect("default config is valid")`）。
- **`unsafe` は最小限**。`libc::kill`（process group kill）/ `localtime_r` 等。理由をコメントで書く。
- **公開 API（`pub`）には doc comment**（日本語可）。

## プロセス管理（最重要）

- 外部プロセスは必ず:
  ```rust
  cmd.kill_on_drop(true);
  #[cfg(unix)] cmd.process_group(0);
  ```
- キャンセル時は**プロセスグループごと** `SIGKILL`（`kill(-pid, SIGKILL)`）+ `child.kill()`。
- fire-and-forget（`termux-*`）は `spawn_detached`（`kill_on_drop` + 20秒タイムアウト）。
- 終了時は `cancel_all()` → `sleep(300ms)`（kill が届くのを待つ）。
- 詳細・監査コマンドは `.agent/skills/memory-leak/SKILL.md`。

## 決定論・純粋関数

- **純粋層（`config` / `job`）に I/O・時刻・乱数を混入させない**。
- ロジックは純粋関数に分離して `#[cfg(test)]` でテストする。
- `OnceLock<Regex>` は**コンパイル時固定値**のみ（`extract_urls` の regex）。
- 時刻を使う関数（`fallback_name` / `local_timestamp`）は `runner` の非純粋層に置き、doc で明示。
- 詳細は `.agent/skills/determinism/SKILL.md`。

## ホットパス（アロケーション）

- **200ms tick + 描画 + 行解析**がホットパス。
- 行解析は**スタックバッファ（8192B）+ 行バッファ再利用**（`read_lines`）。
- 呼び出し毎の regex コンパイルをしない（`OnceLock`）。
- イベントは `try_recv` で**バッチ処理**してから1回描画。
- 詳細は `.agent/skills/zero-alloc/SKILL.md`。

## モジュール境界

- 依存は下位へのみ（`config/job` → `runner` → `app` → `ui` → `main`）。
- `AppMsg`（メッセージ型）は `crate::` で共有してよい（例外として許容）。
- UI 状態（`HitAreas` / `picker_idx` 等）は `runner` へ漏らさない。
- 詳細は `.agent/skills/import-boundaries/SKILL.md`。

## 設定の扱い

- `config.default.toml` は `include_str!` で**バイナリに埋め込み**（`config.rs::DEFAULT_CONFIG`）。
- 初回起動時に `~/.config/asmr-dl/config.toml` を作成（無ければ既定値を書き出す）。
- `Config::normalize()`:
  - `profiles` が空なら既定の profiles を使う
  - `generic` が無ければ追加、**`generic` を常に末尾に移動**
  - `max_concurrent == 0` → 1
  - `bitrate_presets` が空なら既定
- 環境変数: `ASMR_DL_CONFIG`（設定パス）/ `XDG_CONFIG_HOME` / `HOME`。
- **`config.default.toml` を変えたら** `Config::default()` / `AudioSettings::default()` との整合、docs 同期を確認（rules/01 §5）。

## 命名

- モジュール: `snake_case.rs`（Rust 慣例）
- 関数: `snake_case`
- 型・列挙子: `PascalCase`（`Status` / `JobEvent` / `RunCtx`）
- 定数: `SCREAMING_SNAKE`（`MAX_LOG_LINES` / `NON_MEDIA_EXT` / `GENERIC_NAMES`）

## 関連

- [architecture.md](./architecture.md) — モジュール構造
- [pipeline.md](./pipeline.md) — プロセス起動の詳細
- `.agent/skills/`（determinism / memory-leak / zero-alloc / import-boundaries）
