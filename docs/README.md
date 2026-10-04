# ドキュメント索引

本リポジトリ（asmr-dl）のドキュメント一式を種類別に整理した目次です。ルート `README.md` からプロジェクトの概要・セットアップ・使い方へアクセスできます。

> **構成の出典**: `shiratama644/TEMPLATE_REPO`（cod-web / DropMod / ytdl / PalmIDE 統合の活発リポジトリ構成）を asmr-dl に適用。
> `arch/`（仕様）+ `planning/`（計画）+ `research/`（調査）+ `audit/`（監査）+ `ops/`（運用）+ `examples/`（設定例）+ `task-list.md`（進捗の正本）。
> 2026-10-04 に asmr-dl 向けに全ファイルを書き換えた（TEMPLATE_REPO 由来の bootstrap / detector / cache 等の仕様書は本プロジェクトに該当しないため削除）。

---

## 📂 ディレクトリ構成

```
docs/
├── README.md                          ← 本ファイル（全ドキュメントの目次）
├── task-list.md                       ★ タスク管理の唯一の正本（進捗・証拠）
├── arch/                              ★ 仕様書（どう作るか）
│   ├── README.md                      # 仕様書一覧と実装時に守ること
│   ├── product.md                     # プロダクト定義・用語・現行資産
│   ├── architecture.md                # モジュール構造・依存規則・メッセージフロー
│   ├── pipeline.md                    # ダウンロード/変換パイプライン（yt-dlp 経路 / ffmpeg 経路）
│   ├── profiles.md                    # サイト別プロファイルと自動判定
│   ├── tech-stack.md                  # Rust + 外部ツール（yt-dlp/ffmpeg）+ Termux
│   ├── termux.md                      # Termux 連携・端末制約（wake-lock/通知/共有メニュー）
│   ├── quality.md                     # 品質ゲート（fmt/clippy/test/build）・テスト方針
│   ├── security.md                    # セキュリティ・プライバシー（cookies・ファイル名・DRM）
│   ├── engineering.md                 # エンジニアリング規約（エラー処理・プロセス管理・ホットパス）
│   ├── cicd.md                        # CI/CD（現状: 未導入、将来構成）
│   ├── adr.md                         # 意思決定ログ
│   └── milestones.md                  # 現行フェーズと次の候補
├── planning/                          # 計画書（タスク単位・_TEMPLATE.md形式）
│   ├── README.md                      # 計画書索引・次に使う計画
│   ├── index.md                       # フォルダ説明
│   ├── _TEMPLATE.md                   # 計画書テンプレート（新規は必ず本形式）
│   └── complete/                      # 完了済み計画
│       └── README.md
├── research/                          # 調査結果
│   └── README.md                      # 調査一覧・採用判断
├── audit/                             # 差分・バグ監査（時点記録）
│   ├── index.md                       # 本フォルダの説明
│   └── activity.md                    # 活動ログ（2026-10-04 テンプレート適用の記録）
├── ops/                               # 運用ドキュメント（実機運用の実務）
│   ├── index.md
│   ├── INSTALL_TERMUX.md              # Termux インストール手順
│   ├── TERMUX_BATTERY.md              # Galaxy/One UI のバッテリー設定
│   └── UPDATING_YTDLP.md              # yt-dlp 更新手順
└── examples/                          # 設定例
    ├── profile-examples.toml          # [[profiles]] 定義例（独自サイト用）
    └── cookies.example.txt            # Netscape 形式 cookies.txt の例
```

> 各フォルダの `index.md` / `README.md` に「何のフォルダか・そこに何を置くか・運用ルール」が書かれています。

---

## 📖 読み方ガイド

| 状況 | 最初に読むもの | 次に読むもの |
|---|---|---|
| プロジェクトの全体把握（初回） | ルート `README.md` | `arch/product.md` → `arch/architecture.md` |
| 実装を始める | `task-list.md` → `planning/README.md` | 対象タスクの計画書 → 該当 `arch/*.md` |
| ダウンロード/変換パイプラインを変更 | `arch/pipeline.md` | `src/runner.rs` / `adr.md` |
| 新サイト（プロファイル）を追加 | `arch/profiles.md` | `examples/profile-examples.toml` / `.agent/skills/site-onboarding/` |
| Termux 連携を変更 | `arch/termux.md` | `ops/INSTALL_TERMUX.md` / `ops/TERMUX_BATTERY.md` |
| 品質・テストの方針を確認 | `arch/quality.md` | `.agent/skills/testing/` / `.agent/skills/ci-quality-gates/` |
| 運用（インストール・更新） | `ops/index.md` | 該当 `ops/*.md` |
| 過去の変更・判断の経緯を追う | `audit/activity.md` / `audit/diff-*.md` | `planning/complete/*` |

## 運用ルール（要約）

1. **進捗・タスク状態の正本は `task-list.md`**。矛盾時は本ファイルを正とする
2. **仕様は `arch/` が正本**。計画は `planning/`、調査は `research/`、監査は `audit/`、運用は `ops/`、設定例は `examples/`
3. **ドキュメントを追加・削除・移動したら本ファイルの目次を必ず更新**
4. **`config.default.toml` を変更したら** `arch/profiles.md` / `examples/` / ルート `README.md` を同期
5. ファイル名は短く正確、ハイフン最大1つ
6. `audit/` と `.agent/logs/` の過去記録は**時点記録**のため書き換えない
