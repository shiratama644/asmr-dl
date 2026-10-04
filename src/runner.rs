//! ダウンロード & Opus 変換パイプライン
//!
//!  [yt-dlp 経路]  yt-dlp で一時フォルダへ取得 → 中の全メディアを ffmpeg で .opus 化 → 出力先へ移動
//!  [ffmpeg 経路]  URL(m3u8/直リンク) を ffmpeg に直接渡して 1 パスで .opus 化
//!
//!  元が Opus で加工指定が無ければ再エンコードせずコピー (無劣化)。

use crate::config::{expand_tilde, host_of, Backend, Config, Profile};
use crate::job::{JobEvent, JobMsg};
use anyhow::{anyhow, bail, Context, Result};
use std::path::{Path, PathBuf};
use std::process::{ExitStatus, Stdio};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::io::{AsyncRead, AsyncReadExt};
use tokio::process::Command;
use tokio::sync::{mpsc, watch};

pub struct RunCtx {
    pub id: u64,
    pub url: String,
    pub profile: Profile,
    pub bitrate: u32,
    pub cfg: Arc<Config>,
    pub tx: mpsc::UnboundedSender<crate::AppMsg>,
    pub cancel: watch::Receiver<bool>,
}

impl RunCtx {
    fn send(&self, ev: JobEvent) {
        let _ = self.tx.send(crate::AppMsg::Job(JobMsg { id: self.id, ev }));
    }
    fn log(&self, s: impl Into<String>) {
        self.send(JobEvent::Log(s.into()));
    }
    fn stage(&self, s: impl Into<String>) {
        self.send(JobEvent::Stage(s.into()));
    }
    fn progress(&self, pct: Option<f64>, speed: impl Into<String>, eta: impl Into<String>) {
        self.send(JobEvent::Progress {
            pct,
            speed: speed.into(),
            eta: eta.into(),
        });
    }
    fn canceled(&self) -> bool {
        *self.cancel.borrow()
    }
}

#[derive(Debug)]
struct CanceledError;
impl std::fmt::Display for CanceledError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "キャンセルされました")
    }
}
impl std::error::Error for CanceledError {}

/// ジョブのエントリポイント
pub async fn run(ctx: RunCtx) {
    let work = expand_tilde(&ctx.cfg.temp_dir).join(format!("job-{}-{}", ctx.id, now_secs()));
    let res = run_inner(&ctx, &work).await;
    if !ctx.cfg.keep_temp {
        let _ = tokio::fs::remove_dir_all(&work).await;
    }
    match res {
        Ok(files) => {
            post_actions(&ctx, &files);
            ctx.send(JobEvent::Done(files));
        }
        Err(_) if ctx.canceled() => ctx.send(JobEvent::Canceled),
        Err(e) if e.downcast_ref::<CanceledError>().is_some() => ctx.send(JobEvent::Canceled),
        Err(e) => {
            let msg = format!("{e:#}");
            notify(&ctx, "ASMR DL: 失敗", &msg);
            ctx.send(JobEvent::Failed(msg));
        }
    }
}

async fn run_inner(ctx: &RunCtx, work: &Path) -> Result<Vec<PathBuf>> {
    tokio::fs::create_dir_all(work)
        .await
        .with_context(|| format!("一時フォルダを作成できません: {}", work.display()))?;
    let out_root = {
        let mut p = ctx.cfg.output_root();
        if !ctx.profile.subdir.is_empty() {
            p = p.join(&ctx.profile.subdir);
        }
        p
    };
    ctx.log(format!("プロファイル: {} / backend: {}", ctx.profile.name, ctx.profile.backend.label()));
    ctx.log(format!("出力先: {}", out_root.display()));

    match ctx.profile.backend {
        Backend::Ffmpeg => ffmpeg_direct(ctx, work, &out_root).await,
        Backend::Ytdlp => via_ytdlp(ctx, work, &out_root).await,
        Backend::Auto => match via_ytdlp(ctx, work, &out_root).await {
            Ok(v) => Ok(v),
            Err(e) if ctx.canceled() => Err(e),
            Err(e) if ctx.cfg.fallback_to_ffmpeg => {
                ctx.log(format!("yt-dlp 失敗: {e:#}"));
                ctx.log("→ ffmpeg で直接取得を試します");
                // 失敗した残骸を掃除
                let _ = tokio::fs::remove_dir_all(work).await;
                tokio::fs::create_dir_all(work).await?;
                ffmpeg_direct(ctx, work, &out_root)
                    .await
                    .map_err(|e2| anyhow!("{e2:#}\n(yt-dlp: {e:#})"))
            }
            Err(e) => Err(e),
        },
    }
}

// ───────────────────────── yt-dlp 経路 ─────────────────────────

async fn via_ytdlp(ctx: &RunCtx, work: &Path, out_root: &Path) -> Result<Vec<PathBuf>> {
    ctx.stage("解析中");
    ctx.progress(None, "", "");
    let p = &ctx.profile;
    let mut cmd = Command::new(&ctx.cfg.ytdlp_path);
    cmd.arg("--newline")
        .arg("--progress")
        .args(["--color", "never"])
        .args([
            "--progress-template",
            "download:[PROG]%(progress._percent_str)s|%(progress._speed_str)s|%(progress._eta_str)s|%(info.playlist_index)s|%(info.n_entries)s|%(info.title)s",
        ])
        .arg("-P")
        .arg(work)
        .args(["-o", &p.filename])
        .arg("--windows-filenames") // Android 共有ストレージで使えない文字を避ける
        .arg("--no-mtime")
        .args(["--retries", "10", "--fragment-retries", "20"])
        .arg("--no-write-playlist-metafiles")
        .arg(if p.playlist { "--yes-playlist" } else { "--no-playlist" });
    if !p.format.is_empty() {
        cmd.args(["-f", &p.format]);
    }
    if !p.format_sort.is_empty() {
        cmd.args(["-S", &p.format_sort]);
    }
    if p.concurrent_fragments > 1 {
        cmd.args(["-N", &p.concurrent_fragments.to_string()]);
    }
    if p.embed_metadata {
        cmd.arg("--embed-metadata");
    }
    if p.write_thumbnail {
        cmd.args(["--write-thumbnail", "--convert-thumbnails", "jpg"]);
    }
    if p.split_chapters {
        cmd.arg("--split-chapters")
            .args(["-o", &format!("chapter:_chapters/{}", p.chapter_filename)]);
    }
    if !p.referer.is_empty() {
        cmd.args(["--add-header", &format!("Referer:{}", p.referer)]);
    }
    if !p.user_agent.is_empty() {
        cmd.args(["--add-header", &format!("User-Agent:{}", p.user_agent)]);
    }
    for h in &p.headers {
        if let Some((k, v)) = h.split_once(':') {
            cmd.args(["--add-header", &format!("{}:{}", k.trim(), v.trim())]);
        }
    }
    if !p.cookies.is_empty() {
        cmd.arg("--cookies").arg(expand_tilde(&p.cookies));
    }
    cmd.args(&p.ytdlp_args);
    cmd.arg("--").arg(&ctx.url);

    let mut last_err: Option<String> = None;
    let mut title_sent = false;
    let status = run_proc(ctx, cmd, "yt-dlp", |_, line| {
        if let Some(rest) = line.strip_prefix("[PROG]") {
            let f: Vec<&str> = rest.splitn(6, '|').collect();
            let pct = f
                .first()
                .and_then(|s| s.trim().trim_end_matches('%').trim().parse::<f64>().ok());
            let speed = f.get(1).map(|s| clean_na(s)).unwrap_or_default();
            let eta = f.get(2).map(|s| clean_na(s)).unwrap_or_default();
            let idx = f.get(3).map(|s| clean_na(s)).unwrap_or_default();
            let n = f.get(4).map(|s| clean_na(s)).unwrap_or_default();
            if !idx.is_empty() && !n.is_empty() {
                ctx.stage(format!("DL {idx}/{n}"));
            } else {
                ctx.stage("DL");
            }
            if let Some(t) = f.get(5) {
                let t = clean_na(t);
                if !t.is_empty() && !title_sent {
                    ctx.send(JobEvent::Title(t));
                    title_sent = true;
                }
            }
            ctx.progress(pct, speed, eta);
            return;
        }
        if line.starts_with("ERROR:") {
            last_err = Some(line.trim_start_matches("ERROR:").trim().to_string());
        }
        if line.starts_with("[Merger]") || line.starts_with("[Metadata]") || line.starts_with("[ThumbnailsConvertor]") {
            ctx.stage("後処理");
        }
        if line.starts_with("[SplitChapters]") {
            ctx.stage("チャプター分割");
        }
        ctx.log(line.to_string());
    })
    .await?;

    let files = collect_media(work);
    if !status.success() {
        let msg = last_err.unwrap_or_else(|| format!("yt-dlp が異常終了 ({status})"));
        if files.is_empty() {
            bail!(msg);
        }
        ctx.log(format!("⚠ 一部失敗: {msg} (取得済み {} 件は変換します)", files.len()));
    }
    if files.is_empty() {
        bail!("ダウンロードされたメディアファイルが見つかりません");
    }

    // チャプター分割した場合は分割後のファイルだけを使う
    let chapter_dir = work.join("_chapters");
    let chapters: Vec<PathBuf> = files.iter().filter(|f| f.starts_with(&chapter_dir)).cloned().collect();
    let (targets, base) = if !chapters.is_empty() {
        (chapters, chapter_dir.clone())
    } else {
        (files, work.to_path_buf())
    };

    let total = targets.len();
    let mut outputs = Vec::new();
    for (i, src) in targets.iter().enumerate() {
        if ctx.canceled() {
            return Err(CanceledError.into());
        }
        let label = if total > 1 { format!("変換 {}/{}", i + 1, total) } else { "変換".to_string() };
        let tmp = work.join(format!("__out_{i}.opus"));
        let info = convert(ctx, &src.to_string_lossy(), false, &tmp, &label).await?;

        let rel = src.strip_prefix(&base).unwrap_or(src);
        let mut rel = rel.with_extension("opus");
        // "index" "video" 等の意味のない名前 → 埋め込みタイトル → 「ホスト_日時」の順で置き換え
        if let Some(stem) = rel.file_stem().map(|s| s.to_string_lossy().to_string()) {
            if is_generic_name(&stem) {
                let name = match info.title.as_deref().map(str::trim) {
                    Some(t) if !t.is_empty() && !is_generic_name(t) => {
                        if total == 1 {
                            ctx.send(JobEvent::Title(t.to_string()));
                        }
                        sanitize_filename(t)
                    }
                    _ => {
                        let n = fallback_name(&ctx.url, i, total);
                        if total == 1 {
                            ctx.send(JobEvent::Title(n.clone()));
                        }
                        n
                    }
                };
                rel.set_file_name(format!("{name}.opus"));
            }
        }
        ctx.stage("保存中");
        // 同名ファイルとの衝突は保存直前に判定 (並列ジョブ対策)
        let dest = unique_path(&out_root.join(&rel));
        move_file(&tmp, &dest).await?;
        ctx.log(format!("保存: {}", dest.display()));
        // サムネイルを横に置く (cover.jpg 的に使えるプレイヤーが多い)
        if ctx.profile.write_thumbnail {
            let thumb = src.with_extension("jpg");
            if thumb.exists() {
                let _ = tokio::fs::copy(&thumb, dest.with_extension("jpg")).await;
            }
        }
        outputs.push(dest);
    }
    Ok(outputs)
}

// ───────────────────────── ffmpeg 直接経路 ─────────────────────────

async fn ffmpeg_direct(ctx: &RunCtx, work: &Path, out_root: &Path) -> Result<Vec<PathBuf>> {
    ctx.stage("解析中");
    let info = probe(ctx, &ctx.url, true).await?;
    let name = info
        .title
        .clone()
        .filter(|t| !t.trim().is_empty())
        .unwrap_or_else(|| name_from_url(&ctx.url));
    ctx.send(JobEvent::Title(name.clone()));
    let tmp = work.join("__out_direct.opus");
    convert_with_info(ctx, &ctx.url, true, &tmp, "DL+変換", info).await?;
    ctx.stage("保存中");
    let dest = unique_path(&out_root.join(format!("{}.opus", sanitize_filename(&name))));
    move_file(&tmp, &dest).await?;
    ctx.log(format!("保存: {}", dest.display()));
    Ok(vec![dest])
}

// ───────────────────────── ffprobe / ffmpeg ─────────────────────────

#[derive(Debug, Default, Clone)]
struct ProbeInfo {
    codec: Option<String>,
    duration: Option<f64>,
    title: Option<String>,
    has_audio: bool,
}

fn ffmpeg_http_args(ctx: &RunCtx, url: &str) -> Vec<String> {
    let mut v = Vec::new();
    let headers = ctx.profile.http_headers(url);
    let mut extra = String::new();
    for (k, val) in headers {
        if k.eq_ignore_ascii_case("user-agent") {
            v.push("-user_agent".into());
            v.push(val);
        } else {
            extra.push_str(&format!("{k}: {val}\r\n"));
        }
    }
    if !extra.is_empty() {
        v.push("-headers".into());
        v.push(extra);
    }
    v
}

async fn probe(ctx: &RunCtx, input: &str, is_url: bool) -> Result<ProbeInfo> {
    let mut cmd = Command::new(&ctx.cfg.ffprobe_path);
    cmd.args(["-v", "error"]);
    if is_url {
        cmd.args(ffmpeg_http_args(ctx, input));
        cmd.args(["-rw_timeout", "30000000"]);
    }
    cmd.args([
        "-select_streams",
        "a:0",
        "-show_entries",
        "stream=codec_name,codec_type:format=duration:format_tags=title",
        "-of",
        "default=nw=1",
    ])
    .arg(input);

    let mut info = ProbeInfo::default();
    let mut errs = Vec::new();
    let status = run_proc(ctx, cmd, "ffprobe", |is_err, line| {
        if is_err {
            errs.push(line.to_string());
            return;
        }
        if let Some((k, v)) = line.split_once('=') {
            match k {
                "codec_name" => info.codec = Some(v.trim().to_lowercase()),
                "codec_type" if v.trim() == "audio" => info.has_audio = true,
                "duration" => info.duration = v.trim().parse::<f64>().ok().filter(|d| *d > 0.0),
                k if k.eq_ignore_ascii_case("TAG:title") => info.title = Some(v.trim().to_string()),
                _ => {}
            }
        }
    })
    .await?;
    if !status.success() {
        bail!("ffprobe 失敗: {}", errs.last().cloned().unwrap_or_else(|| status.to_string()));
    }
    for e in errs {
        ctx.log(format!("ffprobe: {e}"));
    }
    Ok(info)
}

async fn convert(ctx: &RunCtx, input: &str, is_url: bool, out: &Path, label: &str) -> Result<ProbeInfo> {
    ctx.stage(format!("{label} (解析)"));
    let info = probe(ctx, input, is_url).await?;
    convert_with_info(ctx, input, is_url, out, label, info.clone()).await?;
    Ok(info)
}

async fn convert_with_info(
    ctx: &RunCtx,
    input: &str,
    is_url: bool,
    out: &Path,
    label: &str,
    info: ProbeInfo,
) -> Result<()> {
    if !info.has_audio {
        bail!("音声トラックがありません: {}", short(input));
    }
    let a = &ctx.profile.audio;
    let copy = a.copy_if_opus
        && !a.needs_processing()
        && info.codec.as_deref() == Some("opus");

    ctx.log(format!(
        "{label}: 入力 codec={} 長さ={} → {}",
        info.codec.as_deref().unwrap_or("?"),
        info.duration.map(fmt_hms).unwrap_or_else(|| "?".into()),
        if copy { "Opusのままコピー(無劣化)".to_string() } else { format!("libopus {}kbps", ctx.bitrate) }
    ));
    ctx.stage(if copy { format!("{label} (copy)") } else { label.to_string() });
    ctx.progress(Some(0.0), "", "");

    let mut cmd = Command::new(&ctx.cfg.ffmpeg_path);
    cmd.args(["-hide_banner", "-nostdin", "-y", "-loglevel", "warning"]);
    if is_url {
        cmd.args(ffmpeg_http_args(ctx, input));
        cmd.args(["-rw_timeout", "30000000"]);
        let lower = input.to_lowercase();
        if !lower.contains(".m3u8") && !lower.contains(".mpd") {
            cmd.args(["-reconnect", "1", "-reconnect_streamed", "1", "-reconnect_delay_max", "10"]);
        }
    }
    cmd.arg("-i").arg(input);
    cmd.args(["-map", "0:a:0", "-vn", "-sn", "-dn", "-map_metadata", "0"]);
    if copy {
        cmd.args(["-c:a", "copy"]);
    } else {
        cmd.args([
            "-c:a",
            "libopus",
            "-b:a",
            &format!("{}k", ctx.bitrate),
            "-vbr",
            &a.vbr,
            "-compression_level",
            &a.compression_level.min(10).to_string(),
            "-application",
            "audio",
            "-frame_duration",
            &format!("{}", a.frame_duration),
            "-ar",
            "48000",
        ]);
        if a.channels > 0 {
            cmd.args(["-ac", &a.channels.to_string()]);
        }
        let mut filters = Vec::new();
        if a.normalize {
            // ASMR 向けに緩め (ダイナミクスを潰しすぎない)
            filters.push("loudnorm=I=-24:TP=-2:LRA=20".to_string());
        }
        if !a.filter.trim().is_empty() {
            filters.push(a.filter.trim().to_string());
        }
        if !filters.is_empty() {
            cmd.args(["-af", &filters.join(",")]);
        }
    }
    cmd.args(["-progress", "pipe:1", "-nostats", "-f", "opus"]).arg(out);

    let dur = info.duration;
    let mut speed_x: Option<f64> = None;
    let mut last_err: Option<String> = None;
    let status = run_proc(ctx, cmd, "ffmpeg", |is_err, line| {
        if is_err {
            last_err = Some(line.to_string());
            ctx.log(format!("ffmpeg: {line}"));
            return;
        }
        let Some((k, v)) = line.split_once('=') else { return };
        match k {
            "speed" => {
                speed_x = v.trim().trim_end_matches('x').parse::<f64>().ok();
            }
            "out_time_us" | "out_time_ms" => {
                if let Ok(us) = v.trim().parse::<f64>() {
                    let t = us / 1_000_000.0;
                    let pct = dur.map(|d| (t / d * 100.0).clamp(0.0, 100.0));
                    let eta = match (dur, speed_x) {
                        (Some(d), Some(s)) if s > 0.0 => fmt_hms(((d - t) / s).max(0.0)),
                        _ => String::new(),
                    };
                    let sp = speed_x.map(|s| format!("{s:.1}x")).unwrap_or_default();
                    let sp = if dur.is_none() { format!("{} {}", fmt_hms(t), sp) } else { sp };
                    ctx.progress(pct, sp, eta);
                }
            }
            _ => {}
        }
    })
    .await?;
    if !status.success() {
        bail!("ffmpeg 失敗: {}", last_err.unwrap_or_else(|| status.to_string()));
    }
    // 壊れた出力 (途中で切れた等) を検出
    let out_info = probe(ctx, &out.to_string_lossy(), false)
        .await
        .context("出力ファイルの検証に失敗")?;
    if !out_info.has_audio {
        bail!("出力に音声がありません (入力が壊れている可能性): {}", last_err.unwrap_or_default());
    }
    if let (Some(i), Some(o)) = (dur, out_info.duration) {
        if o < i * 0.9 - 1.0 {
            bail!(
                "出力が途中で切れています ({} / {})。{}",
                fmt_hms(o),
                fmt_hms(i),
                last_err.unwrap_or_default()
            );
        }
    }
    ctx.progress(Some(100.0), "", "");
    Ok(())
}

// ───────────────────────── プロセス実行 ─────────────────────────

async fn run_proc(
    ctx: &RunCtx,
    mut cmd: Command,
    name: &str,
    mut on_line: impl FnMut(bool, &str),
) -> Result<ExitStatus> {
    cmd.stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    // 独立したプロセスグループで起動 → 中止時に yt-dlp が呼んだ ffmpeg ごと止める
    #[cfg(unix)]
    cmd.process_group(0);
    let mut child = cmd.spawn().map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            anyhow!("{name} が見つかりません。README のセットアップ手順でインストールしてください")
        } else {
            anyhow!("{name} を起動できません: {e}")
        }
    })?;
    let (ltx, mut lrx) = mpsc::unbounded_channel::<(bool, String)>();
    if let Some(out) = child.stdout.take() {
        tokio::spawn(read_lines(out, false, ltx.clone()));
    }
    if let Some(err) = child.stderr.take() {
        tokio::spawn(read_lines(err, true, ltx.clone()));
    }
    drop(ltx);

    let mut cancel = ctx.cancel.clone();
    loop {
        tokio::select! {
            line = lrx.recv() => match line {
                Some((is_err, l)) => on_line(is_err, &l),
                None => break,
            },
            r = cancel.changed() => {
                if r.is_err() || *cancel.borrow() {
                    #[cfg(unix)]
                    if let Some(pid) = child.id() {
                        unsafe { libc::kill(-(pid as libc::pid_t), libc::SIGKILL) };
                    }
                    let _ = child.kill().await;
                    return Err(CanceledError.into());
                }
            }
        }
    }
    let status = child.wait().await?;
    Ok(status)
}

/// \r と \n の両方で行を区切って送る (UTF-8 が途中で切れても行単位で復元)
async fn read_lines<R: AsyncRead + Unpin>(mut r: R, is_err: bool, tx: mpsc::UnboundedSender<(bool, String)>) {
    let mut buf: Vec<u8> = Vec::with_capacity(512);
    let mut chunk = [0u8; 8192];
    loop {
        let n = match r.read(&mut chunk).await {
            Ok(0) | Err(_) => break,
            Ok(n) => n,
        };
        for &b in &chunk[..n] {
            if b == b'\n' || b == b'\r' {
                if !buf.is_empty() {
                    let s = String::from_utf8_lossy(&buf).trim_end().to_string();
                    buf.clear();
                    if !s.is_empty() && tx.send((is_err, s)).is_err() {
                        return;
                    }
                }
            } else {
                buf.push(b);
            }
        }
    }
    if !buf.is_empty() {
        let _ = tx.send((is_err, String::from_utf8_lossy(&buf).trim_end().to_string()));
    }
}

// ───────────────────────── 後処理 (Termux連携) ─────────────────────────

fn post_actions(ctx: &RunCtx, files: &[PathBuf]) {
    if files.is_empty() {
        return;
    }
    if ctx.cfg.media_scan {
        let mut cmd = Command::new("termux-media-scan");
        for f in files {
            cmd.arg(f);
        }
        spawn_detached(cmd);
    }
    let name = files[0]
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    let body = if files.len() > 1 { format!("{name} ほか{}件", files.len() - 1) } else { name };
    notify(ctx, "ASMR DL: 完了", &body);
}

fn notify(ctx: &RunCtx, title: &str, content: &str) {
    if !ctx.cfg.notify {
        return;
    }
    let mut cmd = Command::new("termux-notification");
    cmd.args(["--group", "asmr-dl", "--title", title, "--content", &short(content)]);
    spawn_detached(cmd);
}

/// termux-api は Termux:API アプリが無いと固まるので、タイムアウト付きで投げっぱなし
fn spawn_detached(mut cmd: Command) {
    cmd.stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .kill_on_drop(true);
    if let Ok(mut child) = cmd.spawn() {
        tokio::spawn(async move {
            if tokio::time::timeout(Duration::from_secs(20), child.wait()).await.is_err() {
                let _ = child.kill().await;
            }
        });
    }
}

// ───────────────────────── ユーティリティ ─────────────────────────

const NON_MEDIA_EXT: &[&str] = &[
    "part", "ytdl", "json", "jpg", "jpeg", "png", "webp", "gif", "vtt", "srt", "ass", "lrc", "txt",
    "description", "temp", "tmp", "url", "html", "xml", "log",
];

/// 一時フォルダ内のメディアファイルを再帰的に列挙
fn collect_media(dir: &Path) -> Vec<PathBuf> {
    fn walk(d: &Path, out: &mut Vec<PathBuf>) {
        let Ok(rd) = std::fs::read_dir(d) else { return };
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                walk(&p, out);
                continue;
            }
            let name = p.file_name().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
            if name.starts_with("__out_") || name.contains(".part-Frag") || name.contains(".temp.") {
                continue;
            }
            let ext = p
                .extension()
                .map(|s| s.to_string_lossy().to_lowercase())
                .unwrap_or_default();
            if ext.is_empty() || NON_MEDIA_EXT.contains(&ext.as_str()) {
                continue;
            }
            out.push(p);
        }
    }
    let mut v = Vec::new();
    walk(dir, &mut v);
    v.sort();
    v
}

/// rename → 失敗(別ファイルシステム)ならコピー&削除
async fn move_file(src: &Path, dst: &Path) -> Result<()> {
    if let Some(parent) = dst.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .with_context(|| format!("出力フォルダを作成できません: {} (termux-setup-storage は実行済み?)", parent.display()))?;
    }
    if tokio::fs::rename(src, dst).await.is_ok() {
        return Ok(());
    }
    tokio::fs::copy(src, dst)
        .await
        .with_context(|| format!("保存に失敗: {}", dst.display()))?;
    let _ = tokio::fs::remove_file(src).await;
    Ok(())
}

fn unique_path(p: &Path) -> PathBuf {
    if !p.exists() {
        return p.to_path_buf();
    }
    let stem = p.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
    let ext = p.extension().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
    for n in 2.. {
        let cand = p.with_file_name(format!("{stem} ({n}).{ext}"));
        if !cand.exists() {
            return cand;
        }
    }
    unreachable!()
}

pub fn sanitize_filename(s: &str) -> String {
    let mut out: String = s
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            c if c.is_control() => '_',
            c => c,
        })
        .collect();
    out = out.trim().trim_matches('.').to_string();
    // 200 バイト程度に制限 (文字境界を守る)
    if out.len() > 200 {
        let mut end = 200;
        while !out.is_char_boundary(end) {
            end -= 1;
        }
        out.truncate(end);
    }
    if out.is_empty() {
        "audio".into()
    } else {
        out
    }
}

const GENERIC_NAMES: &[&str] = &[
    "index", "playlist", "master", "chunklist", "manifest", "video", "audio", "stream", "media",
    "prog_index", "main", "output", "file", "download", "videoplayback",
];

fn is_generic_name(stem: &str) -> bool {
    let s = stem.trim().to_lowercase();
    // "index [index]" のような yt-dlp 汎用抽出の名前も対象
    let s = s.split(" [").next().unwrap_or(&s).trim();
    s.is_empty() || GENERIC_NAMES.contains(&s) || s.starts_with("chunklist_")
}

/// 「ホスト_YYYYmmdd-HHMMSS」形式の名前
fn fallback_name(url: &str, idx: usize, total: usize) -> String {
    let base = format!("{}_{}", host_of(url).unwrap_or_else(|| "audio".into()), local_timestamp());
    if total > 1 {
        format!("{base}_{:02}", idx + 1)
    } else {
        base
    }
}

fn name_from_url(url: &str) -> String {
    let stem = url::Url::parse(url).ok().and_then(|u| {
        u.path_segments()
            .and_then(|mut s| s.next_back().map(|x| x.to_string()))
            .map(|last| {
                let decoded = percent_decode(&last);
                match decoded.rsplit_once('.') {
                    Some((s, _)) => s.to_string(),
                    None => decoded,
                }
            })
    });
    match stem {
        Some(s) if !is_generic_name(&s) => s,
        _ => fallback_name(url, 0, 1),
    }
}

/// 端末のローカル時刻 (Android の TZ 設定に従う)
fn local_timestamp() -> String {
    let t = now_secs() as libc::time_t;
    let mut tm: libc::tm = unsafe { std::mem::zeroed() };
    let ok = unsafe { !libc::localtime_r(&t, &mut tm).is_null() };
    if !ok {
        return t.to_string();
    }
    format!(
        "{:04}{:02}{:02}-{:02}{:02}{:02}",
        tm.tm_year + 1900,
        tm.tm_mon + 1,
        tm.tm_mday,
        tm.tm_hour,
        tm.tm_min,
        tm.tm_sec
    )
}

fn percent_decode(s: &str) -> String {
    fn hex(b: u8) -> Option<u8> {
        match b {
            b'0'..=b'9' => Some(b - b'0'),
            b'a'..=b'f' => Some(b - b'a' + 10),
            b'A'..=b'F' => Some(b - b'A' + 10),
            _ => None,
        }
    }
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let (Some(h), Some(l)) = (hex(bytes[i + 1]), hex(bytes[i + 2])) {
                out.push(h * 16 + l);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn clean_na(s: &str) -> String {
    let t = s.trim();
    if t.is_empty() || t == "NA" || t == "N/A" || t.starts_with("Unknown") {
        String::new()
    } else {
        t.to_string()
    }
}

pub fn fmt_hms(secs: f64) -> String {
    let s = secs.max(0.0) as u64;
    let (h, m, s) = (s / 3600, (s % 3600) / 60, s % 60);
    if h > 0 {
        format!("{h}:{m:02}:{s:02}")
    } else {
        format!("{m:02}:{s:02}")
    }
}

fn short(s: &str) -> String {
    let mut out: String = s.chars().take(120).collect();
    if s.chars().count() > 120 {
        out.push('…');
    }
    out
}

fn now_secs() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names() {
        assert_eq!(name_from_url("https://a.com/x/%E5%A3%B0.mp3?t=1"), "声");
        assert!(name_from_url("https://cdn.a.com/hls/master.m3u8").starts_with("cdn.a.com_20"));
        assert!(is_generic_name("index [index]"));
        assert!(!is_generic_name("耳かきボイス"));
        assert_eq!(sanitize_filename("a/b:c?"), "a_b_c_");
    }

    #[test]
    fn hms() {
        assert_eq!(fmt_hms(65.0), "01:05");
        assert_eq!(fmt_hms(3725.0), "1:02:05");
    }
}
