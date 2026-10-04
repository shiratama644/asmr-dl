# Security — セキュリティ・プライバシー

> 正本: `src/config.rs`（`cookie_header`）・`src/runner.rs`（`sanitize_filename`）・`settings.json`（deny）・`AGENTS.md §6`。
> 個人利用ツールとしてのセキュリティ・プライバシー方針。

## 基本方針

- **個人の私的利用向けツール**。公開サービスではない。
- **DRM 回避はしない**（機能も持たない）。DRM 保護された配信は対象外。
- **利用規約・著作権法に従う**。自分が視聴・保存してよいコンテンツ（自分の作品、許可されたもの、私的利用の範囲）にだけ使うこと（ルート README「注意」）。

## 機密情報の扱い

| 情報 | 扱い |
|---|---|
| **cookies.txt**（Netscape 形式） | **ユーザー個人ファイル**（`~/cookies/...`）。**リポジトリに含めない**。プロファイルの `cookies` はパス指定のみ。例は `../examples/cookies.example.txt`（ダミー値） |
| **`.env` 等** | 本プロジェクトは使わない（設定は `config.toml`）。`.gitignore` で除外（`.agent/settings.local.json` / `agent-memory/`） |
| **設定ファイル**（`~/.config/asmr-dl/config.toml`） | ユーザー環境にのみ存在。リポジトリには**既定値**（`config.default.toml`）のみ |
| **出力メディア** | ユーザーの共有ストレージ（`Music/ASMR`）にのみ |

- `.agent/settings.json` の `deny` に `Read(**/cookies*)` / `Read(**/.env*)` を設定（Agent が機密を読まない）。
- コミット前の機密混入チェック: `git diff --cached --name-only | grep -E "(\.env|cookies)"`（`verify-doc-integrity` / `post_edit_verify.sh`）。

## ファイル名の安全性

- **`sanitize_filename`**（`src/runner.rs`）:
  - `/ \ : * ? " < > |` と制御文字を `_` に置換（Android 共有ストレージで使えない文字を回避）
  - 前後の空白・`.` を除去
  - **200 バイトに制限**（文字境界を守る — 多バイト文字の途中で切れない）
  - 空なら `audio`
- **パス走査の防止**: 出力先は `output_dir`（既定 `~/storage/shared/Music/ASMR`）以下のみ。`profile.subdir` / `filename` の `/` でサブフォルダを作るが、`..` の混入は `sanitize_filename` と `unique_path` で無効化（`..` は制御文字ではなく `.` で始まるため前後除去で空になり `audio` に置換される — 実装時に確認）。
- yt-dlp 側でも `--windows-filenames` でファイル名を安全化。

## 外部プロセスの扱い

- **コマンドインジェクションの防止**: 外部コマンド（yt-dlp/ffmpeg/ffprobe/termux-*）は `tokio::process::Command` で**引数配列**として渡し、シェル経由にしない（`sh -c` を使わない）。URL は最後の引数（`-- <url>`）。
- **プロセスの管理**: `kill_on_drop(true)` + `process_group(0)`（unix）。キャンセル時に**プロセスグループごと** kill（孤児プロセス防止）。
- **`spawn_detached`**: `termux-*` は Termux:API 不在で固まるため、`kill_on_drop` + 20秒タイムアウトで kill。

## ネットワーク

- **任意の URL** に接続する（ユーザー指定）。HTTPS を含む。
- プロファイルの `referer` / `user_agent` / `headers` / `cookies` はユーザーが設定した値をそのまま送出（ユーザーの責任）。
- **推測のヘッダ値・cookies 値を設定しない**（`site-onboarding` スキル）。
- yt-dlp の `--cookies` はユーザーの cookies.txt を参照（ユーザーの責任で管理）。

## サプライチェーン

- **依存の最小化**: 新規クレート追加はユーザーの明示的合意が必要（`AGENTS.md §6.1`）。
- **yt-dlp の更新**: サイト対応・セキュリティのため頻繁に更新（`ops/UPDATING_YTDLP.md`）。`pip install -U yt-dlp`。
- **Cargo.lock をコミット**: bin なので依存バージョンを固定。

## 監査

- 機密・成果物（`target/`）のコミット混入は `post_edit_verify.sh` で検出。
- 差分レビューは `docs/audit/diff-{context}.md`（`.agent/skills/diff-review-report/`）。

## 関連

- [termux.md](./termux.md) — 保存先の扱い
- [profiles.md](./profiles.md) — cookies.txt の扱い
- `../examples/cookies.example.txt` — cookies.txt の例（ダミー）
- `.agent/settings.json` — Agent の機密読取禁止（deny）
