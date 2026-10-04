# Tech Stack — Rust + 外部ツール + Termux

> 正本: `Cargo.toml` / `config.default.toml` / `scripts/`。ハマりどころの詳細は `.agent/skills/tech-stack/SKILL.md`。

## 言語・ランタイム

| 項目 | 採用 | 備考 |
|---|---|---|
| 言語 | Rust `edition = "2021"`（stable） | 依存は最小構成 |
| パッケージ管理 | Cargo | `Cargo.lock` をコミット（bin なので锁を固定） |
| 非同期 | tokio 1（rt-multi-thread, macros, process, sync, time, io-util, fs） | キー入力は専用 OS スレッド |

## 依存（Cargo.toml、最小構成を維持）

| クレート | バージョン | 用途 |
|---|---|---|
| ratatui | 0.30 | TUI 描画 + crossterm（再エクスポート）。feature: `unstable-rendered-line-info` |
| tokio | 1 | 非同期ランタイム・プロセス管理 |
| serde | 1 | シリアライズ（derive） |
| toml | 0.8 | 設定ファイルの parse |
| anyhow | 1 | エラー処理 |
| regex | 1 | URL 抽出・プロファイルの url_regex |
| url | 2 | URL 解析（host 抽出） |
| unicode-width | 0.2 | 描画幅計算（全角対応、縦画面レイアウト） |
| libc | 0.2 | process group kill / localtime_r（unix） |

- **crossterm を直接依存にしない**（ratatui が再エクスポート。バージョン不一致防止）。
- **新規クレート追加はユーザーの明示的合意が必要**（`AGENTS.md §6.1`）。

## Release プロファイル

```toml
[profile.release]
opt-level = 3
lto = "thin"
codegen-units = 4
strip = true
panic = "unwind"
```

- S24 Ultra で `cargo build --release` は数分。
- `panic = "unwind"` は panic 時の画面復元フック（`ratatui::init`）を踏まないため。

## 外部コマンド（実行時依存、コードには含めない）

| コマンド | 用途 | 入手 |
|---|---|---|
| `yt-dlp` | 取得（サイト対応が広い） | `pip install -U yt-dlp`（YouTube 用は `yt-dlp-ejs` + `nodejs`） |
| `ffmpeg` / `ffprobe` | 変換（libopus）/ 解析 | `pkg install ffmpeg`（**libopus エンコーダ必須**） |
| `termux-*` | Termux 連携 | `pkg install termux-api` + Termux:API アプリ |

- パスは `config.toml` の `ytdlp_path` / `ffmpeg_path` / `ffprobe_path` で指定可。
- 起動時にバックグラウンドで検出（`--version` / `-encoders` に `libopus`）。Header に `yt-dlp✓ ffmpeg✓ opus✓` を表示。

## ターゲット環境

| 環境 | 対応 | 備考 |
|---|---|---|
| **Termux (Android)** | ◎（設計基準: Galaxy S24 Ultra / One UI） | 共有ストレージ / 通知 / メディアスキャン / 共有メニュー |
| Linux (デスクトップ) | ○ | 動作はするが Termux 連携は無音でスキップ |

- Termux の `rust` パッケージは stable。rustup は不要（Sandbox では動かない）。
- 共有ストレージは `termux-setup-storage` で許可が必要（`termux.md`）。

## CI / ツールチェーン

- **CI は未導入**（2026-10-04 時点）。将来構成は [cicd.md](./cicd.md)。
- 開発・検証は開発機（cargo あり）または実機。Sandbox では §6.2 の代替検証のみ。

## 関連

- [termux.md](./termux.md) — Termux 連携の詳細
- [quality.md](./quality.md) — 品質ゲート
- `.agent/skills/tech-stack/SKILL.md` — ハマりどころ（progress-template / --js-runtimes / process group kill 等）
