//! アプリ状態とキー操作

use crate::config::{extract_urls, Config};
use crate::job::{Job, JobEvent, JobMsg, Status};
use crate::runner::{self, RunCtx};
use crate::AppMsg;
use ratatui::crossterm::event::{
    Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};
use ratatui::layout::Rect;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, watch};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Input,
    Normal,
    ProfilePicker,
    Help,
    ConfirmQuit,
}

#[derive(Debug, Clone, Default)]
pub struct ToolStatus {
    pub checked: bool,
    pub ytdlp: Option<String>,
    pub ffmpeg: Option<String>,
    pub ffprobe: bool,
    pub libopus: bool,
}

/// UI 描画時に記録するクリック判定用の領域
#[derive(Debug, Clone, Default)]
pub struct HitAreas {
    pub input: Rect,
    pub jobs: Rect,
    pub jobs_offset: usize,
    pub log: Rect,
}

pub struct App {
    pub cfg: Arc<Config>,
    pub config_path: PathBuf,
    pub mode: Mode,
    pub input: String,
    pub cursor: usize, // 文字単位
    pub profile_override: Option<usize>,
    pub bitrate_override: Option<u32>,
    pub jobs: Vec<Job>,
    pub selected: usize,
    pub show_log: bool,
    pub log_scroll: u16,
    pub picker_idx: usize,
    pub status_msg: Option<(String, Instant)>,
    pub next_id: u64,
    pub tx: mpsc::UnboundedSender<AppMsg>,
    pub should_quit: bool,
    pub tools: ToolStatus,
    pub hit: HitAreas,
}

impl App {
    pub fn new(cfg: Config, config_path: PathBuf, tx: mpsc::UnboundedSender<AppMsg>) -> Self {
        Self {
            cfg: Arc::new(cfg),
            config_path,
            mode: Mode::Input,
            input: String::new(),
            cursor: 0,
            profile_override: None,
            bitrate_override: None,
            jobs: Vec::new(),
            selected: 0,
            show_log: true,
            log_scroll: 0,
            picker_idx: 0,
            status_msg: None,
            next_id: 1,
            tx,
            should_quit: false,
            tools: ToolStatus::default(),
            hit: HitAreas::default(),
        }
    }

    pub fn flash(&mut self, s: impl Into<String>) {
        self.status_msg = Some((s.into(), Instant::now()));
    }

    // ───────── プロファイル / ビットレート ─────────

    /// 入力欄の URL から自動判定 (または手動指定) したプロファイル
    pub fn current_profile_idx(&self) -> (usize, bool) {
        if let Some(i) = self.profile_override {
            return (i, false);
        }
        let urls = extract_urls(&self.input);
        match urls.first() {
            Some(u) => (self.cfg.detect_profile(u), true),
            None => (self.cfg.generic_index(), true),
        }
    }

    pub fn current_bitrate(&self) -> u32 {
        let (pi, _) = self.current_profile_idx();
        self.bitrate_override
            .unwrap_or(self.cfg.profiles[pi].audio.bitrate_kbps)
    }

    fn cycle_profile(&mut self, forward: bool) {
        let n = self.cfg.profiles.len();
        self.profile_override = match (self.profile_override, forward) {
            (None, true) => Some(0),
            (Some(i), true) if i + 1 < n => Some(i + 1),
            (Some(_), true) => None,
            (None, false) => Some(n - 1),
            (Some(0), false) => None,
            (Some(i), false) => Some(i - 1),
        };
        let name = self
            .profile_override
            .map(|i| self.cfg.profiles[i].name.clone())
            .unwrap_or_else(|| "自動判定".into());
        self.flash(format!("プロファイル: {name}"));
    }

    fn cycle_bitrate(&mut self) {
        let presets = &self.cfg.bitrate_presets;
        let cur = self.current_bitrate();
        let next = match presets.iter().position(|&b| b > cur) {
            Some(i) => Some(presets[i]),
            None => None, // 一周したらプロファイル既定に戻す
        };
        self.bitrate_override = next;
        match next {
            Some(b) => self.flash(format!("ビットレート: {b} kbps")),
            None => self.flash(format!("ビットレート: プロファイル既定 ({} kbps)", self.current_bitrate())),
        }
    }

    // ───────── キュー ─────────

    pub fn enqueue_text(&mut self, text: &str) -> usize {
        let urls = extract_urls(text);
        let n = urls.len();
        for u in urls {
            let pi = self.profile_override.unwrap_or_else(|| self.cfg.detect_profile(&u));
            let br = self
                .bitrate_override
                .unwrap_or(self.cfg.profiles[pi].audio.bitrate_kbps);
            let job = Job::new(self.next_id, u, pi, br);
            self.next_id += 1;
            self.jobs.push(job);
        }
        if n > 0 {
            self.selected = self.jobs.len() - 1;
            self.log_scroll = 0;
        }
        n
    }

    fn submit_input(&mut self) {
        if self.input.trim().is_empty() {
            self.mode = Mode::Normal;
            return;
        }
        let text = std::mem::take(&mut self.input);
        self.cursor = 0;
        let n = self.enqueue_text(&text);
        if n == 0 {
            self.input = text;
            self.cursor = self.input.chars().count();
            self.flash("URL が見つかりません (http:// か https:// で始まる URL を入力)");
        } else {
            self.flash(format!("{n} 件をキューに追加"));
            // 手動指定はワンショット
            self.profile_override = None;
        }
    }

    /// 空きがあれば待機中ジョブを開始
    pub fn schedule(&mut self) {
        let running = self.jobs.iter().filter(|j| j.status == Status::Running).count();
        let mut slots = self.cfg.max_concurrent.saturating_sub(running);
        if slots == 0 {
            return;
        }
        for job in self.jobs.iter_mut().filter(|j| j.status == Status::Queued) {
            if slots == 0 {
                break;
            }
            slots -= 1;
            let (ctx_tx, ctx_rx) = watch::channel(false);
            job.status = Status::Running;
            job.stage = "開始".into();
            job.started = Some(Instant::now());
            job.cancel = Some(ctx_tx);
            let ctx = RunCtx {
                id: job.id,
                url: job.url.clone(),
                profile: self.cfg.profiles[job.profile_idx].clone(),
                bitrate: job.bitrate,
                cfg: self.cfg.clone(),
                tx: self.tx.clone(),
                cancel: ctx_rx,
            };
            tokio::spawn(runner::run(ctx));
        }
    }

    pub fn running_count(&self) -> usize {
        self.jobs.iter().filter(|j| j.status == Status::Running).count()
    }

    pub fn cancel_all(&mut self) {
        for j in &mut self.jobs {
            if let Some(c) = &j.cancel {
                let _ = c.send(true);
            }
        }
    }

    // ───────── メッセージ ─────────

    pub fn on_msg(&mut self, msg: AppMsg) {
        match msg {
            AppMsg::Tools(t) => {
                self.tools = t;
                if self.tools.ytdlp.is_none() || self.tools.ffmpeg.is_none() {
                    self.flash("yt-dlp / ffmpeg が見つかりません。? でヘルプ");
                } else if !self.tools.libopus {
                    self.flash("ffmpeg に libopus がありません (pkg install ffmpeg で再インストール)");
                }
            }
            AppMsg::Job(m) => self.on_job_msg(m),
        }
    }

    fn on_job_msg(&mut self, JobMsg { id, ev }: JobMsg) {
        let Some(job) = self.jobs.iter_mut().find(|j| j.id == id) else { return };
        match ev {
            JobEvent::Stage(s) => job.stage = s,
            JobEvent::Progress { pct, speed, eta } => {
                job.progress = pct;
                job.speed = speed;
                job.eta = eta;
            }
            JobEvent::Title(t) => {
                if job.title.as_deref() != Some(t.as_str()) {
                    job.push_log(format!("タイトル: {t}"));
                    job.title = Some(t);
                }
            }
            JobEvent::Log(l) => job.push_log(l),
            JobEvent::Done(files) => {
                job.status = Status::Done;
                job.stage = "完了".into();
                job.progress = Some(100.0);
                job.speed.clear();
                job.eta.clear();
                job.push_log(format!("✔ 完了 ({} ファイル)", files.len()));
                job.outputs = files;
                job.cancel = None;
                job.finished = Some(Instant::now());
            }
            JobEvent::Failed(e) => {
                job.status = Status::Failed;
                job.stage = "失敗".into();
                job.push_log(format!("✖ {e}"));
                job.error = Some(e);
                job.cancel = None;
                job.finished = Some(Instant::now());
            }
            JobEvent::Canceled => {
                job.status = Status::Canceled;
                job.stage = "キャンセル".into();
                job.push_log("■ キャンセルしました");
                job.cancel = None;
                job.finished = Some(Instant::now());
            }
        }
    }

    pub fn on_tick(&mut self) {
        if let Some((_, t)) = &self.status_msg {
            if t.elapsed() > Duration::from_secs(4) {
                self.status_msg = None;
            }
        }
    }

    // ───────── 入力イベント ─────────

    pub fn on_event(&mut self, ev: Event) {
        match ev {
            Event::Key(k) if k.kind != KeyEventKind::Release => self.on_key(k),
            Event::Paste(s) => self.on_paste(&s),
            Event::Mouse(m) => self.on_mouse(m),
            _ => {}
        }
    }

    fn on_paste(&mut self, s: &str) {
        // 複数 URL を貼り付けたら即キューへ
        let urls = extract_urls(s);
        if urls.len() > 1 {
            let n = self.enqueue_text(s);
            self.flash(format!("貼り付けから {n} 件をキューに追加"));
            return;
        }
        self.mode = Mode::Input;
        let clean: String = s.chars().filter(|c| !c.is_control()).collect();
        self.insert_str(&clean);
    }

    fn on_mouse(&mut self, m: MouseEvent) {
        if !matches!(self.mode, Mode::Input | Mode::Normal) {
            return;
        }
        let inside = |r: Rect| m.column >= r.x && m.column < r.x + r.width && m.row >= r.y && m.row < r.y + r.height;
        match m.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                if inside(self.hit.input) {
                    self.mode = Mode::Input;
                } else if inside(self.hit.jobs) {
                    self.mode = Mode::Normal;
                    // 各ジョブは 2 行 + 枠 1 行
                    let row = (m.row.saturating_sub(self.hit.jobs.y + 1)) as usize / 2;
                    let idx = self.hit.jobs_offset + row;
                    if idx < self.jobs.len() {
                        if self.selected == idx {
                            self.show_log = !self.show_log;
                        }
                        self.selected = idx;
                        self.log_scroll = 0;
                    }
                }
            }
            MouseEventKind::ScrollUp => {
                if inside(self.hit.log) {
                    self.log_scroll = self.log_scroll.saturating_add(3);
                } else {
                    self.select_prev();
                }
            }
            MouseEventKind::ScrollDown => {
                if inside(self.hit.log) {
                    self.log_scroll = self.log_scroll.saturating_sub(3);
                } else {
                    self.select_next();
                }
            }
            _ => {}
        }
    }

    fn on_key(&mut self, k: KeyEvent) {
        let ctrl = k.modifiers.contains(KeyModifiers::CONTROL);
        // Ctrl+C はどこでも終了
        if ctrl && k.code == KeyCode::Char('c') {
            self.request_quit();
            return;
        }
        match self.mode {
            Mode::Input => self.key_input(k, ctrl),
            Mode::Normal => self.key_normal(k, ctrl),
            Mode::ProfilePicker => self.key_picker(k),
            Mode::Help => {
                self.mode = Mode::Normal;
            }
            Mode::ConfirmQuit => match k.code {
                KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
                    self.cancel_all();
                    self.should_quit = true;
                }
                _ => self.mode = Mode::Normal,
            },
        }
    }

    fn request_quit(&mut self) {
        if self.running_count() > 0 {
            self.mode = Mode::ConfirmQuit;
        } else {
            self.should_quit = true;
        }
    }

    fn key_input(&mut self, k: KeyEvent, ctrl: bool) {
        match k.code {
            KeyCode::Enter => self.submit_input(),
            KeyCode::Esc => self.mode = Mode::Normal,
            KeyCode::Down if !self.jobs.is_empty() => self.mode = Mode::Normal,
            KeyCode::Tab => self.cycle_profile(true),
            KeyCode::BackTab => self.cycle_profile(false),
            KeyCode::F(1) => self.mode = Mode::Help,
            KeyCode::F(2) => self.open_picker(),
            KeyCode::Char('p') if ctrl => self.open_picker(),
            KeyCode::Char('b') if ctrl => self.cycle_bitrate(),
            KeyCode::Char('u') if ctrl => {
                self.input.clear();
                self.cursor = 0;
            }
            KeyCode::Char('a') if ctrl => self.cursor = 0,
            KeyCode::Char('e') if ctrl => self.cursor = self.input.chars().count(),
            KeyCode::Char('w') if ctrl => self.delete_word(),
            KeyCode::Char(c) if !ctrl => self.insert_str(&c.to_string()),
            KeyCode::Backspace => {
                if self.cursor > 0 {
                    let b = self.byte_idx(self.cursor - 1);
                    self.input.remove(b);
                    self.cursor -= 1;
                }
            }
            KeyCode::Delete => {
                if self.cursor < self.input.chars().count() {
                    let b = self.byte_idx(self.cursor);
                    self.input.remove(b);
                }
            }
            KeyCode::Left => self.cursor = self.cursor.saturating_sub(1),
            KeyCode::Right => self.cursor = (self.cursor + 1).min(self.input.chars().count()),
            KeyCode::Home => self.cursor = 0,
            KeyCode::End => self.cursor = self.input.chars().count(),
            _ => {}
        }
    }

    fn key_normal(&mut self, k: KeyEvent, ctrl: bool) {
        match k.code {
            KeyCode::Char('q') => self.request_quit(),
            KeyCode::Char('i') | KeyCode::Char('a') | KeyCode::Char('/') | KeyCode::Esc => {
                self.mode = Mode::Input
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.selected == 0 {
                    self.mode = Mode::Input;
                } else {
                    self.select_prev();
                }
            }
            KeyCode::Down | KeyCode::Char('j') => self.select_next(),
            KeyCode::Home | KeyCode::Char('g') => self.selected = 0,
            KeyCode::End | KeyCode::Char('G') => self.selected = self.jobs.len().saturating_sub(1),
            KeyCode::Enter | KeyCode::Char('l') => {
                self.show_log = !self.show_log;
                self.log_scroll = 0;
            }
            KeyCode::PageUp => self.log_scroll = self.log_scroll.saturating_add(5),
            KeyCode::PageDown => self.log_scroll = self.log_scroll.saturating_sub(5),
            KeyCode::Char('c') | KeyCode::Char('x') if !ctrl => self.cancel_selected(),
            KeyCode::Char('r') => self.retry_selected(),
            KeyCode::Char('d') | KeyCode::Delete => self.remove_selected(),
            KeyCode::Char('C') => self.clear_finished(),
            KeyCode::Char('R') => self.retry_all_failed(),
            KeyCode::Char('o') => self.open_selected(),
            KeyCode::Char('p') | KeyCode::F(2) => self.open_picker(),
            KeyCode::Char('b') => self.cycle_bitrate(),
            KeyCode::Tab => self.cycle_profile(true),
            KeyCode::BackTab => self.cycle_profile(false),
            KeyCode::Char('?') | KeyCode::F(1) => self.mode = Mode::Help,
            _ => {}
        }
    }

    fn key_picker(&mut self, k: KeyEvent) {
        let n = self.cfg.profiles.len() + 1; // 0 = 自動判定
        match k.code {
            KeyCode::Esc | KeyCode::Char('q') => self.mode = Mode::Input,
            KeyCode::Up | KeyCode::Char('k') => self.picker_idx = (self.picker_idx + n - 1) % n,
            KeyCode::Down | KeyCode::Char('j') | KeyCode::Tab => self.picker_idx = (self.picker_idx + 1) % n,
            KeyCode::Enter => {
                self.profile_override = if self.picker_idx == 0 { None } else { Some(self.picker_idx - 1) };
                let name = self
                    .profile_override
                    .map(|i| self.cfg.profiles[i].name.clone())
                    .unwrap_or_else(|| "自動判定".into());
                self.flash(format!("プロファイル: {name}"));
                self.mode = Mode::Input;
            }
            _ => {}
        }
    }

    fn open_picker(&mut self) {
        self.picker_idx = self.profile_override.map(|i| i + 1).unwrap_or(0);
        self.mode = Mode::ProfilePicker;
    }

    // ───────── ジョブ操作 ─────────

    fn select_prev(&mut self) {
        self.selected = self.selected.saturating_sub(1);
        self.log_scroll = 0;
    }
    fn select_next(&mut self) {
        if self.selected + 1 < self.jobs.len() {
            self.selected += 1;
            self.log_scroll = 0;
        }
    }

    fn cancel_selected(&mut self) {
        let Some(j) = self.jobs.get_mut(self.selected) else { return };
        match j.status {
            Status::Running => {
                if let Some(c) = &j.cancel {
                    let _ = c.send(true);
                }
                j.stage = "キャンセル中…".into();
            }
            Status::Queued => {
                j.status = Status::Canceled;
                j.stage = "キャンセル".into();
            }
            _ => {}
        }
    }

    fn retry_selected(&mut self) {
        if let Some(j) = self.jobs.get_mut(self.selected) {
            if matches!(j.status, Status::Failed | Status::Canceled) {
                j.reset_for_retry();
            }
        }
    }

    fn retry_all_failed(&mut self) {
        let mut n = 0;
        for j in &mut self.jobs {
            if matches!(j.status, Status::Failed) {
                j.reset_for_retry();
                n += 1;
            }
        }
        self.flash(format!("{n} 件を再試行"));
    }

    fn remove_selected(&mut self) {
        if let Some(j) = self.jobs.get(self.selected) {
            if j.status == Status::Running {
                self.flash("実行中のジョブは先に c でキャンセルしてください");
                return;
            }
            self.jobs.remove(self.selected);
            if self.selected >= self.jobs.len() {
                self.selected = self.jobs.len().saturating_sub(1);
            }
        }
    }

    fn clear_finished(&mut self) {
        let before = self.jobs.len();
        self.jobs.retain(|j| j.status != Status::Done);
        self.selected = self.selected.min(self.jobs.len().saturating_sub(1));
        self.flash(format!("完了 {} 件を一覧から削除", before - self.jobs.len()));
    }

    /// 完了ファイルを Android 側のアプリで開く (termux-open)
    fn open_selected(&mut self) {
        let Some(j) = self.jobs.get(self.selected) else { return };
        let Some(f) = j.outputs.first().cloned() else {
            self.flash("まだ出力ファイルがありません");
            return;
        };
        let r = std::process::Command::new("termux-open")
            .arg(&f)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn();
        match r {
            Ok(_) => self.flash(format!("開く: {}", f.display())),
            Err(e) => self.flash(format!("termux-open 失敗: {e}")),
        }
    }

    // ───────── テキスト編集 ─────────

    fn byte_idx(&self, char_idx: usize) -> usize {
        self.input
            .char_indices()
            .nth(char_idx)
            .map(|(b, _)| b)
            .unwrap_or(self.input.len())
    }

    fn insert_str(&mut self, s: &str) {
        let b = self.byte_idx(self.cursor);
        self.input.insert_str(b, s);
        self.cursor += s.chars().count();
    }

    fn delete_word(&mut self) {
        let chars: Vec<char> = self.input.chars().collect();
        let mut i = self.cursor;
        while i > 0 && chars[i - 1].is_whitespace() {
            i -= 1;
        }
        while i > 0 && !chars[i - 1].is_whitespace() && chars[i - 1] != '/' {
            i -= 1;
        }
        if i == self.cursor && i > 0 {
            i -= 1;
        }
        let (a, b) = (self.byte_idx(i), self.byte_idx(self.cursor));
        self.input.replace_range(a..b, "");
        self.cursor = i;
    }
}
