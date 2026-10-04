//! ジョブ(1 URL = 1 ジョブ)の状態とメッセージ

use std::collections::VecDeque;
use std::path::PathBuf;
use std::time::Instant;
use tokio::sync::watch;

const MAX_LOG_LINES: usize = 400;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Queued,
    Running,
    Done,
    Failed,
    Canceled,
}

impl Status {
    pub fn icon(self) -> &'static str {
        match self {
            Status::Queued => "…",
            Status::Running => "⇣",
            Status::Done => "✔",
            Status::Failed => "✖",
            Status::Canceled => "■",
        }
    }
}

#[derive(Debug)]
pub struct Job {
    pub id: u64,
    pub url: String,
    pub profile_idx: usize,
    pub bitrate: u32,
    pub title: Option<String>,
    pub status: Status,
    pub stage: String,
    pub progress: Option<f64>,
    pub speed: String,
    pub eta: String,
    pub logs: VecDeque<String>,
    pub outputs: Vec<PathBuf>,
    pub error: Option<String>,
    pub cancel: Option<watch::Sender<bool>>,
    pub started: Option<Instant>,
    pub finished: Option<Instant>,
}

impl Job {
    pub fn new(id: u64, url: String, profile_idx: usize, bitrate: u32) -> Self {
        Self {
            id,
            url,
            profile_idx,
            bitrate,
            title: None,
            status: Status::Queued,
            stage: "待機中".into(),
            progress: None,
            speed: String::new(),
            eta: String::new(),
            logs: VecDeque::new(),
            outputs: vec![],
            error: None,
            cancel: None,
            started: None,
            finished: None,
        }
    }

    pub fn push_log(&mut self, s: impl Into<String>) {
        if self.logs.len() >= MAX_LOG_LINES {
            self.logs.pop_front();
        }
        self.logs.push_back(s.into());
    }

    pub fn reset_for_retry(&mut self) {
        self.status = Status::Queued;
        self.stage = "待機中".into();
        self.progress = None;
        self.speed.clear();
        self.eta.clear();
        self.outputs.clear();
        self.error = None;
        self.cancel = None;
        self.started = None;
        self.finished = None;
        self.push_log("──── 再試行 ────");
    }

    pub fn display_name(&self) -> &str {
        self.title.as_deref().unwrap_or(&self.url)
    }

    pub fn elapsed_secs(&self) -> Option<u64> {
        let s = self.started?;
        Some(self.finished.unwrap_or_else(Instant::now).duration_since(s).as_secs())
    }
}

#[derive(Debug, Clone)]
pub enum JobEvent {
    Stage(String),
    Progress {
        pct: Option<f64>,
        speed: String,
        eta: String,
    },
    Title(String),
    Log(String),
    Done(Vec<PathBuf>),
    Failed(String),
    Canceled,
}

#[derive(Debug, Clone)]
pub struct JobMsg {
    pub id: u64,
    pub ev: JobEvent,
}
