# Ops: Termux へのインストール

> **対象**: Android 上の Termux で asmr-dl を動かすためのセットアップ（**第1人称実行手順**）。
> **事前条件**: Android 端末 + Termux（F-Droid 版推奨）。root 不要。
> **所要**: 依存のサイズ次第で 10〜40分（ビルド込み）。
> **自動版**: `scripts/install-termux.sh` が本手順の大部分を実行する（手動で確認したい場合は以下を読む）。

## 1. Termux の更新と基本パッケージ

```bash
pkg update && pkg upgrade -y
pkg install -y rust ffmpeg python nodejs termux-api git
```

| パッケージ | 用途 |
|---|---|
| `rust` | asmr-dl のビルド（`cargo build --release`） |
| `ffmpeg` | 音声抽出・Opus エンコード・ffprobe（長さ検証） |
| `python` | yt-dlp のランタイム（`pip install yt-dlp`） |
| `nodejs` | YouTube の JS チャレンジ解析（`--js-runtimes node`） |
| `termux-api` | wake-lock / 通知 / メディアスキャン / termux-open（**任意**） |

> `termux-api` パッケージは `termux-xxx` コマンド本体のみ。**動作には Play ストアの「Termux:API」アプリも必要**（アプリがシステム API と通信）。入っていなくても asmr-dl は動くが、wake-lock/通知が静かに失敗する。

## 2. Python 側に yt-dlp 一式

```bash
pip install --upgrade pip
pip install yt-dlp yt-dlp-ejs mutagen
```

| パッケージ | 用途 |
|---|---|
| `yt-dlp` | 動画/音声の取得本体 |
| `yt-dlp-ejs` | YouTube の JS エミュレーション（`--remote-components ejs:github` が読む） |
| `mutagen` | 一部のメタデータ処理（任意だが推奨） |

確認:

```bash
yt-dlp --version
ffmpeg -version | head -1
ffprobe -version | head -1
node --version
```

## 3. 共有ストレージの許可（初回のみ）

```bash
termux-setup-storage
```

- 許可ダイアログが現れたら「許可」。
- `~/storage/shared` ができたことを確認:

```bash
ls ~/storage/shared
```

- できていない場合: `rm -rf ~/storage` を**やらずに**、`termux-setup-storage` を再実行、または `reboot` 後に再実行する（Termux の既知の問題）。
- `config.toml` の `output_dir = "~/storage/shared/Music/ASMR"` がこの `~/storage/shared` を前提にしている。

## 4. asmr-dl のビルドとインストール

```bash
# リポジトリを取得（既に clone 済みなら不要）
git clone <あなたの asmr-dl リポジトリ> ~/asmr-dl-src && cd ~/asmr-dl-src

# release ビルド（数分）
cargo build --release

# Termux の $PREFIX/bin へ（PATH 済み）
install -m 0755 target/release/asmr-dl "$PREFIX/bin/asmr-dl"

# 動作確認
asmr-dl --help
```

## 5. 初回設定の生成

```bash
asmr-dl
```

- 初回起動で `~/.config/asmr-dl/config.toml` が `config.default.toml` 相当で作成される（バイナリに埋め込み済み）。
- `output_dir` / `bitrate` / プロファイルなどを確認:

```bash
$EDITOR ~/.config/asmr-dl/config.toml
```

- 編集したら **asmr-dl を再起動**して反映。

## 6. 共有メニュー登録（任意・推奨）

- **Android 側**: 動画サイト（YouTube 等）で「共有」→「Termux」を選ぶ設定を追加する。
- `scripts/install-termux.sh` は `~/bin/termux-url-opener`（`exec asmr-dl "$1"`）を作成する。
- 共有メニューが `termux-url-opener` を呼べるように Termux:API の共有設定に合わせる（アプリの共有先が `$PREFIX/bin/termux-url-opener` を指すこと）。

## 7. 動作確認（最小スモーク）

```bash
# 共有ストレージにダミーの m3u8 が無いなら、ffmpeg 直接のみの確認
which asmr-dl yt-dlp ffmpeg ffprobe
termux-wake-lock 2>&1 || echo "（Termux:API なし — OK、wake-lock は無効化される）"
```

| 確認項目 | 期待値 |
|---|---|
| `which asmr-dl` | `$PREFIX/bin/asmr-dl` |
| `yt-dlp --version` | バージョン表示 |
| `ffmpeg -version` | バージョン表示（`libopus` を含む: `ffmpeg -encoders \| grep opus`） |
| `ls ~/storage/shared` | 共有フォルダの中身 |

## 8. 初回取得のテスト

任意の m3u8 直リンクや YouTube の ASMR 動画 URL を、TUI の入力欄（Ctrl+V で貼り付け）で取得する:

1. `asmr-dl` を起動 → 入力欄に URL → Enter
2. ジョブラストに `queued` → `downloading` → `converting` → `completed`
3. 通知（Termux:API あり）+ 出力先 `~/storage/shared/Music/ASMR/*.opus`

## よくある失敗

| 症状 | 原因と対処 |
|---|---|
| `could not find src/*.rs` 系のビルドエラー | リポジトリのルート外から `cargo build`。リポジトリルートで実行する |
| `pip: command not found` | `python3 -m pip` を使う / `pkg install python` 後 `hash -r` |
| `~/storage/shared` が無い | 第3節再実行 / `termux-setup-storage` を許可 / reboot |
| `command not found: asmr-dl` | 第4節の `install` を忘れた / `export PATH="$PREFIX/bin:$PATH"` |
| YouTube が 403 | `UPDATING_YTDLP.md` を参照（yt-dlp 更新 + `--js-runtimes node`） |
| 充電中でも止まる | `TERMUX_BATTERY.md`（wake-lock / バッテリー最適化） |

## 関連

- [INSTALL_TERMUX.md](./INSTALL_TERMUX.md)（本ファイル）
- [TERMUX_BATTERY.md](./TERMUX_BATTERY.md) / [UPDATING_YTDLP.md](./UPDATING_YTDLP.md)
- 設計: `../arch/termux.md` / `../arch/profiles.md`
- スキル: `.agent/skills/device-testing/SKILL.md` / `.agent/skills/site-onboarding/SKILL.md`
