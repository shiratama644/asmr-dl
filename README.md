# asmr-dl 🎧

URL を入れると、そのページの **動画・音声（HLS/m3u8、DASH/mpd、mp4、webm、mp3 など）を全部 Opus 音声（.opus）にして保存する** Rust + Ratatui 製の TUI です。
ASMR 向けのサイト別プロファイル付きで、**Termux（Galaxy S24 Ultra）** での利用を想定して作っています。

```
 ♪ ASMR→Opus │ yt-dlp✓ ffmpeg✓ opus✓ │ ⇣1 · 1 待機
╭ URL  Enter=追加 · Esc=一覧 ──────────╮
│ https://youtu.be/xxxxxxxx            │
╰──────────────────────────────────────╯
◈  youtube 自動  160k ch維持 opus→copy  auto
  YouTube / YouTube Music。Opus(251)を優先
╭ ジョブ  1/3 完了  ⇣1 実行中 ──────────╮
│  ✔ [youtube] 【耳かき】安眠ASMR       │
│   → YouTube/ch/【耳かき】安眠ASMR.opus│
│▸ ⠸ [niconico] 囁き雑談 2時間          │
│   ██████░░░░ 58.2% 転換中 4.1MiB/s 残 │
│  … [hls-direct] index                 │
│   待機中 · 128kbps                    │
╰──────────────────────────────────────╯
```

> UI は Catppuccin Mocha ベースのダークテーマ（角丸枠・ヘッダ/フッタのステータスバー・実行中ジョブは回転スピナー表示）。

## 特徴

- **何でも Opus に**：動画ならトラック、音声ならそのまま。元から Opus（YouTube の 251 など）なら**再エンコードせずに無劣化でコピー**します
- **ASMR 向けの音質設定**
  - チャンネル数は元のまま（**バイノーラル/ステレオの定位を崩さない**）
  - libopus の `-application audio`・VBR・compression 10・48kHz
  - 音量正規化は**既定で OFF**（小さな音のダイナミクスを残す）。`normalize` / `filter` で調整可能
- **サイト別プロファイル**（URL から自動判定。Tab で手動切替）：YouTube / ニコニコ / bilibili / Twitch / SoundCloud / X / TikTok / m3u8 直指定 / ファイル直リンク / その他
  - Referer・User-Agent・追加ヘッダ・cookies.txt・並列フラグメント数・プレイリスト・チャプター分割などをプロファイルごとに設定可能
- **2 段構え**：yt-dlp で取得 → 失敗したら ffmpeg で URL を直接取得（m3u8 直リンク向け）
- **キュー + 同時実行**（既定 2 本）、進捗・速度・残り時間表示、中止 / 再試行
- **壊れたファイル対策**：変換後に長さを検証し、途中で切れていたら失敗扱い
- **Termux 連携**
  - 共有テキストごと貼り付け OK（`見て→https://youtu.be/... です` から URL を抽出）／複数 URL の一括貼り付け
  - 他アプリの「共有 → Termux」で直接キューへ（`termux-url-opener`）
  - タップでジョブ選択、スワイプでスクロール
  - `termux-wake-lock` / 完了通知 / メディアスキャン（音楽アプリにすぐ表示）/ `o` キーで Android アプリで再生
  - 縦画面（40〜60 桁）でも崩れないレイアウト。Android で使えないファイル名文字は自動で置換

## セットアップ（Termux）

> Termux は **F-Droid 版 または GitHub 版**を使ってください（Play ストア版は古いです）。通知・メディアスキャンを使う場合は **Termux:API** アプリも同じ配布元から入れます。

### かんたん（スクリプト）

```bash
# このフォルダを Termux に置いてから
cd asmr-dl
bash scripts/install-termux.sh
```

### 手動

```bash
pkg update && pkg upgrade
pkg install rust ffmpeg python nodejs termux-api
pip install -U yt-dlp yt-dlp-ejs mutagen
termux-setup-storage            # 「許可」を押す → ~/storage/shared が使えるようになる

cd asmr-dl
cargo build --release           # S24 Ultra なら数分
cp target/release/asmr-dl $PREFIX/bin/
```

- `nodejs` + `yt-dlp-ejs`：現在の YouTube は yt-dlp に JavaScript ランタイムが必要です。youtube プロファイルでは `--js-runtimes node` を自動で付けています
- yt-dlp はサイト側の変更に合わせて頻繁に更新されます。取れなくなったら、まず `pip install -U yt-dlp yt-dlp-ejs` を試してください（`docs/ops/UPDATING_YTDLP.md`）

### Galaxy（One UI）でのおすすめ設定

長時間の ASMR 配信アーカイブを落とすときに、途中で止まらないようにするための設定です（詳細は `docs/ops/TERMUX_BATTERY.md`）。

1. **設定 → アプリ → Termux → バッテリー → 「制限なし」**
2. **設定 → バッテリー → バックグラウンドでの使用を制限 → スリープ状態にしないアプリ** に Termux を追加
3. Android 14 以降なら **開発者オプション → 「子プロセスの制限を無効にする」** を ON（yt-dlp/ffmpeg が子プロセスとして強制終了される「Phantom Process Killer」対策）
4. asmr-dl は起動中 `termux-wake-lock` を自動で取ります（`wake_lock = false` で無効）

## 使い方

```bash
asmr-dl                          # 起動して URL を貼り付け → Enter
asmr-dl https://youtu.be/xxxx    # 引数の URL をすぐキューへ（複数可）
```

**共有メニューから**：YouTube アプリやブラウザで「共有 → Termux」→ asmr-dl がその URL をキューに入れた状態で起動します（`~/bin/termux-url-opener` が必要。インストールスクリプトで自動配置）。

保存先は既定で **`内部ストレージ/Music/ASMR/<サイト>/<チャンネル>/<タイトル>.opus`** です。サムネイル（.jpg）も同じ名前で横に置きます（対応プレイヤーではジャケットとして表示されます）。

### キー操作

Termux の追加キー行（ESC / TAB / CTRL / 矢印）だけで操作できるようにしています。

| 入力モード | |
|---|---|
| `Enter` | URL をキューに追加（複数 URL 可） |
| `Tab` / `Shift+Tab` | プロファイル切替（自動 → 各サイト） |
| `Ctrl+P` / `F2` | プロファイル一覧から選択 |
| `Ctrl+B` | ビットレート切替（64〜256k、一周で既定に戻る） |
| `Ctrl+U` / `Ctrl+W` | 全消去 / 1 語消去 |
| `Esc` / `↓` | ジョブ一覧へ |

| 一覧モード | |
|---|---|
| `↑↓` / `j k` / タップ | 選択（選択中をもう一度タップでログ表示切替） |
| `Enter` / `l` | ログ表示切替 |
| `PgUp` / `PgDn` / スワイプ | ログスクロール |
| `c` / `x` | 中止（yt-dlp が起動した ffmpeg もまとめて停止） |
| `r` / `R` | 再試行 / 失敗をまとめて再試行 |
| `d` / `C` | 削除 / 完了済みを一覧から消す |
| `o` | 完了ファイルを Android のアプリで開く |
| `p` / `b` / `Tab` | プロファイル選択 / ビットレート / プロファイル切替 |
| `i` / `Esc` | 入力モードへ |
| `?` / `F1` | ヘルプ（ツールの検出状況も表示） |
| `q` / `Ctrl+C` | 終了（実行中なら確認あり） |

## 設定・プロファイル

初回起動時に `~/.config/asmr-dl/config.toml` が作られます（元は `config.default.toml`。各項目にコメントあり）。

```toml
[[profiles]]
name = "my-asmr-site"
description = "Referer と Cookie が必要な ASMR 配信サイト"
domains = ["example-asmr.jp"]          # ホスト名のサフィックス一致
# url_regex = 'example-asmr\.jp/works/' # 正規表現でも指定可
backend = "auto"                       # auto / ytdlp / ffmpeg
referer = "https://example-asmr.jp/"
headers = ["Origin: https://example-asmr.jp"]
cookies = "~/cookies/example-asmr.txt" # Netscape 形式 cookies.txt
playlist = true                        # 作品ページ内の複数トラックをまとめて取得
split_chapters = true                  # チャプター付き長尺をトラックごとに分割
concurrent_fragments = 8               # HLS の並列ダウンロード数
subdir = "MySite"
filename = "%(uploader).60B/%(title).150B.%(ext)s"  # yt-dlp 出力テンプレート

[profiles.audio]
bitrate_kbps = 160      # バイノーラルは 128〜160 推奨
channels = 0            # 0 = 元のまま
copy_if_opus = true     # 元が Opus なら無劣化コピー
normalize = false       # loudnorm（I=-24, LRA=20 の緩め設定）
filter = "highpass=f=20"  # 任意の ffmpeg -af
```

- プロファイルは**上から順に判定**され、どれにも当たらなければ `generic` が使われます
- **m3u8 を直接指定する場合**：ブラウザの開発者ツールなどで取った `.m3u8` URL は `hls-direct` プロファイルになります。403 になるサイトは `referer` / `headers` を足してください
- **cookies.txt**：ログインが必要なサイト（ニコニコのプレミアム音質など）は、Android の Firefox / Kiwi 等の「cookies.txt を書き出す」系の拡張機能で書き出したファイルを指定します。yt-dlp 経路・ffmpeg 経路の両方で使われます
- 環境変数 `ASMR_DL_CONFIG` で別の設定ファイルを使えます。`asmr-dl --print-config` で既定値を表示

## 処理の流れ

```
URL ─┬─ [yt-dlp 経路] ─ yt-dlp → ~/.cache/asmr-dl/job-*/ ─┐
     │                    (失敗 & backend=auto)              ├─ ffprobe → ffmpeg
     └─ [ffmpeg 経路] ─ URL を直接 ──────────────────────────┘   ├ Opus なら -c:a copy（無劣化）
                                                                  └ それ以外は libopus
                                                   → 長さを検証 → Music/ASMR/... へ移動
                                                   → termux-media-scan / 通知
```

一時ファイルは Termux の内部領域（高速）に書き、完成してから共有ストレージへ移します。

## トラブルシューティング

| 症状 | 対処 |
|---|---|
| ヘッダーで `yt-dlp✗` / `ffmpeg✗` | `pip install -U yt-dlp` / `pkg install ffmpeg`。`?` で詳細 |
| `opus✗` | ffmpeg に libopus が無い → `pkg reinstall ffmpeg` |
| 出力フォルダを作成できません | `termux-setup-storage` を実行して許可。未設定の場合は `~/asmr-dl-output` に保存されます |
| YouTube で `No supported JavaScript runtime` | `pkg install nodejs` と `pip install -U yt-dlp-ejs` |
| HTTP 403 | プロファイルに `referer` / `headers` / `cookies` を設定（`docs/ops/UPDATING_YTDLP.md`） |
| 画面 OFF で止まる | 上の「Galaxy でのおすすめ設定」を参照（`docs/ops/TERMUX_BATTERY.md`） |
| 通知が来ない / 音楽アプリに出ない | Termux:API アプリと `pkg install termux-api` が必要（無くても本体は動きます） |
| 原因を詳しく見たい | 一覧で選んで `l`（ログ）。`keep_temp = true` で一時ファイルを残せます |

## 注意

- 自分が視聴・保存してよいコンテンツ（自分の作品、許可されたもの、私的利用の範囲など）にだけ使ってください。各サイトの利用規約と著作権法に従ってください
- DRM で保護された配信には対応していません（回避する機能もありません）

## 開発

```bash
cargo test        # 設定ファイルの構文・URL 判定などのテスト
cargo build --release
```

構成：

| ファイル | 内容 |
|---|---|
| `src/main.rs` | 起動・端末初期化・イベントループ・ツール検出 |
| `src/app.rs` | 状態管理・キー/マウス/貼り付け処理・キュー制御 |
| `src/ui.rs` | Ratatui 描画（縦画面向けレイアウト・ポップアップ） |
| `src/runner.rs` | yt-dlp / ffprobe / ffmpeg の実行、進捗解析、検証、保存 |
| `src/config.rs` | 設定・プロファイル・URL 抽出・cookies.txt 読み込み |
| `src/job.rs` | ジョブの状態とメッセージ |
| `config.default.toml` | 既定設定（バイナリに埋め込み） |

開発・Agent の運用規則（品質ゲート / Git / ドキュメント整合 / Sandbox 制約 / 実機検証）は **`AGENTS.md`**（§6 が asmr-dl 固有）と **`docs/`**（arch / planning / research / audit / ops）を参照してください。

## ライセンス

MIT（[LICENSE](./LICENSE)）
