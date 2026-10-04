//! 描画 (Catppuccin Mocha ベースのモダンテーマ / スマホ縦画面 40〜60 桁で崩れないレイアウト)
//!
//! デザイン方針:
//! - ヘッダ・フッタは `BG_ALT` の全幅バー (アプリの「chrome」)
//! - 角丸枠 (`BorderType::Rounded`) + 抑制された surface 階調
//! - 状態色はパレットの 6 色 (accent 粉 / blue / teal / green / red / yellow)
//! - 実行中ジョブはブレイル・スピナー (200ms tick 連動、決定論)

use crate::app::{App, Mode};
use crate::job::Status;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{
    Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap,
};
use ratatui::Frame;
use unicode_width::UnicodeWidthStr;

/// Catppuccin Mocha ベースのカラーパレット (ASMR 用のピンクアクセント)
mod theme {
    use ratatui::style::Color;

    pub const BG: Color = Color::Rgb(30, 30, 46); // base (ポップアップの背景)
    pub const BG_ALT: Color = Color::Rgb(24, 24, 37); // mantle (ヘッダ/フッタのバー)
    pub const SURFACE: Color = Color::Rgb(49, 50, 68); // surface0 (チップ・バーの残)
    pub const SURFACE_HI: Color = Color::Rgb(69, 71, 90); // surface1 (選択行)
    pub const TEXT: Color = Color::Rgb(205, 214, 244); // text
    pub const DIM: Color = Color::Rgb(166, 173, 200); // subtext0
    pub const FAINT: Color = Color::Rgb(108, 112, 134); // overlay0
    pub const ACCENT: Color = Color::Rgb(245, 194, 231); // pink
    pub const BLUE: Color = Color::Rgb(137, 180, 250);
    pub const TEAL: Color = Color::Rgb(148, 226, 213);
    pub const GREEN: Color = Color::Rgb(166, 227, 161);
    pub const RED: Color = Color::Rgb(243, 139, 168);
    pub const YELLOW: Color = Color::Rgb(249, 226, 175);
}
use theme::*;

/// 実行中ジョブ用のブレイル・スピナー (10 フレーム / 200ms tick ごとに 1 進み)
const SPIN: &[char] = &['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];

fn spin(frame: u64) -> char {
    SPIN[(frame % SPIN.len()) as usize]
}

/// スパン列の表示幅 (右端まで背景バーを伸ばすのに使う)
fn spans_width(spans: &[Span<'static>]) -> usize {
    spans.iter().map(|s| s.content.as_ref().width()).sum()
}

pub fn draw(f: &mut Frame, app: &mut App) {
    let area = f.area();
    if area.width < 20 || area.height < 8 {
        return; // 極端に小さい端末では描画しない (レイアウト破綻防止)
    }
    let log_visible = app.show_log && !app.jobs.is_empty() && area.height >= 24;
    let footer_h = if area.width < 70 { 2 } else { 1 };

    let mut cons = vec![
        Constraint::Length(1), // header
        Constraint::Length(3), // input
        Constraint::Length(2), // profile
        Constraint::Min(6),    // jobs
    ];
    if log_visible {
        cons.push(Constraint::Percentage(38));
    }
    cons.push(Constraint::Length(footer_h));
    let chunks = Layout::vertical(cons).split(area);

    draw_header(f, app, chunks[0]);
    draw_input(f, app, chunks[1]);
    draw_profile(f, app, chunks[2]);
    draw_jobs(f, app, chunks[3]);
    if log_visible {
        draw_log(f, app, chunks[4]);
    } else {
        app.hit.log = Rect::default();
    }
    draw_footer(f, app, *chunks.last().unwrap());

    match app.mode {
        Mode::ProfilePicker => draw_picker(f, app, area),
        Mode::Help => draw_help(f, app, area),
        Mode::ConfirmQuit => draw_confirm(f, app, area),
        _ => {}
    }
}

// ───────────────────────── ヘッダ ─────────────────────────

fn draw_header(f: &mut Frame, app: &App, area: Rect) {
    let t = &app.tools;
    let w = area.width as usize;
    let bar = Style::new().bg(BG_ALT);
    let mut s: Vec<Span> = vec![
        Span::styled(" ♪ ASMR→Opus ", Style::new().fg(BG).bg(ACCENT).bold()),
        Span::styled(" ", bar),
    ];
    let mark = |ok: bool| {
        if ok {
            Span::styled("✓", Style::new().fg(GREEN).bg(BG_ALT))
        } else {
            Span::styled("✗", Style::new().fg(RED).bg(BG_ALT))
        }
    };
    if !t.checked {
        s.push(Span::styled(" ツール確認中…", Style::new().fg(FAINT).bg(BG_ALT)));
    } else if w < 60 {
        // 縦画面ではツール名を短縮 (40 桁でも収まる)
        s.push(Span::styled(" yt", Style::new().fg(DIM).bg(BG_ALT)));
        s.push(mark(t.ytdlp.is_some()));
        s.push(Span::styled(" ff", Style::new().fg(DIM).bg(BG_ALT)));
        s.push(mark(t.ffmpeg.is_some() && t.ffprobe));
        s.push(Span::styled(" opus", Style::new().fg(DIM).bg(BG_ALT)));
        s.push(mark(t.libopus));
    } else {
        s.push(Span::styled(" yt-dlp", Style::new().fg(DIM).bg(BG_ALT)));
        s.push(mark(t.ytdlp.is_some()));
        s.push(Span::styled(" ffmpeg", Style::new().fg(DIM).bg(BG_ALT)));
        s.push(mark(t.ffmpeg.is_some() && t.ffprobe));
        s.push(Span::styled(" opus", Style::new().fg(DIM).bg(BG_ALT)));
        s.push(mark(t.libopus));
    }
    let running = app.running_count();
    let queued = app.jobs.iter().filter(|j| j.status == Status::Queued).count();
    if running + queued > 0 {
        s.push(Span::styled(
            format!("  ⇣{running} · {queued} 待機"),
            Style::new().fg(BLUE).bg(BG_ALT).bold(),
        ));
    }
    // 右端までバーの背景を伸ばす
    let used = spans_width(&s);
    if used < w {
        s.push(Span::styled(" ".repeat(w - used), bar));
    }
    f.render_widget(Paragraph::new(Line::from(s)), area);
}

// ───────────────────────── 入力 ─────────────────────────

fn draw_input(f: &mut Frame, app: &mut App, area: Rect) {
    app.hit.input = area;
    let focused = app.mode == Mode::Input;
    let border = if focused { ACCENT } else { FAINT };
    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(border))
        .title(Line::from(vec![
            Span::styled(" URL", Style::new().fg(if focused { ACCENT } else { DIM }).bold()),
            Span::styled(
                if focused { "  Enter=追加 · Esc=一覧" } else { "  i=入力" },
                Style::new().fg(FAINT),
            ),
        ]));
    let inner = block.inner(area);

    let before: String = app.input.chars().take(app.cursor).collect();
    let cur_w = before.width() as u16;
    let scroll = cur_w.saturating_sub(inner.width.saturating_sub(1));

    let content = if app.input.is_empty() && !focused {
        Line::styled("URL を貼り付け (長押し→貼り付け)", Style::new().fg(FAINT))
    } else if app.input.is_empty() {
        Line::styled("https://… (共有テキストごと貼ってもOK)", Style::new().fg(FAINT))
    } else {
        Line::styled(app.input.as_str(), Style::new().fg(TEXT))
    };
    f.render_widget(Paragraph::new(content).block(block).scroll((0, scroll)), area);
    if focused {
        f.set_cursor_position((inner.x + cur_w - scroll, inner.y));
    }
}

// ───────────────────────── プロファイル ─────────────────────────

fn draw_profile(f: &mut Frame, app: &App, area: Rect) {
    let (pi, auto) = app.current_profile_idx();
    let p = &app.cfg.profiles[pi];
    let br = app.current_bitrate();
    let l1 = Line::from(vec![
        Span::styled("◈ ", Style::new().fg(ACCENT).bold()),
        Span::styled(format!(" {} ", p.name), Style::new().fg(TEAL).bold().bg(SURFACE)),
        Span::styled(if auto { " 自動" } else { " 手動" }, Style::new().fg(FAINT)),
        Span::styled(format!("  {}", p.audio.summary(br)), Style::new().fg(BLUE)),
        if app.bitrate_override.is_some() {
            Span::styled(" *", Style::new().fg(YELLOW))
        } else {
            Span::raw(" ")
        },
        Span::styled(format!("  {}", p.backend.label()), Style::new().fg(FAINT)),
    ]);
    let l2 = Line::styled(format!("  {}", p.description), Style::new().fg(FAINT));
    f.render_widget(Paragraph::new(vec![l1, l2]), area);
}

// ───────────────────────── 進捗バー ─────────────────────────

/// 進捗バー (決定論: 不定進捗のアニメーションは tick フレームから算出)
fn bar(pct: Option<f64>, width: usize, frame: u64) -> Vec<Span<'static>> {
    let width = width.max(4);
    match pct {
        Some(p) => {
            let filled = ((p / 100.0) * width as f64).round() as usize;
            let filled = filled.min(width);
            vec![
                Span::styled("█".repeat(filled), Style::new().fg(ACCENT)),
                Span::styled("░".repeat(width - filled), Style::new().fg(SURFACE)),
            ]
        }
        None => {
            let t = (frame % width as u64) as usize;
            let s: String = (0..width)
                .map(|i| if (i + width - t) % width < 3 { '▓' } else { '░' })
                .collect();
            vec![Span::styled(s, Style::new().fg(BLUE))]
        }
    }
}

// ───────────────────────── ジョブラスト ─────────────────────────

fn draw_jobs(f: &mut Frame, app: &mut App, area: Rect) {
    app.hit.jobs = area;
    let focused = app.mode == Mode::Normal;
    let done = app.jobs.iter().filter(|j| j.status == Status::Done).count();
    let running = app.running_count();
    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(if focused { ACCENT } else { FAINT }))
        .title(Line::from(vec![
            Span::styled(" ジョブ ", Style::new().fg(TEXT).bold()),
            Span::styled(format!("{done}/{} 完了", app.jobs.len()), Style::new().fg(FAINT)),
            if running > 0 {
                Span::styled(format!("  ⇣{running} 実行中"), Style::new().fg(BLUE))
            } else {
                Span::raw("")
            },
        ]));
    let inner_w = block.inner(area).width.saturating_sub(2) as usize; // ハイライト記号分

    if app.jobs.is_empty() {
        let help = vec![
            Line::raw(" "),
            Line::from(vec![
                Span::styled("  ♪", Style::new().fg(ACCENT).bold()),
                Span::styled(" URL を貼り付けて Enter で開始", Style::new().fg(DIM)),
            ]),
            Line::styled("  動画でも音声でも .opus で保存します", Style::new().fg(FAINT)),
            Line::raw(" "),
            Line::styled("  ? ヘルプ · Tab プロファイル切替 · ^B 音質", Style::new().fg(FAINT)),
        ];
        f.render_widget(Paragraph::new(help).block(block), area);
        return;
    }

    let root = app.cfg.output_root();
    let items: Vec<ListItem> = app
        .jobs
        .iter()
        .map(|j| {
            let pname = &app.cfg.profiles[j.profile_idx].name;
            let (icon, icon_style) = match j.status {
                Status::Queued => ("…".to_string(), Style::new().fg(FAINT)),
                Status::Running => (spin(app.tick_count).to_string(), Style::new().fg(ACCENT)),
                Status::Done => ("✔".to_string(), Style::new().fg(GREEN)),
                Status::Failed => ("✖".to_string(), Style::new().fg(RED)),
                Status::Canceled => ("■".to_string(), Style::new().fg(FAINT)),
            };
            let name_style = match j.status {
                Status::Running => Style::new().fg(TEXT).bold(),
                Status::Done => Style::new().fg(DIM),
                Status::Failed => Style::new().fg(TEXT),
                Status::Canceled => Style::new().fg(FAINT),
                Status::Queued => Style::new().fg(DIM),
            };
            let l1 = Line::from(vec![
                Span::styled(format!("{icon} "), icon_style),
                Span::styled(format!("[{pname}]"), Style::new().fg(TEAL)),
                Span::styled(" ", Style::new()),
                Span::styled(j.display_name().to_string(), name_style),
            ]);
            let l2 = match j.status {
                Status::Running => {
                    let pct_s = j
                        .progress
                        .map(|p| format!("{p:5.1}%"))
                        .unwrap_or_else(|| "   --%".into());
                    let mut tail = format!(" {}", j.stage);
                    if !j.speed.is_empty() {
                        tail.push_str(&format!(" {}", j.speed));
                    }
                    if !j.eta.is_empty() {
                        tail.push_str(&format!(" 残{}", j.eta));
                    }
                    let tail_w = pct_s.width() + tail.width();
                    let bar_w = inner_w.saturating_sub(tail_w + 2).clamp(6, 40);
                    let mut spans = vec![Span::raw("  ")];
                    spans.extend(bar(j.progress, bar_w, app.tick_count));
                    spans.push(Span::styled(pct_s, Style::new().fg(BLUE).bold()));
                    spans.push(Span::styled(tail, Style::new().fg(DIM)));
                    Line::from(spans)
                }
                Status::Done => {
                    let first = j
                        .outputs
                        .first()
                        .map(|p| p.strip_prefix(&root).unwrap_or(p).display().to_string())
                        .unwrap_or_default();
                    let more = if j.outputs.len() > 1 {
                        format!(" +{}件", j.outputs.len() - 1)
                    } else {
                        String::new()
                    };
                    let el = j
                        .elapsed_secs()
                        .map(|s| format!(" ({})", crate::runner::fmt_hms(s as f64)))
                        .unwrap_or_default();
                    Line::from(vec![
                        Span::styled(format!(" → {first}"), Style::new().fg(GREEN)),
                        Span::styled(format!("{more}{el}"), Style::new().fg(FAINT)),
                    ])
                }
                Status::Failed => {
                    let e = j.error.as_deref().unwrap_or("失敗").lines().next().unwrap_or("");
                    Line::styled(format!("  {e}"), Style::new().fg(RED))
                }
                Status::Canceled => Line::styled("  キャンセル済み (r で再試行)", Style::new().fg(FAINT)),
                Status::Queued => Line::styled(format!("  待機中 · {}kbps", j.bitrate), Style::new().fg(FAINT)),
            };
            ListItem::new(Text::from(vec![l1, l2]))
        })
        .collect();

    let list = List::new(items)
        .block(block)
        .highlight_symbol("▸ ")
        .highlight_style(Style::new().bg(SURFACE_HI));
    let mut state = ListState::default().with_selected(Some(app.selected.min(app.jobs.len() - 1)));
    // 前回のオフセットを維持してスクロールがガタつかないようにする
    *state.offset_mut() = app.hit.jobs_offset;
    f.render_stateful_widget(list, area, &mut state);
    app.hit.jobs_offset = state.offset();
}

// ───────────────────────── ログ ─────────────────────────

fn draw_log(f: &mut Frame, app: &mut App, area: Rect) {
    app.hit.log = area;
    let Some(job) = app.jobs.get(app.selected) else { return };
    let pname = &app.cfg.profiles[job.profile_idx].name;
    let mut lines: Vec<Line> = vec![Line::from(vec![
        Span::styled(format!("[{pname}] "), Style::new().fg(TEAL)),
        Span::styled(job.url.clone(), Style::new().fg(DIM)),
    ])];
    for o in &job.outputs {
        lines.push(Line::styled(format!("→ {}", o.display()), Style::new().fg(GREEN)));
    }
    for l in &job.logs {
        let style = if l.starts_with("ERROR") || l.starts_with('✖') {
            Style::new().fg(RED)
        } else if l.starts_with("WARNING") || l.starts_with('⚠') {
            Style::new().fg(YELLOW)
        } else if l.starts_with('✔') || l.starts_with("保存") {
            Style::new().fg(GREEN)
        } else {
            Style::new().fg(DIM)
        };
        lines.push(Line::styled(l.clone(), style));
    }
    let block = Block::new()
        .borders(Borders::TOP)
        .border_style(Style::new().fg(FAINT))
        .title(Line::from(vec![
            Span::styled(format!(" ログ #{} ", job.id), Style::new().fg(TEXT).bold()),
            Span::styled(
                if app.log_scroll > 0 { format!("↑{} ", app.log_scroll) } else { String::new() },
                Style::new().fg(YELLOW),
            ),
        ]));
    let inner = block.inner(area);
    let para = Paragraph::new(lines).wrap(Wrap { trim: false });
    let total = para.line_count(inner.width) as u16;
    let max_scroll = total.saturating_sub(inner.height);
    if app.log_scroll > max_scroll {
        app.log_scroll = max_scroll;
    }
    let y = max_scroll.saturating_sub(app.log_scroll);
    f.render_widget(para.block(block).scroll((y, 0)), area);
}

// ───────────────────────── フッタ (ステータスバー) ─────────────────────────

fn draw_footer(f: &mut Frame, app: &App, area: Rect) {
    let w = area.width as usize;
    if let Some((msg, _)) = &app.status_msg {
        let line = Line::from(vec![
            Span::styled(" ⚠ ", Style::new().fg(YELLOW).bold()),
            Span::styled(msg.as_str(), Style::new().fg(YELLOW)),
        ]);
        f.render_widget(Paragraph::new(line).wrap(Wrap { trim: true }), area);
        return;
    }
    let key = |k: &str| Span::styled(format!(" {k} "), Style::new().fg(TEXT).bg(SURFACE));
    let txt = |t: &str| Span::styled(format!(" {t} "), Style::new().fg(DIM).bg(BG_ALT));
    let mut spans = match app.mode {
        Mode::Input => vec![
            key("Enter"),
            txt("追加"),
            key("Tab"),
            txt("プロファイル"),
            key("^B"),
            txt("音質"),
            key("Esc"),
            txt("一覧"),
            key("F1"),
            txt("ヘルプ"),
        ],
        _ => vec![
            key("↑↓"),
            txt("選択"),
            key("i"),
            txt("入力"),
            key("c"),
            txt("中止"),
            key("r"),
            txt("再試行"),
            key("o"),
            txt("再生"),
            key("l"),
            txt("ログ"),
            key("p"),
            txt("プロファイル"),
            key("q"),
            txt("終了"),
        ],
    };
    // 広い画面では右端までバーの背景を伸ばす (縦画面はラップで 2 行になる)
    if area.width >= 70 {
        let used = spans_width(&spans);
        if used < w {
            spans.push(Span::styled(" ".repeat(w - used), Style::new().bg(BG_ALT)));
        }
    }
    f.render_widget(Paragraph::new(Line::from(spans)).wrap(Wrap { trim: true }), area);
}

// ───────────────────────── ポップアップ ─────────────────────────

fn popup_area(area: Rect, w_pct: u16, h: u16) -> Rect {
    // スマホ縦画面では全幅にする (全角文字が枠にかかって表示が崩れるのを防ぐ)
    let w = if area.width < 60 {
        area.width
    } else {
        (area.width as u32 * w_pct as u32 / 100).max(30).min(area.width as u32) as u16
    };
    let h = h.min(area.height);
    Rect {
        x: area.x + (area.width - w) / 2,
        y: area.y + (area.height - h) / 2,
        width: w,
        height: h,
    }
}

fn draw_picker(f: &mut Frame, app: &App, area: Rect) {
    let n = app.cfg.profiles.len() + 1;
    let r = popup_area(area, 92, (n as u16) * 2 + 2);
    f.render_widget(Clear, r);
    let mut items = vec![ListItem::new(Text::from(vec![
        Line::styled(" 自動判定", Style::new().fg(TEXT).bold()),
        Line::styled("   URL のドメインから選択", Style::new().fg(FAINT)),
    ]))];
    for p in &app.cfg.profiles {
        items.push(ListItem::new(Text::from(vec![
            Line::from(vec![
                Span::styled(format!(" {} ", p.name), Style::new().fg(TEAL).bold().bg(SURFACE)),
                Span::styled(
                    format!("  {}", p.audio.summary(p.audio.bitrate_kbps)),
                    Style::new().fg(BLUE),
                ),
            ]),
            Line::styled(format!("  {}", p.description), Style::new().fg(FAINT)),
        ])));
    }
    let list = List::new(items)
        .block(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .border_style(Style::new().fg(ACCENT))
                .title(Line::from(vec![
                    Span::styled(" プロファイル", Style::new().fg(ACCENT).bold()),
                    Span::styled("  Enter=決定 / Esc=戻る", Style::new().fg(FAINT)),
                ])),
        )
        .style(Style::new().bg(BG))
        .highlight_symbol("▸ ")
        .highlight_style(Style::new().bg(SURFACE_HI).add_modifier(Modifier::BOLD));
    let mut st = ListState::default().with_selected(Some(app.picker_idx));
    f.render_stateful_widget(list, r, &mut st);
}

fn draw_help(f: &mut Frame, app: &App, area: Rect) {
    let r = popup_area(area, 94, area.height.saturating_sub(2));
    f.render_widget(Clear, r);
    let k = |s: &str| Span::styled(format!("{s:<10}"), Style::new().fg(BLUE).bold());
    let row = |key: &str, desc: &str| Line::from(vec![k(key), Span::raw(desc.to_string())]);
    let env_row = |name: &str, value: String| {
        Line::from(vec![
            Span::styled(format!("{name:<7}"), Style::new().fg(FAINT)),
            Span::styled(value, Style::new().fg(TEXT)),
        ])
    };
    let t = &app.tools;
    let mut lines = vec![
        Line::styled(" ■ 入力モード", Style::new().fg(ACCENT).bold()),
        row("Enter", "URLをキューに追加 (複数URL可)"),
        row("Tab", "プロファイル切替 (自動→各サイト)"),
        row("Ctrl+P/F2", "プロファイル一覧から選択"),
        row("Ctrl+B", "ビットレート切替"),
        row("Ctrl+U/W", "全消去 / 1語消去"),
        row("Esc / ↓", "ジョブ一覧へ"),
        Line::raw(" "),
        Line::styled(" ■ 一覧モード", Style::new().fg(ACCENT).bold()),
        row("↑↓ / jk", "選択 (タップでも可)"),
        row("Enter / l", "ログ表示切替"),
        row("PgUp/PgDn", "ログスクロール (スワイプ可)"),
        row("c / x", "選択ジョブを中止"),
        row("r / R", "再試行 / 失敗を全部再試行"),
        row("d / C", "削除 / 完了済みを一掃"),
        row("o", "完了ファイルを Android アプリで開く"),
        row("i / Esc", "入力モードへ"),
        row("q", "終了"),
        Line::raw(" "),
        Line::styled(" ■ 環境", Style::new().fg(ACCENT).bold()),
        env_row(
            "yt-dlp:",
            t.ytdlp
                .clone()
                .unwrap_or_else(|| "未検出 → pip install -U \"yt-dlp[default]\"".into()),
        ),
        env_row(
            "ffmpeg:",
            t.ffmpeg
                .clone()
                .unwrap_or_else(|| "未検出 → pkg install ffmpeg".into()),
        ),
        env_row("libopus", if t.libopus { "OK".into() } else { "なし/未確認".into() }),
        env_row("設定", app.config_path.display().to_string()),
        env_row("出力先", app.cfg.output_root().display().to_string()),
        Line::raw(" "),
        Line::styled("何かキーを押すと閉じます", Style::new().fg(FAINT)),
    ];
    if t.ytdlp.is_none() || t.ffmpeg.is_none() {
        lines.insert(
            0,
            Line::styled("⚠ 必要なツールが不足しています (下の環境欄参照)", Style::new().fg(RED).bold()),
        );
    }
    f.render_widget(
        Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .block(
                Block::bordered()
                    .border_type(BorderType::Rounded)
                    .border_style(Style::new().fg(ACCENT))
                    .title(Line::from(vec![Span::styled(
                        " ヘルプ",
                        Style::new().fg(ACCENT).bold(),
                    )])),
            ),
        r,
    );
}

fn draw_confirm(f: &mut Frame, app: &App, area: Rect) {
    let r = popup_area(area, 80, 5);
    f.render_widget(Clear, r);
    f.render_widget(
        Paragraph::new(vec![
            Line::styled(
                format!("  実行中のジョブが {} 件あります。", app.running_count()),
                Style::new().fg(TEXT),
            ),
            Line::styled("  中止して終了しますか? (y / n)", Style::new().fg(YELLOW).bold()),
        ])
        .wrap(Wrap { trim: true })
        .block(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .border_style(Style::new().fg(RED))
                .title(Line::from(vec![Span::styled(
                    " 終了確認",
                    Style::new().fg(RED).bold(),
                )])),
        ),
        r,
    );
}
