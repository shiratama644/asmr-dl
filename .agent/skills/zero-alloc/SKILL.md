---
name: zero-alloc
description: 200ms tick と行解析のホットパスで不要アロケーションを削減するスキル。スタックバッファ、OnceLock、String 再利用、逐次 String 生成の回避、unicode-width 幅計算。Use when optimizing the render/event loop, line parsing, or reviewing hot-path allocations.
---

# Zero-Alloc — ホットパスの不要アロケーションを削減するスキル

> 出典: TEMPLATE_REPO の `zero-alloc/SKILL.md`（JS GC / 60-120Hz シム前提）を **asmr-dl の TUI 200ms tick + 行解析向けに書き換え**
> 正本: `src/runner.rs`（`read_lines` / `run_proc`）、`src/main.rs`（イベントループ）、`src/ui.rs`、`docs/arch/engineering.md`

## なぜ重要か（asmr-dl での意味）

asmr-dl はリアルタイムシムではないが、**ホットパス**が明確にある:

- **イベントループ**: `main` のループは 200ms tick + 毎フレーム `terminal.draw`（`ui::draw`）。描画はジョブ数 × 行数分の文字列を組み立てる
- **行解析**: `read_lines` / `run_proc` の `on_line` は yt-dlp / ffmpeg の標準出力・標準エラーの**全行**を処理する（ダウンロード中は高速に流れる）
- **S24 Ultra の制約**: Android 端末（メモリ・CPU 限定）で長時間動かなければならない

GC は無い（Rust）ので「フレームドロップ」は起きにくい。ただし**不要なアロケーションはメモリ増加・キャッシュミス・長いダウンロードでの電池消耗**につながる。

## 既定のパターン（asmr-dl 既存、維持すること）

| # | 場所 | パターン | 効果 |
|---|---|---|---|
| Z1 | `runner::read_lines` | 8192 バイトの**スタックバッファ**（`[0u8; 8192]`）+ 512 バイトの**行バッファ**（`Vec<u8>` を再利用）で読み、`\n`/`\r` 毎に flush | 行ごとに大きな Vec を新規確保しない。バッファ再利用 |
| Z2 | `config::extract_urls` | `static RE: OnceLock<Regex>` で**1回コンパイル** | 呼び出し毎に regex をコンパイルしない |
| Z3 | `runner::run_proc` の `on_line` | 行文字列を `&str` として受け、`strip_prefix` / `splitn` でパース（中間配列は小） | 全行を格納しない（ストリーミング処理） |
| Z4 | `app::on_event` / `on_msg` | `try_recv` で溜まったイベントを**まとめて処理**してから描画 | 1イベント毎に描画しない（バッチ処理） |
| Z5 | `ui::draw` | `unicode-width` で幅を計算し、文字列組み立ては最小限に | 全角対応しつつ幅预算で描画 |
| Z6 | `job::push_log` | `VecDeque` + cap（`memory-leak` スキル R4） | 履歴が無制限に増えない |

## 違反パターンと修正（参考）

| 違反 | 例 | 修正 |
|---|---|---|
| 行ごとに大きな `Vec<String>` を新規確保 | `let mut lines: Vec<String> = Vec::new(); ... lines.push(...)` をループ内で毎回 new | スタックバッファ + 行バッファ再利用（Z1 の `read_lines` に倣う） |
| 呼び出し毎に regex コンパイル | 関数内で `Regex::new(...)` | `OnceLock<Regex>` / `lazy_static` 相当で1回（Z2） |
| 1イベント毎に全描画 | `for ev in events { handle(ev); draw(); }` | `try_recv` でバッチ処理してから1回 draw（Z4） |
| 進捗更新で全文字列を再構築 | 進捗行を毎回 `format!` で作り直す | 差分のみ更新、または `ratatui` の_BUF 再利用 |
| 幅計算に `to_string` + 逐次連結 | 行組み立てで `+=` を多用 | `write!` マクロ / `format_args!` で1回の確保 |

## 実装パターン

### スタックバッファ + 行バッファ再利用（Z1 に倣う）

```rust
let mut buf: Vec<u8> = Vec::with_capacity(512);
let mut chunk = [0u8; 8192];
loop {
    let n = r.read(&mut chunk).await?;
    if n == 0 { break; }
    for &b in &chunk[..n] {
        if b == b'\n' || b == b'\r' {
            if !buf.is_empty() {
                let s = String::from_utf8_lossy(&buf).trim_end().to_string();
                buf.clear();   // バッファ再利用
                // s を処理（send）
            }
        } else {
            buf.push(b);
        }
    }
}
```

### 1回コンパイルの regex（Z2 に倣う）

```rust
static RE: OnceLock<Regex> = OnceLock::new();
let re = RE.get_or_init(|| Regex::new(r"https?://[^\s<>'`「」『』（）、。　]+").unwrap());
```

### イベントバッチ処理（Z4 に倣う）

```rust
Some(ev) = erx.recv() => {
    app.on_event(ev);
    while let Ok(ev) = erx.try_recv() { app.on_event(ev); }
}
```

## 監査コマンド

```bash
# 行解析/ループ内で毎回 Vec を新規確保していないか
grep -n "Vec::new\|vec!\[" src/runner.rs src/ui.rs

# regex が毎回コンパイルされていないか
grep -n "Regex::new" src/*.rs
# OnceLock / lazy_static で1回コンパイルになっているか確認

# 1イベント毎に draw していないか（main のループ）
grep -n "draw\|try_recv" src/main.rs
```

## やってはいけないこと

- 可読性を犠牲にしてまで微視的なゼロアロケをしない（**ホットパスのみ**が対象。設定読み込み等の低速路径は可読性優先）
- `unsafe` でアロケーションを回避しない（正当性が高い場合のみ、理由コメント必須）
- バッファ再利用で**行の境界を壊さない**（`read_lines` の不変条件: 行 = 文字列として扱える。Z1 参照）
- `format!` の使いすぎを「全部 `write!` に」と機械的に置き換えない（用途を選ぶ）

## 関連

- `docs/arch/engineering.md` — 性能・ホットパスの設計
- `src/runner.rs::read_lines` — スタックバッファの既存実装
- `memory-leak/SKILL.md` — バッファの cap（リング）
