# Ops: yt-dlp の更新で取得を直す

> **対象**: 「以前は取得できたサイトが、突然 403 / "Sign in to confirm you're not a bot" / `yt_dlp.utils.ExtractorError` になる」場合の yt-dlp 更新と確認（**第1人称実行手順**）。
> **事前条件**: `INSTALL_TERMUX.md` を完了済み。
> **所要**: 通常 2〜5分。

## 0. まず「取得側の問題か」を見分ける

- **取得側**（yt-dlp / ネットワーク / サイトの仕様変更）: `failed: yt-dlp: ...`、`HTTP 403`、`Sign in to confirm...`、`Unable to extract` 等
- **変換側**（ffmpeg / Opus）: `failed: ffmpeg: ...`、`Invalid data found when processing input` 等 → yt-dlp 更新は**無意味**（ffmpeg 側を確認）

> 見分け方: ログでエラーの先頭が `yt-dlp:` か `ffmpeg:` か。

## 1. yt-dlp を更新

```bash
pip install -U yt-dlp yt-dlp-ejs
hash -r   # シェルのコマンドキャッシュを切る（重要）
yt-dlp --version   # 最新版の表示を確認
```

- `-U`（upgrade）を忘れない。
- `hash -r` を忘れると、旧バージョンの `yt-dlp` が呼ばれ「更新したのに直らない」に見える。

## 2. 直接実行で切り分け

TUI ではなく、**yt-dlp 単体**で同じ URL を実行し、エラーの種別を確認する:

```bash
yt-dlp -f bestaudio \
  --js-runtimes node \
  --remote-components ejs:github \
  -o "/tmp/test.%(ext)s" \
  "<URL>"
```

- 成功 → 取得側は直っている（TUI で再試行）
- 失敗 → 以下の3種に分類

| エラー | 種別 | 対処 |
|---|---|---|
| `Sign in to confirm you're not a bot` | **JS チャレンジ未解析** | `--js-runtimes node` と `yt-dlp-ejs` の有無を確認（第3節） |
| `HTTP 403` | **IP ブロック / 地域制限** | コokies 設定（第4節）/ 別回線 / 諦める |
| `Unable to extract ...` | **サイト仕様変更** | `--verbose` を付けて確認、`UPDATING_YTDLP.md` の手順再実行 / 新サイト対応（`site-onboarding` スキル） |

## 3. JS チャレンジ系（YouTube 等）の追加確認

```bash
# nodejs が存在するか
node --version

# yt-dlp-ejs が存在するか
pip show yt-dlp-ejs

# config.toml の YouTube プロファイルに以下があるか
#   "args": [ "--js-runtimes", "node", "--remote-components", "ejs:github" ]
grep -A 5 'id = "youtube"' ~/.config/asmr-dl/config.toml
```

- 無ければ追加（`site-onboarding` スキル参照）。
- それでも `Sign in to confirm...` なら、**その端末の IP がブロックされている可能性**が高い（第4節 or 別回線）。

## 4. cookies の設定（要ログインのサイト / 403 が続く場合）

1. PC のブラウザ（Chrome / Firefox）で、対象サイトに**ログイン**
2. 拡張機能「**Get cookies.txt LOCALLY**」等で cookies を `cookies.txt` としてエクスポート
3. 端末へコピー:

```bash
cp cookies.txt ~/.config/asmr-dl/cookies.txt
chmod 600 ~/.config/asmr-dl/cookies.txt
```

4. `config.toml` の対象プロファイルに追加:

```toml
[[profiles]]
id = "youtube"
domains = ["youtube.com", "youtu.be"]
cookies = "~/.config/asmr-dl/cookies.txt"   # ← 追加
# ... 既存の args / audio
```

5. **asmr-dl を再起動**して再試行。

> cookies は定期的に**失効**する。再度 403 になったら再エクスポート。
> **cookies をコミットしない**（`docs/examples/cookies.example.txt` はダミー値のみ）。

## 5. 新サイト・よく落ちるサイトへの恒久対応

- プロファイルの新規追加は、`.agent/skills/site-onboarding/SKILL.md` の手順で
- 調査結果は `docs/research/{TOPIC}_RESEARCH.md` に記録
- `config.default.toml` に反映（**generic は末尾のまま**）

## 6. 確認（復旧したか）

```bash
# TUI 内で再試行（Ctrl+R）または新規取得
# 完了条件: output_dir に .opus が生成され、再生できること
ls -lh ~/storage/shared/Music/ASMR/
```

## よくある失敗

| 症状 | 原因と対処 |
|---|---|
| 「更新したのに直らない」 | `hash -r` を忘れた / `pip` 側の `yt-dlp` が古い（`pip show yt-dlp`） |
| `Sign in to confirm...` が続く | `yt-dlp-ejs` が無い / `node` が無い / その IP がブロック（第3節・第4節） |
| `Unable to extract` | サイト仕様変更。`--verbose` を確認 / 新サイト対応（site-onboarding） |
| cookies を設定したのに 403 | cookies が失効 / 対象サイトが違う（第4節再実行） |

## 関連

- スキル: `.agent/skills/site-onboarding/SKILL.md`
- 設計: `../arch/profiles.md`（プロファイル判定の順）
- 調査: `../research/README.md`（調査済み一覧）
