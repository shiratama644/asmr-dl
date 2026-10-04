---
name: determinism
description: 純粋関数・決定論ロジックの決定論を守るスキル。時刻/乱数/I/O 禁止、same-input テスト、既存の純粋関数（extract_urls / detect_profile / sanitize_filename / fmt_hms）。Use when writing pure functions, refactoring logic out of I/O paths, or adding tests for input/output mapping.
---

# Determinism — 決定論を守る実装スキル

> 出典: TEMPLATE_REPO の `determinism/SKILL.md`（ゲームシム前提）を **asmr-dl の純粋関数向けに書き換え**
> 正本: `src/config.rs` / `src/runner.rs` の純粋関数、`docs/arch/architecture.md`、`AGENTS.md §6.4`

## なぜ決定論が重要か（asmr-dl での意味）

asmr-dl はリアルタイム同期系ではないが、**「同じ入力 → 同じ出力」が要る箇所**が明確にある:

- **プロファイル判定**（`detect_profile` / `Profile::matches`）: 同じ URL で毎回同じプロファイルが選ばれること。ユーザーが「同じサイトなのに挙動が違う」と感じると困る
- **URL 抽出**（`extract_urls`）: 共有テキストから安定して同じ URL 列が得られること
- **ファイル名生成**（`sanitize_filename` / `name_from_url` / `fallback_name`）: 同じ情報から同じ名前になること（ただし `fallback_name` は時刻を使う = 意図的に非決定論、§3 参照）
- **progress パース**（`clean_na` / `[PROG]` 行パース）: 同じ yt-dlp 出力行から同じ進捗構造が得られること

これらが非決定論的になると、**回帰テストが書けなくなる**（同じ入力で出力が変わる = テスト不能）。

## 禁止事項（純粋関数内）

純粋関数（I/O なし・テスト対象）内では以下を使わない:

- **時刻**: `std::time::{SystemTime, Instant, UNIX_EPOCH}` / `libc::localtime_r`（`now_secs` / `local_timestamp` は `runner` の非純粋層で使う）
- **乱数**: 本プロジェクトでは乱数自体を使わない（必要になったら seed 注入で決定論的に）
- **I/O**: `std::fs` / `tokio::fs` / `std::process` / `tokio::process` / 環境変数直接参照（`std::env::var` は `config` の一部除く — 後述）
- **状態の読み書き**: 静的可变状態を渡さない（`OnceLock` は**初期化のみ**なら可、値はコンパイル時固定の regex）

### 例外として許容するもの

- `config::home_dir()` / `expand_tilde` / `Config::config_path()` は**環境変数（`HOME` / `XDG_CONFIG_HOME` / `ASMR_DL_CONFIG`）を読む**が、これは「設定の解決」であり、同じ環境なら同じ出力。テストでは環境変数を固定すれば決定論的になる。
- `OnceLock<Regex>`（`extract_urls`）は**1回コンパイルして固定**されるので決定論的（同じ入力で同じマッチ）。

## 実装パターン

### 1. I/O とロジックを分離する（最重要）

I/O 関数の内側にロジックが埋もれている時は、**ロジックを純粋関数に切り出す**。

```rust
// ❌ I/O とロジックが混在（テスト困難）
async fn parse(yt_output: &str) -> Result<Progress> {
    // ファイルから読む + パースが混ざっている
}

// ✅ ロジックを純粋関数に分離（テスト可能）
fn parse_progress_line(line: &str) -> Option<Progress> {
    let rest = line.strip_prefix("[PROG]")?;
    let f: Vec<&str> = rest.splitn(6, '|').collect();
    // ... percent / speed / eta をパース（clean_na で NA 吸収）
    Some(Progress { .. })
}
// I/O 側は parse_progress_line を呼ぶだけ
```

### 2. Same-input テスト

同じ入力で同じ出力になることを検証する（回帰テストとして効く）:

```rust
#[test]
fn detection() {
    let mut cfg: Config = toml::from_str(DEFAULT_CONFIG).unwrap();
    cfg.normalize();
    let name = |u: &str| cfg.profiles[cfg.detect_profile(u)].name.clone();
    assert_eq!(name("https://m.youtube.com/watch?v=x"), "youtube");
    assert_eq!(name("https://www.nicovideo.jp/watch/sm9"), "niconico");
    // 同一入力を複数回呼んでも同じ判定になる（純粋）
    assert_eq!(name("https://youtu.be/abc"), name("https://youtu.be/abc"));
}
```

### 3. 時刻・乱数を「意図的に非決定論」に使う時は明示する

`fallback_name`（`ホスト_YYYYmmdd-HHMMSS`）は**時刻を使うこと自体が仕様**（ユニークなファイル名生成）。
こういう箇所は:

- 関数名・doc comment で「時刻を使う」ことを明示する
- テストでは**時刻部分を除外して検証**する（`name_from_url(...).starts_with("cdn.a.com_20")` のように先頭のみ assert）
- 決定論が必要な箇所には時刻を渡さない（参数として注入し、テストで固定値を渡す）

## 監査コマンド

```bash
# 純粋層（config / job）に I/O が混入していないか
grep -n "std::fs\|tokio::fs\|std::process\|tokio::process\|SystemTime\|UNIX_EPOCH" src/config.rs src/job.rs || echo "OK: pure layer has no I/O"

# 純粋層に時刻 API が混入していないか（config/job は home_dir の env 参照のみ可）
grep -n "localtime_r\|Instant::now\|SystemTime" src/config.rs src/job.rs || echo "OK: no clock in pure layer"
```

## やってはいけないこと

- 純粋関数に `SystemTime::now()` / `Instant::now()` を直接書かない
- `OnceLock` で**実行時に変化しうる値**を固定しない（regex のような固定値のみ）
- 時刻・乱数が必要な仕様でも、引数注入で決定論的にテストできる形にしない（`fallback_name` は例外として先頭検証で扱う）
- same-input テストを「1回だけ呼ぶ」テストと混同しない（同一入力の再現性を確認するのが目的）

## 関連

- `testing/SKILL.md` — 純粋関数のテスト
- `docs/arch/architecture.md` — 純粋層 / I/O 層の分け方
- `src/config.rs` / `src/runner.rs` — 既存の純粋関数とテスト
