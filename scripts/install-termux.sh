#!/data/data/com.termux/files/usr/bin/bash
# asmr-dl を Termux にセットアップするスクリプト
#   使い方: cd asmr-dl && bash scripts/install-termux.sh
set -euo pipefail

cd "$(dirname "$0")/.."

echo "==> パッケージを更新・インストール"
pkg update -y
pkg install -y rust ffmpeg python nodejs termux-api

echo "==> yt-dlp をインストール/更新 (yt-dlp-ejs は YouTube の JS チャレンジ用)"
pip install -U yt-dlp yt-dlp-ejs mutagen

echo "==> 共有ストレージへのアクセス許可 (ダイアログが出たら「許可」)"
if [ ! -d "$HOME/storage/shared" ]; then
  termux-setup-storage || true
  sleep 3
fi
mkdir -p "$HOME/storage/shared/Music/ASMR" 2>/dev/null || true

echo "==> ビルド (S24 Ultra で数分)"
cargo build --release
install -m 755 target/release/asmr-dl "$PREFIX/bin/asmr-dl"

echo "==> 共有メニュー連携 (~/bin/termux-url-opener)"
mkdir -p "$HOME/bin"
if [ -e "$HOME/bin/termux-url-opener" ]; then
  echo "   既存の termux-url-opener があるので上書きしません"
  echo "   手動で scripts/termux-url-opener の内容を追記してください"
else
  install -m 755 scripts/termux-url-opener "$HOME/bin/termux-url-opener"
fi

echo
echo "完了! 'asmr-dl' で起動します。"
echo "  設定: ~/.config/asmr-dl/config.toml (初回起動時に生成)"
echo "  保存: ~/storage/shared/Music/ASMR"
