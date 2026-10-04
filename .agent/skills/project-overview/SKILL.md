---
name: project-overview
description: asmr-dl（Rust + Ratatui 製 ASMR 音声ダウンロード TUI）の全体像（目的・技術スタック・モジュール構成・ドキュメント構成・進捗管理）を掴む。新規セッションの最初に読む1スキル。Use when starting a new session or needing overall context.
---

# Project Overview — asmr-dl

> **asmr-dl** は、URL から動画/音声（HLS/m3u8・DASH/mpd・直リンク等）を取得し、**全て Opus（.opus）に変換して保存**する Rust + Ratatui 製の TUI です。
> ASMR 向けのサイト別プロファイル付きで、**Termux（Galaxy S24 Ultra）** での利用を想定して作られています。

## 目的

- **速く大量に作ることではなく、常に復旧可能で、壊れた状態を長時間維持しないこと**を最優先とする（AGENTS.md）。
- 小さい実装 → 検証 → 修正 → Commit → 次の機能のサイクルを徹底。
- 個人利用向けのツールとして、ユーザーのコンテンツ（ASMR 作品）を安定して Opus に変換し続けられることを目指す。

## 技術スタック

| 層 | 採用 | 備考 |
|---|---|---|
| 言語 | Rust（edition 2021, stable） | 依存は最小構成 |
| TUI | ratatui 0.30（crossterm を再エクスポート） | `unstable-rendered-line-info` 有効。crossterm 直接依存はしない（バージョン不一致防止） |
| 非同期ランタイム | tokio 1（rt-multi-thread, process, sync, time, io-util, fs） | キー入力は専用スレッドで読み mpsc へ |
| 設定 | serde + toml | `config.default.toml` を `include_str!` でバイナリ埋め込み |
| エラー処理 | anyhow | `?` + `.context()`。キャンセルは `CanceledError` |
| 外部ツール | yt-dlp / ffmpeg / ffprobe | 実行時依存。パスは `config.toml` で指定可 |
| ターゲット | Termux (Android) + Linux | 設計基準: Galaxy S24 Ultra / One UI |
| ドキュメント | Markdown + `docs/` 構成 | `task-list.md`（正本）+ `arch/`（仕様）+ `planning/`（計画）+ `research/`（調査）+ `audit/`（監査）+ `ops/`（運用）+ `examples/`（設定例） |
| Agent設定 | `.agent/` | 公式 `.claude/` 構成に準拠（TEMPLATE_REPO 由来） |
| CI | 未導入（2026-10-04 時点） | ローカル品質ゲート（fmt/clippy/test/build）中心。`docs/arch/cicd.md` に将来構成 |

## モジュール構成（依存は下位へのみ）

```
src/
├── main.rs    # 起動・端末初期化・イベントループ・ツール検出（配線のみ）
├── app.rs     # アプリ状態・キュー制御・キー/マウス/貼り付け処理
├── ui.rs      # Ratatui 描画（縦画面向けレイアウト・ポップアップ・HitAreas記録）
├── runner.rs  # yt-dlp / ffprobe / ffmpeg の実行、進捗解析、出力検証、保存
├── config.rs  # 設定・プロファイル・URL抽出・cookies.txt読み込み（純粋中心）
└── job.rs     # ジョブの状態とメッセージ（純粋データ）
```

依存方向（上位が下位を参照してよい。逆は禁止）:

```
config / job  ←  runner  ←  app  ←  ui  ←  main
（純粋データ）   （I/O層）   （状態）  （描画）  （配線）
```

## 処理の流れ（要約）

```
URL ─┬─ [yt-dlp 経路] ─ yt-dlp → ~/.cache/asmr-dl/job-*/ ─┐
     │                    (失敗 & backend=auto)              ├─ ffprobe → ffmpeg
     └─ [ffmpeg 経路] ─ URL を直接 ──────────────────────────┘   ├ Opus なら -c:a copy（無劣化）
                                                                  └ それ以外は libopus
                                                   → 長さを検証 → Music/ASMR/... へ移動
                                                   → termux-media-scan / 通知
```

詳細は `docs/arch/pipeline.md` / `docs/arch/profiles.md`。

## ドキュメント構成

```
docs/
├── README.md          # 全ドキュメントの目次
├── task-list.md       # タスク管理の唯一の正本（進捗・証拠）
├── arch/              # 仕様書（どう作るか）
│   ├── README.md
│   ├── product.md
│   ├── architecture.md
│   ├── pipeline.md
│   ├── profiles.md
│   ├── tech-stack.md
│   ├── termux.md
│   ├── quality.md
│   ├── security.md
│   ├── engineering.md
│   ├── cicd.md
│   ├── adr.md
│   └── milestones.md
├── planning/          # 計画書（_TEMPLATE.md形式）
│   ├── README.md
│   ├── index.md
│   ├── _TEMPLATE.md
│   └── complete/      # 完了済み計画
├── research/          # 調査結果
├── audit/             # 差分・バグ監査（時点記録）
├── ops/               # 運用（Termux インストール・バッテリー・yt-dlp 更新）
└── examples/          # 設定例（プロファイル・cookies.txt）
```

## Agent設定構成（公式準拠）

```
.agent/
├── settings.json              # チーム共有設定（permissions, hooks, env）
├── rules/                     # トピック別ルール（pathsで発火条件）
├── skills/<name>/SKILL.md     # 再利用プロンプト（/nameで呼び出し）
├── agents/                    # サブエージェント定義
├── hooks/                     # フック手順(.md) + 実行スクリプト(.sh)
├── commands/                  # 旧commands互換
├── output-styles/             # 出力スタイル
├── workflows/                 # 動的ワークフロー（implement-task.js）
├── agent-memory/              # サブエージェント永続メモリ（gitignore）
└── logs/                      # タスク実行ログ（追加のみ）
```

## 進捗管理

- 正本は `docs/task-list.md`。チャット・Issue・AIの完了報告と矛盾する場合は本ファイルを正とする
- 仕様の正本は `docs/arch/`（どう作るか）、計画は `docs/planning/`（_TEMPLATE.md形式）、調査は `docs/research/`、監査は `docs/audit/`
- 進行中タスクは原則1件。複数を同時に進めない
- タスクIDは再利用しない。中止したタスクは行を消さず「対象外」にして理由を残す
- 完了はAIの自己申告ではなく証拠（テスト件数 / コミットSHA / PR / 実測値）で判定
- Sandbox では cargo が実行できないため（§6.2）、**「実環境検証待ち」状態を適切に使って**実機確認タスクを分離する

## Sandbox での注意（最重要）

- **Rust ツールチェーンは Sandbox に無い**（`sh.rustup.rs` / `static.rust-lang.org` / `crates.io` 到達不可、apt 不可 — 2026-10-04 実測）。`cargo build/test/clippy/fmt` は実行できない。
- 代替検証: `python3`（tomllib で TOML 解析）/ `sh -n`（shell 構文）/ Markdown リンクチェック（`verify-doc-integrity`）。
- 詳細は `sandbox-constraints/SKILL.md` と `device-testing/SKILL.md`。

## 関連

- 開発規約: `AGENTS.md`（§1〜§5は汎用、§6はプロジェクト固有）
- ルール: `.agent/rules/`（トピック別、pathsで発火）
- スキル: `.agent/skills/`（事実・仕様・ハマりどころ）
