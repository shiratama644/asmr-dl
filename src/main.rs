//! asmr-dl — URL から動画/音声 (HLS/m3u8・DASH・直リンク等) を取得して Opus にする TUI
//!
//! 使い方:  asmr-dl [URL ...]
//!   起動時に URL を渡すとすぐキューに入ります (termux-url-opener 連携用)

mod app;
mod config;
mod job;
mod runner;
mod ui;

use anyhow::Result;
use app::{App, ToolStatus};
use config::Config;
use ratatui::crossterm::event::{
    self, DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture,
};
use ratatui::crossterm::execute;
use std::io::stdout;
use std::process::Stdio;
use std::time::Duration;
use tokio::process::Command;
use tokio::sync::mpsc;

pub enum AppMsg {
    Job(job::JobMsg),
    Tools(ToolStatus),
}

fn print_usage() {
    println!(
        "asmr-dl {}\n\n使い方: asmr-dl [URL ...]\n\n  URL を渡すと起動と同時にキューへ追加します。\n  設定ファイル: {}\n  (環境変数 ASMR_DL_CONFIG で変更可)\n\nオプション:\n  -h, --help          このヘルプ\n  --print-config      既定の設定ファイル内容を表示\n",
        env!("CARGO_PKG_VERSION"),
        Config::config_path().display()
    );
}

fn is_termux() -> bool {
    std::env::var("TERMUX_VERSION").is_ok()
        || std::env::var("PREFIX").map(|p| p.contains("com.termux")).unwrap_or(false)
}

fn fire_and_forget(cmd: &str) {
    let _ = std::process::Command::new(cmd)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
}

async fn cmd_output(bin: &str, args: &[&str]) -> Option<String> {
    let out = tokio::time::timeout(
        Duration::from_secs(20),
        Command::new(bin).args(args).stdin(Stdio::null()).kill_on_drop(true).output(),
    )
    .await
    .ok()?
    .ok()?;
    if !out.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&out.stdout).to_string())
}

async fn check_tools(cfg: &Config) -> ToolStatus {
    let (yt, ff, fp, enc) = tokio::join!(
        cmd_output(&cfg.ytdlp_path, &["--version"]),
        cmd_output(&cfg.ffmpeg_path, &["-hide_banner", "-version"]),
        cmd_output(&cfg.ffprobe_path, &["-hide_banner", "-version"]),
        cmd_output(&cfg.ffmpeg_path, &["-hide_banner", "-encoders"]),
    );
    ToolStatus {
        checked: true,
        ytdlp: yt.map(|s| s.trim().to_string()),
        ffmpeg: ff.map(|s| {
            s.lines()
                .next()
                .unwrap_or("")
                .split_whitespace()
                .take(3)
                .collect::<Vec<_>>()
                .join(" ")
        }),
        ffprobe: fp.is_some(),
        libopus: enc.map(|s| s.contains("libopus")).unwrap_or(false),
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "-h" || a == "--help") {
        print_usage();
        return Ok(());
    }
    if args.iter().any(|a| a == "--print-config") {
        print!("{}", config::DEFAULT_CONFIG);
        return Ok(());
    }

    let (cfg, cfg_path, note) = match Config::load() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("設定の読み込みに失敗しました:\n{e:#}");
            std::process::exit(1);
        }
    };
    let _ = std::fs::create_dir_all(config::expand_tilde(&cfg.temp_dir));

    let (tx, mut rx) = mpsc::unbounded_channel::<AppMsg>();
    let mut app = App::new(cfg, cfg_path, tx.clone());
    if let Some(n) = note {
        app.flash(n);
    }
    let n = app.enqueue_text(&args.join(" "));
    if n > 0 {
        app.mode = app::Mode::Normal;
        app.flash(format!("引数から {n} 件をキューに追加"));
    }

    // ツール確認はバックグラウンドで (python の起動は少し遅い)
    {
        let cfg = app.cfg.clone();
        let tx = tx.clone();
        tokio::spawn(async move {
            let t = check_tools(&cfg).await;
            let _ = tx.send(AppMsg::Tools(t));
        });
    }

    let termux = is_termux();
    if termux && app.cfg.wake_lock {
        fire_and_forget("termux-wake-lock");
    }

    // 端末初期化 (ratatui::init は raw mode + 代替画面 + panic 時の復元フック)
    let mut terminal = ratatui::init();
    let mouse = app.cfg.mouse;
    let _ = execute!(stdout(), EnableBracketedPaste);
    if mouse {
        let _ = execute!(stdout(), EnableMouseCapture);
    }

    // キー入力は専用スレッドで読む
    let (etx, mut erx) = mpsc::unbounded_channel();
    std::thread::spawn(move || loop {
        match event::read() {
            Ok(ev) => {
                if etx.send(ev).is_err() {
                    break;
                }
            }
            Err(_) => break,
        }
    });

    let mut tick = tokio::time::interval(Duration::from_millis(200));
    let res: Result<()> = async {
        loop {
            app.schedule();
            terminal.draw(|f| ui::draw(f, &mut app))?;
            if app.should_quit {
                break;
            }
            tokio::select! {
                Some(ev) = erx.recv() => {
                    app.on_event(ev);
                    while let Ok(ev) = erx.try_recv() { app.on_event(ev); }
                }
                Some(msg) = rx.recv() => {
                    app.on_msg(msg);
                    // 溜まった進捗をまとめて処理してから描画
                    while let Ok(m) = rx.try_recv() { app.on_msg(m); }
                }
                _ = tick.tick() => app.on_tick(),
            }
        }
        Ok(())
    }
    .await;

    app.cancel_all();
    if mouse {
        let _ = execute!(stdout(), DisableMouseCapture);
    }
    let _ = execute!(stdout(), DisableBracketedPaste);
    ratatui::restore();

    if termux && app.cfg.wake_lock {
        fire_and_forget("termux-wake-unlock");
    }
    // 子プロセスに kill が届くよう少し待つ
    tokio::time::sleep(Duration::from_millis(300)).await;

    let done = app.jobs.iter().filter(|j| j.status == job::Status::Done).count();
    if done > 0 {
        println!("完了 {done} 件 → {}", app.cfg.output_root().display());
    }
    res
}
