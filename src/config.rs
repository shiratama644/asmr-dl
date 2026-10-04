//! 設定ファイルとサイト別プロファイル

use anyhow::{Context, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

pub const DEFAULT_CONFIG: &str = include_str!("../config.default.toml");

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum Backend {
    /// yt-dlp で取得 → 失敗したら ffmpeg で直接
    #[default]
    Auto,
    /// yt-dlp のみ
    Ytdlp,
    /// URL を ffmpeg に直接渡す (m3u8 / 直リンク向け)
    Ffmpeg,
}

impl Backend {
    pub fn label(self) -> &'static str {
        match self {
            Backend::Auto => "auto",
            Backend::Ytdlp => "yt-dlp",
            Backend::Ffmpeg => "ffmpeg",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AudioSettings {
    pub bitrate_kbps: u32,
    /// 0 = 元のチャンネル数を維持 (バイノーラル録音を壊さない)
    pub channels: u32,
    pub copy_if_opus: bool,
    pub normalize: bool,
    pub filter: String,
    pub vbr: String,
    pub compression_level: u8,
    pub frame_duration: f32,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            bitrate_kbps: 128,
            channels: 0,
            copy_if_opus: true,
            normalize: false,
            filter: String::new(),
            vbr: "on".into(),
            compression_level: 10,
            frame_duration: 20.0,
        }
    }
}

impl AudioSettings {
    /// 再エンコードが必須か (フィルタ等が指定されていればコピー不可)
    pub fn needs_processing(&self) -> bool {
        self.normalize || !self.filter.trim().is_empty() || self.channels > 0
    }

    pub fn summary(&self, bitrate: u32) -> String {
        let ch = match self.channels {
            0 => "ch維持".to_string(),
            1 => "mono".to_string(),
            2 => "stereo".to_string(),
            n => format!("{n}ch"),
        };
        let mut s = format!("{bitrate}k {ch}");
        if self.copy_if_opus && !self.needs_processing() {
            s.push_str(" opus→copy");
        }
        if self.normalize {
            s.push_str(" norm");
        }
        if !self.filter.trim().is_empty() {
            s.push_str(" +af");
        }
        s
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Profile {
    pub name: String,
    pub description: String,
    pub domains: Vec<String>,
    pub url_regex: String,
    pub backend: Backend,
    pub format: String,
    pub format_sort: String,
    pub referer: String,
    pub user_agent: String,
    pub headers: Vec<String>,
    pub cookies: String,
    pub concurrent_fragments: u32,
    pub playlist: bool,
    pub split_chapters: bool,
    pub write_thumbnail: bool,
    pub embed_metadata: bool,
    pub subdir: String,
    pub filename: String,
    pub chapter_filename: String,
    pub ytdlp_args: Vec<String>,
    pub audio: AudioSettings,
}

impl Default for Profile {
    fn default() -> Self {
        Self {
            name: "generic".into(),
            description: String::new(),
            domains: vec![],
            url_regex: String::new(),
            backend: Backend::Auto,
            format: "ba/b".into(),
            format_sort: String::new(),
            referer: String::new(),
            user_agent: String::new(),
            headers: vec![],
            cookies: String::new(),
            concurrent_fragments: 4,
            playlist: false,
            split_chapters: false,
            write_thumbnail: false,
            embed_metadata: true,
            subdir: String::new(),
            filename: "%(title).150B [%(id)s].%(ext)s".into(),
            chapter_filename:
                "%(title).100B/%(section_number)03d %(section_title).100B.%(ext)s".into(),
            ytdlp_args: vec![],
            audio: AudioSettings::default(),
        }
    }
}

impl Profile {
    pub fn matches(&self, url: &str, host: Option<&str>) -> bool {
        if !self.url_regex.is_empty() {
            if let Ok(re) = Regex::new(&self.url_regex) {
                if re.is_match(url) {
                    return true;
                }
            }
        }
        if let Some(h) = host {
            return self.domains.iter().any(|d| host_matches(h, d));
        }
        false
    }

    /// "Key: Value" 形式の追加ヘッダ一覧 (Referer / UA / Cookie 含む)
    pub fn http_headers(&self, url: &str) -> Vec<(String, String)> {
        let mut v = Vec::new();
        if !self.referer.is_empty() {
            v.push(("Referer".into(), self.referer.clone()));
        }
        if !self.user_agent.is_empty() {
            v.push(("User-Agent".into(), self.user_agent.clone()));
        }
        for h in &self.headers {
            if let Some((k, val)) = h.split_once(':') {
                v.push((k.trim().to_string(), val.trim().to_string()));
            }
        }
        if !self.cookies.is_empty() {
            if let Some(host) = host_of(url) {
                if let Some(c) = cookie_header(&expand_tilde(&self.cookies), &host) {
                    v.push(("Cookie".into(), c));
                }
            }
        }
        v
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub output_dir: String,
    pub temp_dir: String,
    pub max_concurrent: usize,
    pub ytdlp_path: String,
    pub ffmpeg_path: String,
    pub ffprobe_path: String,
    pub fallback_to_ffmpeg: bool,
    pub keep_temp: bool,
    pub wake_lock: bool,
    pub media_scan: bool,
    pub notify: bool,
    pub mouse: bool,
    pub bitrate_presets: Vec<u32>,
    pub profiles: Vec<Profile>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            output_dir: "~/storage/shared/Music/ASMR".into(),
            temp_dir: "~/.cache/asmr-dl".into(),
            max_concurrent: 2,
            ytdlp_path: "yt-dlp".into(),
            ffmpeg_path: "ffmpeg".into(),
            ffprobe_path: "ffprobe".into(),
            fallback_to_ffmpeg: true,
            keep_temp: false,
            wake_lock: true,
            media_scan: true,
            notify: true,
            mouse: true,
            bitrate_presets: vec![64, 96, 128, 160, 192, 256],
            profiles: vec![],
        }
    }
}

impl Config {
    pub fn config_path() -> PathBuf {
        if let Ok(p) = std::env::var("ASMR_DL_CONFIG") {
            return PathBuf::from(p);
        }
        let base = std::env::var("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| home_dir().join(".config"));
        base.join("asmr-dl").join("config.toml")
    }

    /// 設定を読み込む。無ければ既定値を書き出す
    pub fn load() -> Result<(Config, PathBuf, Option<String>)> {
        let path = Self::config_path();
        let mut note = None;
        let text = if path.exists() {
            std::fs::read_to_string(&path)
                .with_context(|| format!("設定ファイルを読めません: {}", path.display()))?
        } else {
            if let Some(dir) = path.parent() {
                let _ = std::fs::create_dir_all(dir);
            }
            match std::fs::write(&path, DEFAULT_CONFIG) {
                Ok(_) => note = Some(format!("設定ファイルを作成: {}", path.display())),
                Err(e) => note = Some(format!("設定ファイルを作成できません: {e}")),
            }
            DEFAULT_CONFIG.to_string()
        };
        let mut cfg: Config = toml::from_str(&text)
            .with_context(|| format!("設定ファイルの書式エラー: {}", path.display()))?;
        cfg.normalize();
        Ok((cfg, path, note))
    }

    pub fn normalize(&mut self) {
        if self.profiles.is_empty() {
            let def: Config = toml::from_str(DEFAULT_CONFIG).expect("default config is valid");
            self.profiles = def.profiles;
        }
        if !self.profiles.iter().any(|p| p.name == "generic") {
            self.profiles.push(Profile::default());
        }
        // generic は常に最後 (フォールバック)
        if let Some(i) = self.profiles.iter().position(|p| p.name == "generic") {
            let g = self.profiles.remove(i);
            self.profiles.push(g);
        }
        if self.max_concurrent == 0 {
            self.max_concurrent = 1;
        }
        if self.bitrate_presets.is_empty() {
            self.bitrate_presets = vec![96, 128, 160, 192];
        }
    }

    pub fn generic_index(&self) -> usize {
        self.profiles.len() - 1
    }

    pub fn detect_profile(&self, url: &str) -> usize {
        let host = host_of(url);
        let gi = self.generic_index();
        self.profiles
            .iter()
            .enumerate()
            .filter(|(i, _)| *i != gi)
            .find(|(_, p)| p.matches(url, host.as_deref()))
            .map(|(i, _)| i)
            .unwrap_or(gi)
    }

    pub fn output_root(&self) -> PathBuf {
        let p = expand_tilde(&self.output_dir);
        // termux-setup-storage していない場合のフォールバック
        if self.output_dir.starts_with("~/storage") && !home_dir().join("storage").exists() {
            return home_dir().join("asmr-dl-output");
        }
        p
    }
}

pub fn home_dir() -> PathBuf {
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/data/data/com.termux/files/home"))
}

pub fn expand_tilde(s: &str) -> PathBuf {
    if s == "~" {
        home_dir()
    } else if let Some(rest) = s.strip_prefix("~/") {
        home_dir().join(rest)
    } else {
        PathBuf::from(s)
    }
}

pub fn host_of(url: &str) -> Option<String> {
    url::Url::parse(url)
        .ok()
        .and_then(|u| u.host_str().map(|h| h.trim_start_matches("www.").to_lowercase()))
}

fn host_matches(host: &str, domain: &str) -> bool {
    let d = domain.trim().trim_start_matches('.').to_lowercase();
    host == d || host.ends_with(&format!(".{d}"))
}

/// テキストから http(s) URL を全部抜き出す (共有テキスト貼り付け対応)
pub fn extract_urls(text: &str) -> Vec<String> {
    static RE: OnceLock<Regex> = OnceLock::new();
    let re = RE.get_or_init(|| {
        Regex::new(r#"https?://[^\s<>"'`「」『』（）、。　]+"#).expect("valid regex")
    });
    let mut out: Vec<String> = Vec::new();
    for m in re.find_iter(text) {
        let u = m
            .as_str()
            .trim_end_matches(|c| matches!(c, '.' | ',' | ')' | ']' | '!' | '?' | ';'))
            .to_string();
        if url::Url::parse(&u).is_ok() && !out.contains(&u) {
            out.push(u);
        }
    }
    out
}

/// Netscape 形式 cookies.txt から指定ホスト向けの Cookie ヘッダ値を作る
pub fn cookie_header(path: &Path, host: &str) -> Option<String> {
    let text = std::fs::read_to_string(path).ok()?;
    let mut pairs = Vec::new();
    for line in text.lines() {
        let line = line.strip_prefix("#HttpOnly_").unwrap_or(line);
        if line.starts_with('#') || line.trim().is_empty() {
            continue;
        }
        let f: Vec<&str> = line.split('\t').collect();
        if f.len() < 7 {
            continue;
        }
        let domain = f[0].trim_start_matches('.').to_lowercase();
        if host == domain || host.ends_with(&format!(".{domain}")) {
            pairs.push(format!("{}={}", f[5], f[6]));
        }
    }
    if pairs.is_empty() {
        None
    } else {
        Some(pairs.join("; "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_parses() {
        let mut cfg: Config = toml::from_str(DEFAULT_CONFIG).unwrap();
        cfg.normalize();
        assert!(cfg.profiles.len() >= 5);
        assert_eq!(cfg.profiles.last().unwrap().name, "generic");
        let yt = &cfg.profiles[cfg.detect_profile("https://youtu.be/abc")];
        assert_eq!(yt.name, "youtube");
        assert_eq!(yt.audio.bitrate_kbps, 160);
        assert!(yt.embed_metadata);
    }

    #[test]
    fn detection() {
        let mut cfg: Config = toml::from_str(DEFAULT_CONFIG).unwrap();
        cfg.normalize();
        let name = |u: &str| cfg.profiles[cfg.detect_profile(u)].name.clone();
        assert_eq!(name("https://m.youtube.com/watch?v=x"), "youtube");
        assert_eq!(name("https://www.nicovideo.jp/watch/sm9"), "niconico");
        assert_eq!(name("https://cdn.example.com/a/master.m3u8?token=1"), "hls-direct");
        assert_eq!(name("https://example.com/voice.mp3"), "file-direct");
        assert_eq!(name("https://example.com/page"), "generic");
        assert_eq!(name("https://x.com/user/status/1"), "x");
    }

    #[test]
    fn urls_from_share_text() {
        let t = "この動画おすすめ→https://youtu.be/abc123?si=xx。あと https://example.com/a.m3u8, 以上";
        let v = extract_urls(t);
        assert_eq!(v, vec!["https://youtu.be/abc123?si=xx", "https://example.com/a.m3u8"]);
    }
}
