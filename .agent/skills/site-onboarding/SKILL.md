---
name: site-onboarding
description: 「サイト X のダウンロードが落ちる / 新サイトに対応したい」要求を、調査→ヘッダ/cookie/バックエンド特定→[[profiles]]追加→docs/examples更新→検証依頼(L3)まで一気通貫で実行する。TEMPLATE_REPO の deep-dive-setup を asmr-dl のプロファイル追加向けに転用。Use when a user reports a site fails to download, or asks to add support for a new site/domain.
---

# Site Onboarding — 新サイト対応（プロファイル追加）

> 出典: TEMPLATE_REPO の `deep-dive-setup/SKILL.md`（pnpm setup 主導）を、**asmr-dl のプロファイル追加**向けに転用
> 正本: `docs/arch/profiles.md`、`config.default.toml`、`docs/examples/profile-examples.toml`、`AGENTS.md §6.4`

## 目的

「サイト X が落ちない」「サイト Y に対応して」のような要求を、ユーザーの試行錯誤を減らすために
**調査 → 原因特定 → プロファイル作成 → 設定反映 → docs 更新 → 検証依頼** まで一気通貫で進める。

## フローチャート（Mermaid）

```mermaid
flowchart TD
    Start([「サイトXが落ちない/新サイト」]) --> Analyze[要求分析<br/>URL/対象サイト/症状を確認]
    Analyze --> AskInfo{URL・症状が明確か？}
    AskInfo -->|不明| AskUser[ask_user<br/>URL/症状/cookies有無を確認]
    AskUser --> Analyze
    AskInfo -->|明確| Search
    subgraph Research[調査ループ]
        direction TB
        S1[web_search depth1<br/>「site download yt-dlp / 403」] --> Q1{深掘りが必要？}
        Q1 -->|Yes| S2[web_search depth2]
        Q1 -->|No| Fetch
        S2 --> Fetch[fetch_page<br/>公式/extractor issue]
        Fetch --> Synth[原因を整理<br/>ヘッダ/cookie/backend/JS]
    end
    Search --> Synth
    Synth --> Present[理解と方針を提示<br/>backend/headers/cookies/subdir]
    Present --> Confirm{ask_user<br/>この方針でOK？}
    Confirm -->|No| Revise[修正点を反映] --> Present
    Confirm -->|Yes| Implement

    subgraph ImplementPhase[実装]
        direction TB
        I1[config.default.toml に [[profiles]] 追加<br/>generic は常に末尾] --> I2[docs/examples/profile-examples.toml 追記]
        I2 --> I3[docs/arch/profiles.md / docs/README.md 更新]
        I3 --> I4[docs/task-list.md にタスク登録]
    end

    Implement --> Verify
    Verify{cargo が使える？}
    Verify -->|Yes| CTest[cargo test + cargo build --release]
    Verify -->|No| AltVerify[代替検証: tomllib + generic末尾 + リンク]
    CTest --> Report
    AltVerify --> Report[コミット + push + L3検証依頼<br/>device-testing マトリクス D10〜D12]
```

## Phase 1: 要求分析

### 1.1 必要な情報（`ask_user` で確認。2〜4問までに絞る）

| 項目 | 例 | 質問の必要性 |
|---|---|---|
| **対象 URL** | `https://example-asmr.jp/works/123` | 必須（ドメイン判定の根拠） |
| **症状** | `403` / `No supported JS runtime` / 無音 / 途中で切れる | 必須（原因特定に直結） |
| **cookies の有無** | 有（Firefox で書き出し済み）/ 無 | 有料・ログイン必須のサイトは必須 |
| **backend の希望** | auto / ytdlp / ffmpeg | 403 系は ffmpeg 直接 + referer が効くことがある |

### 1.2 現状確認

```bash
git status; git branch --show-current; git log -5 --oneline
# 既存プロファイルに同名/同ドメインが無いか
grep -n "domains" config.default.toml
```

## Phase 2: 調査（web_search / fetch_page）

- **yt-dlp の extractor 対応有無**を先に確認（`web_search depth1-2`: `yt-dlp <site> extractor / 403 / site:github.com/yt-dlp/yt-dlp/issues`）。
  - 対応している → 設定（headers / cookies / format）の問題として扱う。
  - 非対応 → `backend = "ffmpeg"` 直接経路（m3u8 / 直リンク）の可否を確認。
- **403 / 認証系の症状**では、必要なヘッダ（`Referer` / `Origin` / `User-Agent`）と cookies を特定する。
  - ブラウザの開発者ツールで確認した値が正。推測値で設定しない。
- **外部 URL は公式ソースを優先**（サイトの公式 docs / yt-dlp issue / ffmpeg 公式）。ミラーは参考程度。
- 調査結果は `docs/research/` に `{TOPIC}_RESEARCH.md` として残す（採用判断: 採用/不採用/要確認）。

## Phase 3: プロファイルの作成

### 3.1 作成ルール（`docs/arch/profiles.md` 準拠）

- `config.default.toml` の `[[profiles]]` に**追加**する（既存プロファイルは変更しない）。
- **`generic` は常に末尾**（`Config::normalize()` の前提。`AGENTS.md §6.4`）。
- 最低限書く項目:
  - `name`（kebab-case、一意）/ `description`（日本語）
  - `domains`（ホスト名サフィックス一致）または `url_regex`
  - `backend`（auto / ytdlp / ffmpeg）
  - `subdir`（出力先サブフォルダ）
  - `filename`（yt-dlp 出力テンプレート。`/` でサブフォルダ可）
  - 403 系は `referer` / `headers` / `cookies`
  - `[profiles.audio]`（`bitrate_kbps` / `copy_if_opus` 等。ASMR 向けは 128〜160k）
- **推測で埋めない**: 未確定のヘッダ値・cookies はコメントアウトで残し、`要確認` とする。

### 3.2 設定例

```toml
[[profiles]]
name = "my-asmr-site"
description = "Referer と Cookie が必要な ASMR 配信サイト"
domains = ["example-asmr.jp"]
backend = "auto"
referer = "https://example-asmr.jp/"
headers = ["Origin: https://example-asmr.jp"]
# cookies = "~/cookies/example-asmr.txt"   # ログイン必須なら
subdir = "MySite"
filename = "%(uploader).60B/%(title).150B.%(ext)s"
[profiles.audio]
bitrate_kbps = 160
copy_if_opus = true
```

## Phase 4: docs 更新（必須）

- `docs/examples/profile-examples.toml` に新プロファイル例を追記
- `docs/arch/profiles.md` のプロファイル一覧表に追加
- `docs/README.md` の目次（追加ファイルが varsa）
- `docs/task-list.md` にタスク登録（L3 未確認なら `実環境検証待ち`）

## Phase 5: 検証・報告

- cargo が使える環境: `cargo test` + `cargo build --release`（`config.default.toml` が `include_str!` で埋め込まれるため、変更はビルドで反映される）。
- Sandbox（§6.2）: 代替検証（tomllib パース + `generic` 末尾チェック + リンクチェック）のみ実施し、`cargo test` / `cargo build` を**残課題**として明記。
- **L3 実機検証依頼**: `device-testing/SKILL.md` のマトリクス（D10〜D12、該当する D 番号）を参照して、ユーザーに実機確認を依頼する。
- 報告は AGENTS.md §7.2 のフォーマット（変更内容の表 + 検証結果 + 残課題）。

## やってはいけないこと

- 既存プロファイルの既定値を黙って変更しない（影響が全ユーザーに及ぶ。AGENTS.md §6.4 / rules/01 §3）。
- 推測のヘッダ値・cookies 値を設定しない（要確認で残す）。
- L3（実機）未確認でタスクを `完了` にしない。
- `generic` の末尾位置を変更しない。
- 調査結果を `docs/research/` に残さず口頭だけで終わらせない。

## 関連

- `docs/arch/profiles.md` — プロファイル仕様の正本
- `docs/examples/profile-examples.toml` — プロファイル例
- `device-testing/SKILL.md` — L3 実機検証マトリクス
- `docs/research/README.md` — 調査の運用
