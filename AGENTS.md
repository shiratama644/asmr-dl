# AGENTS.md

本ドキュメントは、AI Agent が本プロジェクト（asmr-dl）の開発・変更を行う際に**必ず遵守すべき開発規約**です。
最優先事項は **「速く大量に作ること」ではなく「常に復旧可能で、壊れた状態を長時間維持しないこと」** です。

> 📌 本リポジトリは `shiratama644/TEMPLATE_REPO`（テンプレートリポジトリ）を元にして整備されています。
> §1〜§5・§7・§8 は汎用規約のためそのまま適用し、**§6 に asmr-dl 固有の遵守事項**を記載しています。

---

## 1. 基本方針 & 作業単位

### 1.1 基本原則
- **小さく実装 → 検証 → 修正 → Git Commit → 次の機能** のサイクルを徹底する。
- 一度に大量の機能を実装して最後にまとめてデバッグする方式は禁止。
- 「ついでに改善できそう」という理由でスコープを広げない（未指定の機能追加・設計変更・大規模リファクタリングの禁止）。

### 1.2 作業単位の粒度
1タスクは**「1つの意味のある論理的単位」**で区切る。

| 区分 | 例 |
| :--- | :--- |
| **良い例（適切な粒度）** | サイト別プロファイルの追加 / 進捗パーサの修正 / キャンセル処理の強化 / 設定項目の追加 |
| **悪い例（細かすぎる）** | 文字列リテラル1個変更ごとにコミット |
| **悪い例（大きすぎる）** | UI + 変換パイプライン + 新サイト対応 を1タスクで一括実装 |

※大規模変更（新経路の追加、依存の入れ替えなど）は、「設計 → 基盤 → 機能A（検証・commit） → 機能B（検証・commit）」と段階的に分割すること。

---

## 2. 開発ワークフロー

各タスクは必ず以下の順序で進め、途中の検証が失敗した状態で次へ進んではならない。

```text
1. 仕様・既存コード確認 (git status / Cargo.toml / docs/arch/ 関連ファイル)
   ↓
2. 実装方針決定
   ↓
3. 実装 (最小限の差分)
   ↓
4. プロジェクト検証 (fmt / clippy / test / build — 実行可能な範囲、§6.2 参照)
   ↓ 失敗時は原因特定して修正し、再度全検証
5. 差分確認 (git diff で意図しない変更がないか確認)
   ↓
6. Git Commit (Conventional Commits形式)
   ↓
7. タスク完了・停止 (勝手に次のタスクを開始しない)
```

---

## 3. テスト・品質保証ルール

### 3.1 検証コマンドの実行
- 存在しないコマンドを捏造・実行しない。本プロジェクトの検証コマンドは以下（§6.2 の Sandbox 制約により**実行できない環境では §6.2 の代替検証に置き換え**、コミット報告にその旨を明記する）：
  ```bash
  cargo fmt --all -- --check          # フォーマット確認 (rustfmt)
  cargo clippy --all-targets -- -D warnings   # Lint (警告をエラー扱い)
  cargo test                          # 単体テスト（#[cfg(test)]、watchモード相当なし）
  cargo build --release               # 本番ビルド (opt-level 3, LTO thin, strip)
  ```
- テストの watch モード（`cargo test --watch` 等）は commit 前検証に使わない。
- 実機（Termux）での動作確認が要る変更は「実環境検証待ち」としてタスクを分離し、Sandbox で無理に実行しようとしない（§6.2）。
- **テスト配置規則**: テストはテスト対象の隣（同一 `.rs` 文件内に `#[cfg(test)] mod tests`）に配置する。本プロジェクトには `_tests_/` ディレクトリは存在しない（Rust の慣例に従う）。
- **pre-commit / commit-msg**: 本リポジトリには husky 等の Git フックは導入されていない。コミットメッセージの Conventional Commits 準拠は Agent の責任（§4.2）。

### 3.2 エラー対応と品質維持
- エラー発生時はエラーメッセージやスタックトレースから根本原因を特定し、最小限の範囲で修正する。
- **テストを通すためだけの不正な修正は厳禁**：
  - テストの削除・スキップ（`#[ignore]`）・アサーションの緩和
  - 型エラーを回避するための安易な `unwrap()` / `expect()` / `unsafe` 使用
  - Lint ルールの勝手な無効化（`#[allow]`）・エラーの握りつぶし（`let _ =` で失敗を捨てる）
- **既存仕様の尊重**：既存テストが落ちた場合、「テストが間違っている」と即断せず、既存仕様を壊していないか確認する。

### 3.3 既存バグの扱い
- **今回のタスクを妨げるバグ**：必要最小限の修正を行う。
- **無関係な既存バグ**：勝手に修正せず、ユーザーに報告する。
- バグ修正時は、可能であれば再発防止の回帰テスト（Regression Test）を追加する。

---

## 4. Git運用 & 環境復旧ルール

Gitは単なる履歴管理ではなく、**「実行環境消失・セッション切断時の復元チェックポイント」**として扱う。

### 4.1 作業開始時の現状把握
作業開始時は必ず以下を実行し、ブランチ・未コミット変更・直近ログを確認する。
```bash
git status
git branch --show-current
git log -5 --oneline
```
※未コミットの変更が存在する場合、勝手に破棄・上書きせず、現在の作業に混ぜない。

#### 4.1.1 サンドボックス再構築時の復旧手順
サンドボックス型の作業環境（Arena 等）は再構築されることがあり、その場合ワークツリーには
「起点コミットのファイル」＋「push 済みコミットで追加されたファイルの未追跡バージョン」が
混在した状態で立ち上がる（`git status` が「大量の削除 + 大量の未追跡」を示す）。
この時点でファイルは破損していないので、以下の手順で確実に復旧すること。

```bash
# 1. リモートの最新を fetch（ブランチ名は git branch --show-current で確認した現在値を使う）
git fetch origin <現在のブランチ>

# 2. FETCH_HEAD にワークツリーごとリセット（この場合の --hard は例外的に必要）
git reset --hard FETCH_HEAD

# 3. 依存を再構築（Rust: cargo fetch。ツールチェーンが無ければ §6.2 の代替検証で進む）
cargo fetch
```

- `git reset --hard FETCH_HEAD` は §4.3 の厳禁ルールの例外で、**サンドボックス再構築後の初回のみ**許可される（未コミット変更は元々存在しない状態のため）。
- 再構築を判定するヒント：`git log --oneline` が起点コミット 1 個しか返ってこない / `git status` が大量の削除を示す / `target/` が無い。
- 復旧後は必ず `git log --oneline -5` と、実行可能な検証（§6.2）で健全性を確認してから作業を再開する。

### 4.2 コミットルール
- **タイミング**: 検証（fmt/clippy/test/build、実行可能な範囲）がすべてPASSした状態でのみコミットする。
- **事前チェック**: `git status` および `git diff` を確認し、意図しないファイルが含まれていないことを確認する。
- **重要な変更前のチェックポイント**: 大規模リファクタリング、設定スキーマ変更、依存関係更新の前には、作業前の正常状態を一度コミット（checkpoint）しておく。
- **コミットメッセージ**: Conventional Commits 形式に従う。
  - `feat:`, `fix:`, `refactor:`, `perf:`, `test:`, `docs:`, `chore:`, `build:`, `ci:`
  - 日本語で書く（例: `feat: bilibili の並列フラグメント数を増やす`）

### 4.3 厳禁なGit操作（明示的な指示がない限り実行禁止）
以下の破壊的・履歴改変コマンドは**絶対に実行してはならない**。
- `git reset --hard` / `git clean -fd`（未コミット作業の消失リスク）
  - ただし §4.1.1 のサンドボックス再構築復旧時の `git reset --hard FETCH_HEAD` のみ例外
- `git rebase` / `git commit --amend`（既存履歴の改変）
- `git push --force` / `git push --force-with-lease`

#### 4.3.1 通常の `git push` の事前許可（ユーザーとの恒久合意）
- サンドボックスは予告なく再構築され、**ローカルコミットのみだと作業が破棄される**。
  成果物を保護するため、検証がすべて PASS し意図しない差分がないことを確認できたら、
  その場で `git push origin <セッション固定ブランチ>` を実行する（push のたびにユーザー確認を取らない）。
- push 先は**セッション固定ブランチのみ**（§4.4）。`main` 等への直接 push、
  他ブランチへの push、force push は引き続き禁止。
- PR の作成も許可済み（`gh pr create`）。作成後は URL を報告する。

### 4.4 ブランチ運用（セッション型環境）
- **作業ブランチはセッション固定**。Arena 等はこのブランチ名でセッションを追跡しており、他ブランチに push した作業は**セッションと紐付かず失われる**。
  - ブランチ名は**セッションごとに変わる**ため、**必ず `git branch --show-current` で確認**すること（pre-task フックの最初の手順）。
  - **過去セッションのブランチ名を文書に残さない**。古いブランチ名を文書へ残すと、後続セッションが別セッションのブランチを fetch/push する事故になる。
- ユーザーから「別ブランチを使ってほしい」と依頼された場合も、セッション固定ブランチから離れる前に「このセッションは `<現在のブランチ>` に固定です」と説明し、そのまま作業を続ける。
- feature branch は切らない。セッション固定ブランチへ直接 commit + push し、`gh pr create` で `main` 向け PR を作成する。マージ判断はユーザー側に委ねる。
- push は `git push origin <現在のブランチ>` の明示指定で行う。default remote/branch 依存の `git push` は避ける。

---

## 5. タスク完了条件（AI Agentの停止条件）

以下の条件が**すべて満たされた時点で作業を完了とし、停止（回答）**する。追加の改善を勝手に開始してはならない。

- [ ] 指定された機能/修正が実装されている
- [ ] すべての検証（fmt, clippy, test, build — Sandbox で実行可能な範囲）がPASSしている
- [ ] タスクと無関係なファイルの変更・意図しない差分がない
- [ ] 適切なメッセージで Git Commit が完了している
- [ ] Working tree が clean である（`git status` で確認）
- [ ] `git push origin <セッション固定ブランチ>` が完了している（§4.4）

---

## 6. プロジェクト固有の遵守事項（asmr-dl）

> この節は**asmr-dl 固有**のルール。技術スタック・制約・実装方針を集約する。
> 計画書と矛盾する指定があった場合は計画書を優先するが、それ以外は本節を厳守する。

### 6.1 環境・ツールチェーン
- **言語**: Rust（`edition = "2021"`、stable チャンネル推奨）。パッケージマネージャは **Cargo**（`package.json` / Node / pnpm は本プロジェクトに存在しない）。
- **主要依存（最小構成を維持）**: ratatui 0.30（crossterm を再エクスポート）、tokio 1、serde、toml、anyhow、regex、url、unicode-width、libc。**新規クレート追加はユーザーの明示的な合意が必要**。
- **外部コマンド（実行時依存、コードには含めない）**: `yt-dlp`（pip。YouTube 用は `yt-dlp-ejs` + `nodejs`）、`ffmpeg` / `ffprobe`（`libopus` エンコーダ必須）。パスは `config.toml` で指定可。
- **ターゲット環境**: **Termux (Android)**（設計基準: Galaxy S24 Ultra / One UI）。デスクトップ Linux での動作も壊さないこと。
- **ビルド**: `cargo build --release`（`opt-level=3, lto=thin, codegen-units=4, strip=true`）。S24 Ultra で数分かかることを前提に設計する（メモリ・CPU 消費の重いツール導入は避ける）。

### 6.2 サンドボックス制約（乗り越えず、迂回する）
Arena Sandbox では以下の制約が**恒常的**（2026-10-04 実測）。修正対象ではない。

| 制約 | 影響 | 対処 |
|---|---|---|
| `sh.rustup.rs` / `static.rust-lang.org` / `crates.io` 到達不可（SSL_ERROR_SYSCALL） | Rust ツールチェーンのインストール不可 → `cargo build` / `cargo test` / `cargo clippy` / `cargo fmt` が実行できない | 構文レベルの代替検証を行う: ① `python3 -c "import tomllib; tomllib.load(open('config.default.toml','rb'))"` ② `sh -n scripts/*.sh`（POSIX sh 構文）③ Markdown 内部リンクの実在チェック（`.agent/skills/verify-doc-integrity/` の手順）。最終検証は CI（未導入時の場合はユーザー環境）で実施し、コミット報告に「build/test 未実行（§6.2）」を明記する |
| apt 不可（root 権限なし） | パッケージでの rustc 入手不可 | 上と同様 |
| `termux-*` ビナリ（wake-lock / notification / media-scan / open / setup-storage）が存在しない | Termux 連携の動作確認不可 | 実機検証タスクとして分離（`docs/task-list.md` に「実環境検証待ち」で記録）。コードは「bin が無い場合は無音でスキップ（spawn 失敗を握りつぶさないこと）」という既存方針を維持 |
| TUI（ratatui）を headless 実行できない | 起動確認・キー操作の確認不可 | `--help` / `--print-config` もビルド後にのみ確認可能。Sandbox では実行しない |
| GitHub（`gh` / git over https）は到達可 | — | fetch/push/PR は通常通り実行可 |

### 6.3 リポジトリ固有の Git 制約
- `target/` は `.gitignore` 済み。ビルド成果物を絶対にコミットしない。
- ルートの `asmr-dl.zip` は初期ソース配布物（アーティファクト）。**編集・削除しない**（再配布用に保持）。
- `config.default.toml` は **バイナリに `include_str!` で埋め込まれる**（`src/config.rs`）。変更したら必ず `cargo build` での反映を前提とし、`Config::normalize()`（`src/config.rs`）の挙動（generic が末尾に移動等）と矛盾しないこと。

### 6.4 実装ルール（Rust / モジュール境界）
- **モジュールの依存は下位へのみ**（上位が下位を参照してよい。逆参照は禁止）:
  ```
  config / job   — 純粋データ・設定（I/O なし、テスト可能）
       ↑
  runner         — yt-dlp / ffprobe / ffmpeg の実行・解析・保存（I/O 層）
       ↑
  app            — アプリ状態・キュー制御・入力イベント（I/O なし、状態のみ）
       ↑
  ui             — 描画のみ（`HitAreas` を更新する副作用を含む）
       ↑
  main           — 起動・端末初期化・イベントループの配線
  ```
- **エラー処理は anyhow**。`?` + `.context()` / `.with_context()` で文脈を付与。キャンセルは `CanceledError`（`src/runner.rs`）で区別し、失敗通知に含めない。
- **外部プロセスは必ず** `kill_on_drop(true)` + （unix）`process_group(0)` で起動し、キャンセル時はプロセスグループごと `SIGKILL` する（`run_proc` の既存パターンを踏襲。yt-dlp が起動した ffmpeg を放置しない）。
- **UI のホットパス**（200ms tick + 描画 + 行解析）での不要なアロケーションは避ける（`.agent/skills/zero-alloc/` 参照）。
- **純粋関数は必ず `#[cfg(test)]` でテストする**（`extract_urls` / `sanitize_filename` / `detect_profile` / `fmt_hms` 等。既存テストの拡張を基本とする）。
- コメント・UI 文案・ログ・コミットメッセージは**日本語**。コード識別子は英語。
- `unwrap()` / `expect()` は「構造的に不可能」な箇所のみ許容。その理由をコメントで示す（`DEFAULT_CONFIG` パース等）。
- 新規プロファイル追加は `config.default.toml`（既定）と必要なら `docs/examples/` に追記。**`generic` は常に最後のプロファイル**（`normalize()` が保証。順序に依存する実装を書くな）。

### 6.5 整形・Lint
- `cargo fmt`（rustfmt 既定設定。`rustfmt.toml` は無い。作成するならユーザー確認）を commit 前に通す。
- `cargo clippy --all-targets -- -D warnings` で警告ゼロを目指す。`#[allow]` は理由をコメントで必ず書く。

### 6.6 UI 実装ルール
- **縦画面（40〜60 桁）でも崩れない**ことを最優先。`ui::draw` で 1 行の文字数を実測ではなく「幅预算で計算」（`unicode-width` を使う）。
- 描画時にクリック判定用の `HitAreas`（入力欄 / ジョブラスト / ログ）を記録し、`on_mouse` で使う。描画と判定の領域を二重管理しない。
- 状態アイコンは Unicode（`… ⇣ ✔ ✖ ■`）。色は Android の端末カラーで見やすい明度を保つ。
- ターミナルの初期化・復元は `ratatui::init()` / `ratatui::restore()` に任せる。追加の raw mode 操作はしない（panic 時フックを壊さない）。

### 6.7 ドキュメント運用
- 構成・命名規則・運用ルール: `docs/README.md`
- タスク進捗管理: `docs/task-list.md`（**唯一の正本**。状態定義・証拠記録のルールは同ファイル冒頭）
- 仕様書: `docs/arch/`（どう作るか）— product / architecture / pipeline / profiles / tech-stack / termux / quality / security / engineering / cicd / adr / milestones
- 計画書: `docs/planning/`（`_TEMPLATE.md` 形式）— 完了済みは `planning/complete/`
- 調査: `docs/research/`（競合・技術調査）
- 監査: `docs/audit/`（差分・バグ。時点記録で書き換え禁止）
- 運用: `docs/ops/`（Termux インストール・Galaxy 設定・yt-dlp 更新）
- 設定例: `docs/examples/`（プロファイル例・cookies.txt 例）
- ドキュメントを追加・削除・移動したら `docs/README.md` の目次を必ず更新する
- ファイル名は短く正確、ハイフン最大1つ（例: `tech-stack.md` OK、`rust-tech-stack-and-toolchain.md` NG）

---

## 7. コミュニケーション規約（Agent の話し方・ユーザーとの対話方針）

本節は「Agent がユーザーとどう会話するか」の型を定める。過去のセッションで受け入れられた話し方を型化しており、次セッションの Agent もこの型を踏襲すること。

### 7.1 返答の基本スタイル
- **言語**: 日本語（ユーザーが日本語で話しかけているため）。技術用語は日本語 + 英語併記可（例: 「プロファイル自動判定」「process group kill」）。
- **文体**: 敬体（です・ます調）をベース。技術説明部分は淡々と事実を述べる。過度な謙譲・冗長な前置きは避ける。
- **絵文字**: 通常会話では使わない。**結果報告・チェックリスト・優先度表示のみ**、最小限で使う。
  - `✅` (完了) / `❌` (失敗) / `🟡` (中優先度) / `🟢` (低優先度) / `🔴` (高優先度・要注意) / `🎉` (フェーズ完了時のみ)
- **見出し**: `##` `###` `####` で構造化。3 段以上は避ける（読みにくくなる）。
- **表**: 実測値・比較・状態一覧は必ず表 (`| 項目 | 値 |`) にまとめる。散文で羅列しない。
- **箇条書き**: `-` を優先。番号付き `1.` は手順・実行順序を示す時のみ。

### 7.2 報告のフォーマット

コミット・タスク完了時は以下の順序で報告する:

1. **見出し**: `## ✅ <タスク名> 完了 (`abc1234`)` のようにタスク名 + commit hash 短縮 7 桁
2. **変更内容の表**: `| # | 問題/目的 | 実装 |` 形式
3. **ファイル変更数**: `新規/変更ファイル (N files, +X / -Y)`
4. **検証結果チェックリスト**:
   ```text
   - ✅ cargo test: N passed（実行可能な場合のみ。Sandbox では §6.2 の代替検証を明記）
   - ✅ config.default.toml: tomllib parse OK
   - ✅ push 済み (`prev..head`)
   ```
   ※ 検証項目はプロジェクトの実情に合わせる（§6.2 の制約を正直に報告する）。
5. **次のアクション**: 「次は何をしますか?」「Go を出していただければ〜」と提示、勝手に次のタスクを開始しない。

### 7.3 事実と推測の分離
- 実測値・確認済み事実は断言する（「HTTP 200 でした」「ビルド 3 分 12 秒でした」）。
- 未検証・推測は明示する（「〜のはずです」「〜と想定」「〜見込み」）。
- 実行環境で計測不能な数値は「実機で計測予定」等と明記し、確定値のように書かない。

### 7.4 ユーザーへの質問方針
**わからないこと・判断に迷うことは、勝手に決めず必ずユーザーに質問する**。

#### 7.4.1 質問すべき場面
- 実装方針が 2 通り以上あり、どちらもメリット・デメリットがある時
- 仕様が確定しておらず、判断が必要な時
- ユーザーの過去発言と現在の指示が矛盾している疑いがある時
- 破壊的変更（設定スキーマ変更、依存関係大規模更新、プロファイル既定値の変更等）を含む時
- 「〜してください」の指示が曖昧で、複数解釈が成り立つ時（例: 「音質を良くして」→ ビットレート？チャンネル数？normalize？）

#### 7.4.2 質問の方法
- **`ask_user` ツール**を使う（自由文で質問文を投げるのではなく、選択肢 UI で提示）
- **選択肢は 2〜4 個 + 自由記述** に絞る。5 個以上は認知負荷が高くなり選ばれない
- 各選択肢には **短いラベル (`label`)** と **詳しい説明 (`description`)** を書く（description で判断材料を提供）
- 質問文は 1 文で明確に。前置きは最小限
- **一度に 4 質問まで**。それ以上は認知負荷過多

#### 7.4.3 質問の悪例と良例

❌ 悪い例（勝手に決める）:

> 「サイト X が落ちない」
> → Agent が独断で「では cookies 必須にして Referer も足そう」と決めて設定を大幅変更

✅ 良い例（`ask_user` で確認）:

> サイト X の取得を安定させるため、以下を確認します:
> 1. 認証: cookies 不要 / 任意 / 必須
> 2. 取得経路: backend=auto / ytdlp のみ / ffmpeg のみ
> 3. 適用範囲: 新規プロファイル追加 / generic の修正のみ

### 7.5 Web 検索の活用方針
**わからないこと・記憶に自信がないことは Web 検索で確認する**。トレーニングデータ (cutoff) 以降の情報や、ライブラリの最新 API 仕様は特に検索必須。

#### 7.5.1 検索すべき場面
- **ライブラリの API 仕様が変わっている可能性がある時**（ratatui / tokio / yt-dlp のオプション等）
- **サイトの取得方法が変わっている可能性がある時**（yt-dlp に対応していない新サイト、403 化、JS チャレンジ等）
- **エラーメッセージが記憶にない、または解決策が不明な時**（ffmpeg のコーデックエラー、yt-dlp の extractor エラー）
- **セキュリティ・法規制関連**（著作権・利用規約・ライセンス互換性等、間違えると影響が大きい）
- ユーザーが「最新の〜」「今の〜」と時期を明示している時

#### 7.5.2 検索の使い方
- `web_search` ツールを使う。`depth` は状況で使い分け:
  - **depth=1**: 事実確認・URL 確認レベル
  - **depth=2**: 標準（複数ソース比較したい時）
  - **depth=3**: 深掘り（詳細仕様・長い記事から抜粋が必要な時）
- 検索結果を引用する時は `[id](url)` 形式で必ずソースを明示
- 公式ドキュメント（ffmpeg / yt-dlp / ratatui の公式 docs）を優先
- **記憶で断言せず、疑わしければ検索する**。ハルシネーションを避けるための必須アクション

#### 7.5.3 検索と質問の使い分け
- **技術的事実の確認** → `web_search`（客観情報）
- **プロジェクト固有の仕様判断** → `ask_user`（ユーザー主観）
- **例**: 「yt-dlp の `--js-runtimes` の現在の推奨値は?」→ 検索 / 「このサイトは cookies 必須にしていいですか?」→ ユーザー質問

### 7.6 失敗・エラー時の対応スタイル
検証失敗・実装エラーが発生した時は、以下の 3 段で説明する:

1. **原因分析**: 「〜が原因です」（推測なら「〜と思われます」を明示）
2. **修正方針**: 「〜で対処します」（2 案以上あるならユーザーに選択させる）
3. **実装**: 実際の修正コード

原因を隠して修正だけ通知しない。ユーザーが同じ地雷を踏まないよう、原因もセットで共有する。

### 7.7 制約・リスクの事前明示
実装前に **実行環境の制約・権限制約・外部サービス制約** 等が関係する場合は必ず先に伝える。

例:

> この変更は ffmpeg のフィルタチェーンを触りますが、**この Sandbox には Rust ツールチェーンが無いため §6.2 の制約により `cargo test` を実行できません**。構文レベルの検証（tomllib / リンクチェック）のみ実施し、テストは実機 or CI での確認が必要です。

事後報告（「実は動作確認できてませんでした」）は信頼を損なうので避ける。

---

## 8. エージェント記憶システム（`.agent/`）

本プロジェクトでは、Agent 自身の**コードベース知識・定型ワークフロー・タスク実行ログ**を `.agent/` 配下に構造化して永続化する。セッションをまたいで記憶を継承し、無駄な再調査を省くための仕組み。

> **構成はClaude Code公式の `.claude/` ディレクトリ構成に準拠**。ディレクトリ名は `.agent/` のまま維持する。
> 公式仕様: https://code.claude.com/docs/en/claude-directory.md

### 8.1 ディレクトリ構成（公式準拠 + テンプレート拡張）

```
.agent/
├── settings.json              # チーム共有設定（permissions, hooks, env）— コミット対象
├── settings.local.json        # 個人オーバーライド（gitignore）— 個人のみ
├── rules/                     # トピック別ルール（pathsで発火条件を絞れる）— 公式準拠
│   ├── 01_information-hierarchy.md
│   ├── 02_git-workflow.md
│   ├── 03_doc-style.md
│   ├── 04_verification.md
│   └── project-template.md    # asmr-dl 固有の遵守事項（AGENTS.md §6 と対応）
├── skills/<name>/SKILL.md     # コードベース知識（/nameで呼び出し、自動発火も可能）— 公式準拠
│   ├── project-overview/      # 製品概要・構成把握
│   ├── tech-stack/            # Rust/Cargo/外部ツール・ハマりどころ
│   ├── sandbox-constraints/   # Sandbox制約・迂回策（2026-10-04実測）
│   ├── verify-doc-integrity/  # ドキュメント整合性検証
│   ├── diff-review-report/    # 差分レビューレポート
│   ├── docs-maintenance/      # ドキュメント整理・URL検証
│   ├── ci-quality-gates/      # 品質ゲート・CI（現状: ローカルゲートのみ、CIは未導入）
│   ├── testing/               # Rust 単体テスト・意味あるアサーション
│   ├── import-boundaries/     # モジュール境界・依存方向
│   ├── determinism/           # 純粋関数・決定論・same-inputテスト
│   ├── memory-leak/           # プロセス・チャネル・タスクのリソース解放
│   ├── zero-alloc/            # ホットパスのアロケーション削減
│   ├── device-testing/        # 実機(Termux)でのTUI検証・テストマトリクス
│   └── site-onboarding/       # 新サイト対応（プロファイル追加）の一気通貫手順
├── agents/                    # サブエージェント定義（name, description, tools, model等）— 公式準拠
│   ├── explore.md
│   ├── plan.md
│   ├── doc-editor.md
│   ├── code-reviewer.md
│   └── test-writer.md
├── hooks/                     # フック手順(.md) + 実行スクリプト(.sh) — テンプレート拡張 + 公式hooks登録
│   ├── index.md
│   ├── pre-task.md
│   ├── verify-commit.md
│   ├── log-task.md
│   ├── sandbox-recovery.md
│   ├── restore-env.sh
│   ├── pre_edit_guard.sh      # PreToolUse: 編集禁止領域ブロック
│   └── post_edit_verify.sh    # PostToolUse: 事後検証
├── commands/                  # 旧commands互換（新しくはskills/を使う）— 公式互換
│   ├── commit.md
│   ├── review.md
│   └── test.md
├── output-styles/             # 出力スタイル（concise, detailed等）— 公式準拠
├── workflows/                 # 動的ワークフロー（複数サブエージェントを束ねる）— 公式準拠
│   └── implement-task.js
├── agent-memory/              # サブエージェント永続メモリ（自動生成、gitignore）— 公式準拠
└── logs/                      # タスク実行ログ（追加のみ、書き換え禁止）— テンプレート独自
    └── YYYY-MM-DD_<summary>.md
```

| ディレクトリ | 役割 | 命名規則 | 公式/独自 |
| :--- | :--- | :--- | :--- |
| `settings.json` | チーム共有設定（permissions, hooks, env） | 固定 | 公式準拠 |
| `settings.local.json` | 個人オーバーライド（gitignore） | 固定 | 公式準拠 |
| `rules/` | トピック別ルール（pathsで発火条件） | `NN_kebab-case.md` | 公式準拠 |
| `skills/<name>/` | コードベースの**事実/仕様/暗黙了解**をサブシステム別に格納 | `<kebab-case>/SKILL.md` | 公式準拠 |
| `agents/` | サブエージェント定義（name, description, tools等） | `kebab-case.md` | 公式準拠 |
| `hooks/` | トリガー別の**定型手順/スクリプト** + 実行スクリプト | `kebab-case.md` / `.sh` | 公式hooks登録 + テンプレート拡張 |
| `commands/` | 単一ファイルプロンプト（旧形式、互換性のため） | `kebab-case.md` | 公式互換 |
| `output-styles/` | 出力スタイルのカスタマイズ | `kebab-case.md` | 公式準拠 |
| `workflows/` | 動的ワークフロー（JS） | `kebab-case.js` | 公式準拠 |
| `agent-memory/` | サブエージェント永続メモリ | `<name>/MEMORY.md` | 公式準拠 |
| `logs/` | タスク完了毎の**実行記録** | `YYYY-MM-DD_kebab-case-summary.md` | テンプレート独自 |

各ディレクトリ直下に **`index.md`** を置き、一覧・参照条件を管理する（`logs/` は日付名でソートされるため不要、`rules/` `skills/` `hooks/` は必須）。

### 8.2 公式構成との対応

| 公式 `.claude/` | 本リポジトリ `.agent/` | 備考 |
|---|---|---|
| `CLAUDE.md` | `AGENTS.md`（本ファイル） | Claude CodeはAGENTS.mdも読める。CLAUDE.mdがあれば併用も可 |
| `settings.json` | `settings.json` | 同型。permissions, hooks, env, model等 |
| `settings.local.json` | `settings.local.json` | 同型。gitignore対象 |
| `rules/*.md` | `rules/*.md` | 同型。pathsフロントマターで発火条件 |
| `skills/<name>/SKILL.md` | `skills/<name>/SKILL.md` | 同型。name, description必須 |
| `commands/*.md` | `commands/*.md` | 同型。旧形式だが互換性のため残す |
| `agents/*.md` | `agents/*.md` | 同型。name, description, tools, model等 |
| `output-styles/*.md` | `output-styles/*.md` | 同型 |
| `workflows/*.js` | `workflows/*.js` | 同型。agent(), parallel(), phase()プリミティブ |
| `agent-memory/` | `agent-memory/` | 同型。サブエージェント永続メモリ（gitignore） |
| — | `hooks/` | テンプレート拡張。手順md + 実行sh + settings.jsonで登録 |
| — | `logs/` | テンプレート拡張。4セクションログ |

### 8.3 `index.md` 起点のピンポイント読込（核心ワークフロー）
- **タスク開始時**（[`.agent/hooks/pre-task.md`](.agent/hooks/pre-task.md)）: 現状把握後、[`.agent/skills/index.md`](.agent/skills/index.md) の「読み方ガイド」で**該当スキルだけ**を読む。全スキルを常に読み込まない（コンテキスト浪費）。`settings.json` の `UserPromptSubmit` フックでも自動実行。
- **編集前**: `settings.json` の `PreToolUse` フックで [`pre_edit_guard.sh`](.agent/hooks/pre_edit_guard.sh) が編集禁止領域をブロック。
- **編集後**: `settings.json` の `PostToolUse` フックで [`post_edit_verify.sh`](.agent/hooks/post_edit_verify.sh) が事後検証。
- **トリガー発生時**: [`.agent/hooks/index.md`](.agent/hooks/index.md) の「対応表」で該当フックを特定し実行。
- 初回/全体把握が必要な時だけ、`project-overview/SKILL.md` を読む。

### 8.4 記憶の同期（書き込みワークフロー）
- **タスク完了時**（[`.agent/hooks/log-task.md`](.agent/hooks/log-task.md)）: 必ず `.agent/logs/YYYY-MM-DD_<summary>.md` を 4 セクション（指示内容/実行内容/気づき/次アクション）で作成。
- **知見のスキル化**: ログの「気づき」が再利用性の高いコードベース知識なら該当 `skills/<name>/SKILL.md` に反映し、`skills/index.md` の「最終更新」を更新する。新スキルは `skills/index.md` の「読み方ガイド」「一覧」両方に追記。frontmatterの `description` も更新。
- **ルールの更新**: 汎用的な作業規約が見つかったら `rules/` に追加。`paths` で発火条件を絞る。
- ログ・スキル・rules・index の変更も commit/push 対象（セッションブランチへ）。

### 8.5 AGENTS.md と skills/rules の役割分担
- **AGENTS.md（本ファイル）** = 「どう作業するか」の**規約**（コミット手順・Lint・Git運用・コミュニケーション等）。常に正。
- **rules/** = 「いつ・どのファイルで何を守るか」の**トピック別ルール**。`paths` で発火条件を絞る。AGENTS.mdの詳細版。
- **skills/** = 「このコードベースが**どう出来ているか**」の**事実/仕様**。深掘り用。自動発火も可能。
- 3者が重複する場合、作業手順は AGENTS.md、詳細ルールは rules/、ドメイン知識は skills/ を参照。

### 8.6 運用ルール
- `.agent/` 配下は Git 追跡対象（永続化）。ただし `settings.local.json` と `agent-memory/` は `.gitignore` で除外（個人情報・自動生成のため）。
- スキル/ルール/フック/エージェントを更新したら対応 `index.md` / `README.md` も必ず更新する（腐らせない）。
- ログは**追加のみ**（過去ログを書き換えない）。
  - ⚠️ **一括置換・リネーム系の指示が来ても、`.agent/logs/` の過去ログを置換対象に含めない。**
    過去ログは「その時点で何が起きたか」の事実記録であり、旧ブランチ名・旧数値・旧パスが
    書かれているのは**正しい状態**。書き換えると記録が偽になる。
  - 一括置換の射程は**現用ドキュメント**（`AGENTS.md` / `.agent/skills/` / `.agent/hooks/` / `.agent/rules/` / `.agent/agents/` / 現用の `docs/`）に限定する。`.agent/logs/` 等の時点記録に触れる必要がある場合は、**必ず事前にユーザーへ確認**する。
  - 当時の事実（旧ブランチ名など）を残す必要がある場合は、過去ログを書き換えるのではなく**当日の新規ログに記録**する。
- **出典（テンプレート整備時の採用元）**:
  - **TEMPLATE_REPO（shiratama644）**: 本 `.agent/` の基本構成。rules/トピック分割、skills/モジュール化、hooks/の POSIX sh + exit code 規約、4セクションログ
  - **PalmIDE**: rules/、agents/、hooks/pre_edit_guard.sh + post_edit_verify.sh、verify-doc-integrity / diff-review-report スキル
  - **cod-web (arena 各セッション)**: skills の frontmatter 形式、testing / determinism / import-boundaries / memory-leak / zero-alloc / docs-maintenance / ci-quality-gates の着想（Rust 向けに書き換え済み）
  - **ytdl**: 検証キットのUXルール・sandbox-constraints の詳細化
