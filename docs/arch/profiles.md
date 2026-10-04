# Profiles — サイト別プロファイル

> 正本: `config.default.toml`（`[[profiles]]`）・`src/config.rs`（`Profile` / `Config::detect_profile` / `normalize`）。
> 本ファイルはプロファイルの仕組みと既定プロファイル一覧を記録する。

## プロファイルとは

サイトごとの**取得・変換設定の束**。URL から自動判定され、Tab / `Ctrl+P`（または `p` / `F2`）で手動切替できる。

### 判定の仕組み（`Config::detect_profile`）

1. URL の host を抽出（`host_of`: `www.` を除去して小文字化）
2. **`generic`（末尾）以外の**プロファイルを上から順にチェック
3. 各プロファイルの `Profile::matches(url, host)`:
   - `url_regex`（空でなければ）が URL 全体にマッチすれば **true**（`domains` より先に評価）
   - それでなければ `domains` のいずれかとホスト名の**サフィックス一致**（`host_matches`）
4. いずれも無ければ **`generic`**（末尾のプロファイル）

> `generic` は常に最後の位置にあることを `Config::normalize()` が保証する（`config.default.toml` を編集する場合も末尾に保つこと、`AGENTS.md §6.4`）。

### プロファイルの主な項目

| 項目 | 意味 | 既定 |
|---|---|---|
| `name` | 識別子（一意） | `generic` |
| `description` | 説明（UI に表示） | 空 |
| `domains` | ホスト名サフィックス一致 | `[]` |
| `url_regex` | URL 全体の正規表現（優先） | 空 |
| `backend` | `auto` / `ytdlp` / `ffmpeg` | `auto` |
| `format` / `format_sort` | yt-dlp の `-f` / `-S` | `ba/b` / 空 |
| `referer` / `user_agent` / `headers` | HTTP ヘッダ（403 対策） | 空 |
| `cookies` | Netscape 形式 cookies.txt のパス | 空 |
| `concurrent_fragments` | HLS/DASH の並列フラグメント数（`-N`） | 4 |
| `playlist` | 作品ページ内の複数トラックをまとめて取得 | false |
| `split_chapters` | チャプター付き長尺をトラックごとに分割 | false |
| `write_thumbnail` | サムネイルを jpg で保存 | false |
| `embed_metadata` | 埋め込みメタデータ | true |
| `subdir` | `output_dir` 以下のサブフォルダ | 空 |
| `filename` | yt-dlp 出力テンプレート（`/` でサブフォルダ可） | `%(title).150B [%(id)s].%(ext)s` |
| `chapter_filename` | チャプター分割時のテンプレート | `%(title).100B/%(section_number)03d %(section_title).100B.%(ext)s` |
| `ytdlp_args` | yt-dlp に追加で渡す引数 | `[]` |
| `[profiles.audio]` | 音質設定（下記） | 下記 |

### 音質設定（`[profiles.audio]` / `AudioSettings`）

| 項目 | 意味 | 既定 |
|---|---|---|
| `bitrate_kbps` | Opus ビットレート（バイノーラル ASMR は 128〜160 推奨） | 128 |
| `channels` | 0 = 元のまま（ステレオ/バイノーラル維持）/ 1 = モノ / 2 = ステレオ | 0 |
| `copy_if_opus` | 元が Opus なら再エンコードせず無劣化コピー | true |
| `normalize` | `loudnorm` で音量を揃える（ASMR は小さい音が持ち味なので既定 OFF） | false |
| `filter` | 追加の ffmpeg `-af`（例 `highpass=f=20`） | 空 |
| `vbr` | `on` / `constrained` / `off` | `on` |
| `compression_level` | 0〜10 | 10 |
| `frame_duration` | ms | 20.0 |

> **`needs_processing()`**: `normalize` / `filter` / `channels > 0` が一つでも true だと再エンコード必須（無劣化コピー不可）。

## 既定プロファイル一覧（`config.default.toml`、2026-10-04）

| 順 | name | 判定 | backend | 特徴 | audio |
|---:|---|---|---|---|---|
| 1 | `youtube` | youtube.com / youtu.be / youtube-nocookie.com | auto | Opus(251) 優先、`--js-runtimes node` | 160k, copy_if_opus |
| 2 | `niconico` | nicovideo.jp / nico.ms | auto | HLS。プレミアム音質は cookies | 128k |
| 3 | `bilibili` | bilibili.com / b23.tv | auto | DASH 音声のみ。`referer` 付き | 128k |
| 4 | `twitch` | twitch.tv | auto | 長時間 ASMR 配信。`audio_only` 優先、`-N 8` | 128k |
| 5 | `soundcloud` | soundcloud.com | auto | ASMR 音声作品 | 128k, copy_if_opus |
| 6 | `x` | x.com / twitter.com | auto | 動画/スペース。必要なら cookies | 128k |
| 7 | `tiktok` | tiktok.com | auto | ショート ASMR | 96k |
| 8 | `hls-direct` | `.m3u8` / `.mpd` の URL | auto | ブラウザ開発者ツール等で取った直リンク。403 なら `referer`/`headers` | 128k, `-N 8` |
| 9 | `file-direct` | `.mp3/.m4a/.wav/.flac/.mp4/...` の URL | auto | ファイル URL を直接変換。`embed_metadata=false` | 128k, copy_if_opus |
| 10 | `generic` | どれにもマッチしなかった場合 | auto | フォールバック。**常に末尾** | 128k, copy_if_opus |

## 独自サイトの追加（新サイト対応）

- 追加手順は `.agent/skills/site-onboarding/SKILL.md`（調査 → ヘッダ/cookie 特定 → `[[profiles]]` 追加 → docs 同期 → 実機検証依頼）。
- 設定例は `../examples/profile-examples.toml`。
- ルール:
  - 既存プロファイルは変更しない（**追加のみ**）
  - `generic` は常に末尾
  - 推測のヘッダ値・cookies 値はコメントアウトで「要確認」に
  - 影響する docs（本ファイル / `examples/` / ルート README）を同期

## cookies.txt（Netscape 形式）

- プロファイルの `cookies` にパスを指定（`~` 展開可）。
- yt-dlp 経路では `--cookies <path>` で、ffmpeg 経路では `config::cookie_header` が**対象ホスト向けの Cookie ヘッダ**を生成して `-headers` で渡す。
- 書き出し方（Android）: Firefox / Kiwi 等の「cookies.txt を書き出す」系の拡張機能。
- **cookies.txt はユーザー個人ファイル（`~/cookies/...`）であり、リポジトリに含めない**（`security.md`）。例は `../examples/cookies.example.txt`（ダミー値）。

## 関連

- [pipeline.md](./pipeline.md) — プロファイルがパイプラインでどう使われるか
- `../examples/profile-examples.toml` — 独自サイトのプロファイル例
- `../examples/cookies.example.txt` — cookies.txt の例
- `.agent/skills/site-onboarding/SKILL.md` — 新サイト対応の手順
