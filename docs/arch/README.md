# docs/arch — 仕様書（asmr-dl）

ここは **どう作るか** の正本です。計画は `../planning/README.md`、進捗は `../task-list.md`。

asmr-dl は、URL から動画/音声（HLS/m3u8・DASH/mpd・直リンク等）を取得し、**全て Opus（.opus）に変換して保存**する Rust + Ratatui 製の TUI です。ASMR 向けのサイト別プロファイル付きで、**Termux（Galaxy S24 Ultra）** での利用を想定しています。

現行コードと本ディレクトリが食い違う場合、**本ディレクトリが目標**です。競合・外部技術調査は `../research/` を入口にし、仕様へ採用する場合は本ディレクトリへ反映してから実装します。

> **TEMPLATE_REPO からの差分（2026-10-04）**: bootstrap.md / detector.md / cache.md（テンプレートツール固有）は削除。代わりに本プロジェクトの中核である pipeline.md / profiles.md を追加。

## 実装時に守ること

1. **存在しない API を発明しない**。記載外の外部 API（yt-dlp オプション / ffmpeg フラグ等）は公式ドキュメントで実在とシグネチャを確認する。
2. **`adr.md` に反する実装をしない**。変更が必要なら実装せず人間に確認する。
3. **モジュール境界を壊さない**。依存は下位へのみ（`architecture.md` / `.agent/skills/import-boundaries/`）。
4. **純粋層（config / job）に I/O を混入させない**（`determinism` スキル）。
5. **既定の設定値（`config.default.toml`）を黙って変えない**。変更時は理由を残す（rules/01 §3）。
6. **壊れたファイルを出さない**。変換後の長さ検証（`pipeline.md` §5）は外さない。

## 読み方

| 状況 | 最初に読むもの | 次に読むもの |
|---|---|---|
| 実装を始める | `../task-list.md` → `../planning/README.md` | 対象フェーズの計画書 → 本 README の仕様書一覧 |
| 外部ツール（yt-dlp/ffmpeg）を使う | `tech-stack.md` | 公式ドキュメント（ffmpeg.org / yt-dlp GitHub） |
| ダウンロード/変換を変更 | `pipeline.md` | `src/runner.rs` / `adr.md` |
| 新サイト（プロファイル）を追加 | `profiles.md` | `../examples/profile-examples.toml` / `.agent/skills/site-onboarding/` |
| Termux 連携を変更 | `termux.md` | `../ops/INSTALL_TERMUX.md` |
| ADR に反しそう | `adr.md` | 実装せずユーザーへ確認 |

## 仕様書一覧

| ファイル | 内容 |
|---|---|
| [product.md](./product.md) | プロダクト定義・用語・現行資産 |
| [architecture.md](./architecture.md) | モジュール構造・依存規則・メッセージフロー |
| [pipeline.md](./pipeline.md) | ダウンロード/変換パイプライン（yt-dlp 経路 / ffmpeg 経路 / 検証 / 保存） |
| [profiles.md](./profiles.md) | サイト別プロファイル・自動判定・既定プロファイル一覧 |
| [tech-stack.md](./tech-stack.md) | Rust + 外部ツール（yt-dlp/ffmpeg）+ Termux |
| [termux.md](./termux.md) | Termux 連携・端末制約（wake-lock/通知/共有メニュー/保存先） |
| [quality.md](./quality.md) | 品質ゲート（fmt/clippy/test/build）・テスト方針 |
| [security.md](./security.md) | セキュリティ・プライバシー（cookies・ファイル名・DRM） |
| [engineering.md](./engineering.md) | エンジニアリング規約（エラー処理・プロセス管理・ホットパス） |
| [cicd.md](./cicd.md) | CI/CD（現状: 未導入、将来構成） |
| [adr.md](./adr.md) | 意思決定ログ（なぜそうしたか） |
| [milestones.md](./milestones.md) | 現行フェーズと次の候補 |
