# Product — プロダクト定義

## 一言でいうと

> **URL を入れると、その動画・音声を全部 Opus（.opus）にしてくれる TUI。**

ASMR 向けのサイト別プロファイル付きで、**Termux（Galaxy S24 Ultra）** で長時間の ASMR 配信アーカイブを安定して落とすことを想定した個人利用ツール。

## 解決する問題

- 好きな ASMR 作品（動画・音声）をオフラインで聴きたいが、サイトごとに取得方法・音質がバラバラ
- 動画は容量が大きく、音声のみが必要（スマホのストレージ・バッテリーに優しい）
- バイノーラル/ステレオの定位（ASMR の生命線）を壊したくない
- 画面を眺めながら複数URLをキューに入れて、完了を通知で受け取りたい

## 価値提案

| 機能 | ユーザー価値 |
|---|---|
| **何でも Opus に** | 動画なら音声トラック、音声ならそのまま。元が Opus（YouTube 251 等）なら**無劣化コピー** |
| **ASMR 向けの音質設定** | チャンネル数を維持（定位を壊さない）、`-application audio`・VBR・48kHz、normalize は既定 OFF（小さな音のダイナミクスを残す） |
| **サイト別プロファイル** | URL から自動判定（Tab で手動切替）。Referer / UA / headers / cookies.txt / 並列数 / プレイリスト / チャプター分割をサイトごとに設定 |
| **2段構えの取得** | yt-dlp → 失敗したら ffmpeg 直接（m3u8 直リンク向け） |
| **キュー + 同時実行** | 既定 2 本。進捗・速度・残り時間。中止 / 再試行 |
| **壊れたファイル対策** | 変換後に長さを検証し、途中で切れていたら失敗扱い |
| **Termux 連携** | 共有テキストからの URL 抽出 / 共有メニュー連携 / タップ選択 / wake-lock / 通知 / メディアスキャン / `o` キーで再生 |

## 非目標（Non-Goals）

- **DRM 回避** — しない（機能も持たない）。`security.md` 参照
- **デスクトップ向けの高度な機能** — GUI 設定画面・ライブラリ公開等。CLI/TUI の個人ツールに留める
- **ライブ配信のリアルタイム録画** — アーカイブ（既存の配信）の取得が対象
- **複数端末・同期** — 単一端末（Termux）で完結

## 用語

| 用語 | 意味 |
|---|---|
| **ジョブ** | 1 URL = 1 ジョブ。状態: `待機中 / 実行中 / 完了 / 失敗 / キャンセル` |
| **プロファイル** | サイトごとの取得・変換設定の束（`[[profiles]]`）。URL から自動判定される |
| **generic** | フォールバックのプロファイル。**常に `config.default.toml` の最後** |
| **backend** | 取得経路。`auto`（yt-dlp→ffmpeg）/ `ytdlp` / `ffmpeg`（URL を直接 ffmpeg に） |
| **無劣化コピー** | 入力が既に Opus かつ加工指定が無ければ `ffmpeg -c:a copy`（再エンコードしない） |
| **Termux 連携** | `termux-wake-lock` / `termux-notification` / `termux-media-scan` / `termux-open` / 共有メニュー |
| **Sandbox** | Arena 等の作業用サンドボックス環境。Rust ツールチェーン無し（`AGENTS.md §6.2`） |

## 現行資産（v0.1.0 時点、2026-10-04）

| 資産 | 状態 |
|---|---|
| TUI（入力 / ジョブラスト / ログ / プロファイル選択 / ヘルプ / 終了確認） | 実装済み |
| 取得パイプライン（yt-dlp 経路 / ffmpeg 直接経路 / auto） | 実装済み |
| Opus 変換（libopus / 無劣化コピー / loudnorm 任意 / フィルタ任意） | 実装済み |
| 既定プロファイル 10 種（youtube / niconico / bilibili / twitch / soundcloud / x / tiktok / hls-direct / file-direct / generic） | 実装済み |
| キュー + 同時実行 + キャンセル + 再試行 | 実装済み |
| Termux 連携（wake-lock / 通知 / メディアスキャン / 共有メニュー / termux-open） | 実装済み |
| 単体テスト（config: パース/判定/URL抽出、runner: 名前/時間） | 実装済み |
| CI | 未導入（`cicd.md`） |

## 関連

- [architecture.md](./architecture.md) — モジュール構造
- [pipeline.md](./pipeline.md) — 取得/変換の詳細
- [profiles.md](./profiles.md) — プロファイル
- [milestones.md](./milestones.md) — フェーズ
