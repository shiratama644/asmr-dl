# Ops 運用手順 — asmr-dl

各手順は**第1人称で実行可能**に書かれた運用手順。設計の「なぜ」は `../arch/`、運用の「どうやる」はここ。

| 文書 | 用途 |
|---|---|
| [INSTALL_TERMUX.md](./INSTALL_TERMUX.md) | Termux（Android）に asmr-dl と依存一式をインストール |
| [TERMUX_BATTERY.md](./TERMUX_BATTERY.md) | 長時間ダウンロードのための電力管理（wake-lock / バッテリー最適化） |
| [UPDATING_YTDLP.md](./UPDATING_YTDLP.md) | yt-dlp を更新して取得が落ちるサイトを直す |

## 共通注意（Termux 前提）

- 作業はすべて `pkg` / `pip` / `cargo` の3種で進める。root 権限は不要。
- 実行前に `pkg update && pkg upgrade -y` を推奨（長い）。
- `~/.config/asmr-dl/config.toml` を編集したら、**asmr-dl を再起動**して反映する。
- 共有ストレージへのアクセスは初回 `termux-setup-storage` が必須（`~/storage/shared` が存在するか確認）。

## トラブルシューティング（よくあるもの）

| 症状 | まず見るもの |
|---|---|
| `command not found: yt-dlp` | `INSTALL_TERMUX.md` の pip 手順 / `hash -r` |
| YouTube が 403 | `UPDATING_YTDLP.md`（yt-dlp を更新 + `--js-runtimes node` を確認） |
| 共有フォルダに出力されない | `TERMUX_BATTERY.md` のストレージ権限（`termux-setup-storage`） |
| 充電しつつダウンロード中アプリが止まる | `TERMUX_BATTERY.md` の wake-lock / バッテリー最適化 |
| `ffprobe` / `ffmpeg` が見つからない | `pkg install ffmpeg`（再インストール） |
