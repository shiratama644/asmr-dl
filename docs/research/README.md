# docs/research — 調査結果

ここは競合・関連技術の調査結果を置く場所です。仕様へ採用する場合は `../arch/` へ反映してから実装します。

## 構成

```
research/
├── README.md          ← 本ファイル
└── <TOPIC>_RESEARCH.md        # 個別調査（例: YOUTUBE_JS_RESEARCH.md / SITE_X_RESEARCH.md）
```

## 運用ルール

- 調査は `{TOPIC}_RESEARCH.md` 形式で、**目的・調査対象・結果・採用判断（採用/不採用/要確認）** を書く
- 外部URLは公式ソースを優先（ffmpeg.org / yt-dlp GitHub / ratatui / tokio / Rust / Termux）、ミラーは排除
- 調査結果を `../arch/` へ反映する際は、該当 `../arch/*.md` を更新し、本ディレクトリの調査と相互リンク
- 過去の調査は書き換えず、新調査で追記
- **新サイト対応（403 / 認証 / JS チャレンジ）の調査は、`.agent/skills/site-onboarding/SKILL.md` の手順で進め、本ディレクトリに記録する**

## 調査済み（asmr-dl）

| 調査 | 結果 | 採用先 |
|---|---|---|
| TEMPLATE_REPO 構成の asmr-dl への適用（2026-10-04） | .agent/ + docs/ + AGENTS.md + README.md を Rust/Termux 向けに書き換え可能 | 本リポジトリ |
| Sandbox 制約の実測（2026-10-04） | Rust 系 egress ブロック / apt 不可 / python3+jq 可 / GitHub 可 | `.agent/skills/sandbox-constraints/` / AGENTS.md §6.2 |

## 次の調査候補

- yt-dlp の YouTube JS チャレンジ対応（`--js-runtimes node` + `yt-dlp-ejs`）の安定性
- 403 になる ASMR サイトのヘッダ/cookies の特定（`site-onboarding` と併用）
- ratatui 0.30 の描画パフォーマンス（縦画面・長時間表示）
- ffmpeg の Opus エンコーダ設定（`-application audio` / VBR / frame_duration）の最適化
