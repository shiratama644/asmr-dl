---
name: docs-maintenance
description: ドキュメント整理とURL検証のスキル。内部リンク整合性、外部URL有効性、ミラー排除、AGENTS.md読込順序の遵守、config.default.toml と docs の同期を確実にする。
---

# Docs Maintenance — ドキュメント整理とURL検証スキル

> 出典: TEMPLATE_REPO の `docs-maintenance/SKILL.md` を asmr-dl 向けに書き換え
> 正本: `AGENTS.md`（§6.7）・`docs/README.md` 索引・`.agent/logs/` 参照

## 読込順序（AGENTS.md最優先）

1. `AGENTS.md` → 2. `.agent/` recursively → 3. `README.md` → 4. `docs/` recursively 全文、部分読み禁止
2. `docs/README.md` 索引に無いファイルは孤児・混入・残骸を疑う
3. 正本は `docs/`（仕様: `arch/`, 計画: `planning/`, 調査: `research/`, 監査: `audit/`, 運用: `ops/`, 設定例: `examples/` + 進捗: `task-list.md`）+ `.agent/`（Agent設定）

## 内部リンク整合性チェック

コードスパン除外してMarkdownリンクを抽出し、相対パス解決で存在確認:

```python
# コードスパン除外してMarkdownリンク抽出
import re, pathlib
for md in pathlib.Path('docs').rglob('*.md'):
    text = md.read_text(errors='ignore')
    text = re.sub(r"```.*?```", '', text, flags=re.DOTALL)  # fenced code除外
    text = re.sub(r'`[^`]*`', '', text)  # inline code除外
    for m in re.finditer(r'\[.*?\]\(#?([^)]+)\)', text):
        url = m.group(1)
        if url.startswith('http'): continue
        if url.startswith('#'): continue
        target = (md.parent / url.split('#')[0]).resolve()
        if not target.exists():
            print(f"BROKEN {md}: {url}")
```

- 階層変更時（例: `docs/arch/` → `docs/`）は `../` がずれる。`git mv` + 中身修正を同時に行う
- ルート `README.md` / `AGENTS.md` からの `docs/...` / `.agent/...` リンクも対象にする

## 外部URL検証ルール（asmr-dl で使う公式ソース）

| カテゴリ | 正本URL | 対応 |
|---|---|---|
| ffmpeg / ffprobe | https://ffmpeg.org/documentation.html / https://ffmpeg.org/ffmpeg.html | 公式pathを使う |
| yt-dlp | https://github.com/yt-dlp/yt-dlp （README / extractor list） | 公式 |
| ratatui | https://ratatui.rs / https://docs.rs/ratatui | 公式 |
| tokio | https://tokio.rs / https://docs.rs/tokio | 公式 |
| Rust | https://doc.rust-lang.org / https://doc.rust-lang.org/cargo | 公式 |
| Termux | https://termux.com / https://github.com/termux | 公式 |
| Claude Code | https://code.claude.com/docs/en/ | 公式 |

### 検証方法

- `fetch_page` で 200 確認、anchor まで含めて取得できるか確認
- ミラー・非公式ドキュメントは primary usage から排除し、参考リンク程度に留める
- 公式ドキュメントのパス変更（マイグレーション）は定期的に見直す

## `config.default.toml` と docs の同期（asmr-dl 固有）

- `config.default.toml` の `[[profiles]]` を変更したら、必ず同期する:
  - `docs/arch/profiles.md` のプロファイル一覧表
  - `docs/examples/profile-examples.toml`（独自サイト例が古くなっていないか）
  - `README.md` の「サイト別プロファイル」記述
- 同期漏れを検出:
  ```bash
  python3 -c "import tomllib; print([p['name'] for p in tomllib.load(open('config.default.toml','rb')).get('profiles',[])])"
  grep -n "profile" docs/arch/profiles.md
  ```

## docs更新時の5点出力（AGENTS.md準拠）

1. 変更ファイル一覧
2. 内部リンク検証結果
3. 外部URL検証結果
4. `config.default.toml` との同期確認（プロファイル名・既定値）
5. 次のTODO

## 運用ルール

- `docs/README.md` を索引として維持、新規ドキュメント追加時は必ず索引にも追記
- 設計の正本は `docs/arch/`（どう作るか）、進捗の正本は `docs/task-list.md`、計画は `docs/planning/`、調査は `docs/research/`、監査は `docs/audit/`、運用は `docs/ops/`、設定例は `docs/examples/`
- 実装テクニックは `.agent/skills/`、作業規約は `AGENTS.md` / `.agent/rules/` を正とする
- 旧名称が残っていないか `grep -R "旧名称" docs/ --include="*.md"` で確認
- ファイル名は短く正確、ハイフン最大1つ

## 関連

- `docs/README.md` 索引
- `docs/task-list.md` 進捗正本
- `docs/arch/README.md` 仕様書一覧
- `AGENTS.md` 読込順序・§6.7
- `.agent/skills/verify-doc-integrity/SKILL.md` 機械検証スキル
