# Milestones — 現行フェーズと次の候補

> 現行の到達点と、次にやる候補。タスクの詳細・進捗は `../task-list.md`（正本）。

## 現行: v0.1.0（2026-10-04 時点）

**個人利用向け ASMR 音声ダウンロード TUI の MVP が完成**している状態。

| 領域 | 到達点 |
|---|---|
| 取得 | yt-dlp 経路 / ffmpeg 直接経路 / auto（2段構え）。10 種の既定プロファイル |
| 変換 | Opus（libopus / 無劣化コピー / loudnorm 任意 / フィルタ任意）。長さ検証 |
| UI | 入力 / ジョブラスト / ログ / プロファイル選択 / ヘルプ / 終了確認。縦画面対応 |
| 制御 | キュー + 同時実行 + キャンセル（process group kill）+ 再試行 |
| Termux | wake-lock / 通知 / メディアスキャン / 共有メニュー / termux-open |
| テスト | 純粋関数の単体テスト（config / runner） |
| 整備 | TEMPLATE_REPO 適用（.agent/ + docs/ + AGENTS.md + README.md、2026-10-04） |

**未到達（残課題）**:
- CI（未導入）— `task-list.md` CI-1
- 実機検証マトリクスの実施 — `task-list.md` DEV-1
- `cargo build/test` の Sandbox 外での実行確認（§6.2）

## 次の候補（優先度）

| 優先 | 候補 | 内容 | 関連 |
|---:|---|---|---|
| 🟡 | **CI 導入** | push/PR で fmt/clippy/test/build を全PASS | `cicd.md` / CI-1 |
| 🟡 | **実機検証の制度化** | device-testing D1〜D23 を実機で実行し結果を記録 | DEV-1 |
| 🟡 | **新サイト対応の強化** | よく落ちるサイト（403 / JS チャレンジ）のプロファイル整備 | `site-onboarding` |
| 🟢 | **設定の UI 化** | TUI 内から `config.toml` を編集（現状はファイル編集） | — |
| 🟢 | **リリース自動化** | GitHub Releases へのバイナリ配布 | `cicd.md` |
| 🟢 | **依存更新** | ratatui / tokio / yt-dlp の定期更新 | `UPDATING_YTDLP.md` |
| 🟢 | **チャプター分割の強化** | `split_chapters` の名前付け・メタデータ | `pipeline.md` |

## 運用ルール

- フェーズを飛ばさない。`task-list.md` のタスクを完了させること。
- 候補を開始する場合は `task-list.md` に ID を登録してから `planning/` で計画する。
- 「ついでに」候補を始めない（AGENTS.md §1.1）。
