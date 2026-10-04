//! 描画 (スマホ縦画面 = 幅 40〜60 桁程度でも崩れないレイアウト)

use crate::app::{App, Mode};
use crate::job::Status;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap};
use ratatui::Frame;
use unicode_width::UnicodeWidthStr;

const ACCENT: Color = Color::Rgb(255, 140, 200); // ASMR っぽいピンク
const ACCENT2: Color = Color::Rgb(150, 200, 255);
const DIM: Color = Color::DarkGray;
const OK: Color = Color::Rgb(120, 220, 140);
const ERR: Color = Color::Rgb(255, 110, 110);
const WARN: Color = Color::Rgb(255, 200, 100);

pub fn draw(f: &mut Frame, app: &mut App) {
    let area = f.area();
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

fn draw_header(f: &mut Frame, app: &App, area: Rect) {
    let t = &app.tools;
    let mut spans = vec![Span::styled(" ♪ ASMR→Opus ", Style::new().fg(Color::Black).bg(ACCENT).bold()), Span::raw(" ")];
    if !t.checked {
        spans.push(Span::styled("ツール確認中…", Style::new().fg(DIM)));
    } else {
        let mark = |ok: bool| if ok { Span::styled("✓", Style::new().fg(OK)) } else { Span::styled("✗", Style::new().fg(ERR)) };
        spans.push(Span::raw("yt-dlp"));
        spans.push(mark(t.ytdlp.is_some()));
        spans.push(Span::raw(" ffmpeg"));
        spans.push(mark(t.ffmpeg.is_some() && t.ffprobe));
        spans.push(Span::raw(" opus"));
        spans.push(mark(t.libopus));
    }
    let running = app.running_count();
    let queued = app.jobs.iter().filter(|j| j.status == Status::Queued).count();
    if running + queued > 0 {
        spans.push(Span::styled(format!("  ⇣{running} …{queued}"), Style::new().fg(ACCENT2)));
    }
    f.render_widget(Paragraph::new(Line::from(spans)), area);
}

fn draw_input(f: &mut Frame, app: &mut App, area: Rect) {
    app.hit.input = area;
    let focused = app.mode == Mode::Input;
    let border = if focused { ACCENT } else { DIM };
    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(border))
        .title(Line::from(vec![
            Span::styled(" URL ", Style::new().fg(border).bold()),
            Span::styled(if focused { "Enter=追加 " } else { "i=入力 " }, Style::new().fg(DIM)),
        ]));
    let inner = block.inner(area);

    let before: String = app.input.chars().take(app.cursor).collect();
    let cur_w = before.width() as u16;
    let scroll = cur_w.saturating_sub(inner.width.saturating_sub(1));

    let content = if app.input.is_empty() && !focused {
        Line::styled("URL を貼り付け (長押し→貼り付け)", Style::new().fg(DIM))
    } else if app.input.is_empty() {
        Line::styled("https://… (共有テキストごと貼ってもOK)", Style::new().fg(DIM))
    } else {
        Line::raw(app.input.as_str())
    };
    f.render_widget(Paragraph::new(content).block(block).scroll((0, scroll)), area);
    if focused {
        f.set_cursor_position((inner.x + cur_w - scroll, inner.y));
    }
}

fn draw_profile(f: &mut Frame, app: &App, area: Rect) {
    let (pi, auto) = app.current_profile_idx();
    let p = &app.cfg.profiles[pi];
    let br = app.current_bitrate();
    let l1 = Line::from(vec![
        Span::styled(" ▶ ", Style::new().fg(ACCENT)),
        Span::styled(p.name.clone(), Style::new().fg(ACCENT).bold()),
        Span::styled(if auto { " (自動)" } else { " (手動)" }, Style::new().fg(DIM)),
        Span::raw("  "),
        Span::styled(p.audio.summary(br), Style::new().fg(ACCENT2)),
        if app.bitrate_override.is_some() { Span::styled("*", Style::new().fg(WARN)) } else { Span::raw("") },
        Span::styled(format!("  {}", p.backend.label()), Style::new().fg(DIM)),
    ]);
    let l2 = Line::styled(format!("   {}", p.description), Style::new().fg(DIM));
    f.render_widget(Paragraph::new(vec![l1, l2]), area);
}

fn progress_bar(pct: Option<f64>, width: usize) -> Vec<Span<'static>> {
    let width = width.max(4);
    match pct {
        Some(p) => {
            let filled = ((p / 100.0) * width as f64).round() as usize;
            let filled = filled.min(width);
            vec![
                Span::styled("█".repeat(filled), Style::new().fg(ACCENT)),
                Span::styled("░".repeat(width - filled), Style::new().fg(DIM)),
            ]
        }
        None => {
            // 不定進捗: 流れるアニメーション
            let t = (std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis())
                .unwrap_or(0)
                / 150) as usize;
            let s: String = (0..width)
                .map(|i| if (i + width - t % width) % width < 3 { '▓' } else { '░' })
                .collect();
            vec![Span::styled(s, Style::new().fg(ACCENT2))]
        }
    }
}

fn draw_jobs(f: &mut Frame, app: &mut App, area: Rect) {
    app.hit.jobs = area;
    let focused = app.mode == Mode::Normal;
    let done = app.jobs.iter().filter(|j| j.status == Status::Done).count();
    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(if focused { ACCENT } else { DIM }))
        .title(Line::from(vec![
            Span::styled(" ジョブ ", Style::new().bold()),
            Span::styled(format!("{done}/{} ", app.jobs.len()), Style::new().fg(DIM)),
        ]));
    let inner_w = block.inner(area).width.saturating_sub(2) as usize; // ハイライト記号分

    if app.jobs.is_empty() {
        let help = vec![
            Line::raw(""),
            Line::styled(" URL を入力して Enter で開始", Style::new().fg(DIM)),
            Line::styled(" 動画でも音声でも .opus で保存します", Style::new().fg(DIM)),
            Line::styled(" ? でヘルプ / Tab でプロファイル切替", Style::new().fg(DIM)),
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
            let (icon_style, name_style) = match j.status {
                Status::Done => (Style::new().fg(OK), Style::new()),
                Status::Failed => (Style::new().fg(ERR), Style::new()),
                Status::Canceled => (Style::new().fg(DIM), Style::new().fg(DIM)),
                Status::Running => (Style::new().fg(ACCENT), Style::new().bold()),
                Status::Queued => (Style::new().fg(DIM), Style::new()),
            };
            let l1 = Line::from(vec![
                Span::styled(format!("{} ", j.status.icon()), icon_style),
                Span::styled(format!("[{pname}] "), Style::new().fg(ACCENT2)),
                Span::styled(j.display_name().to_string(), name_style),
            ]);
            let l2 = match j.status {
                Status::Running => {
                    let pct_s = j.progress.map(|p| format!(" {p:5.1}%")).unwrap_or_else(|| "   --%".into());
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
                    spans.extend(progress_bar(j.progress, bar_w));
                    spans.push(Span::styled(pct_s, Style::new().bold()));
                    spans.push(Span::styled(tail, Style::new().fg(DIM)));
                    Line::from(spans)
                }
                Status::Done => {
                    let first = j
                        .outputs
                        .first()
                        .map(|p| p.strip_prefix(&root).unwrap_or(p).display().to_string())
                        .unwrap_or_default();
                    let more = if j.outputs.len() > 1 { format!(" ほか{}件", j.outputs.len() - 1) } else { String::new() };
                    let el = j.elapsed_secs().map(|s| format!(" ({})", crate::runner::fmt_hms(s as f64))).unwrap_or_default();
                    Line::from(vec![
                        Span::styled(format!("  → {first}"), Style::new().fg(OK)),
                        Span::styled(format!("{more}{el}"), Style::new().fg(DIM)),
                    ])
                }
                Status::Failed => Line::styled(
                    format!("  {}", j.error.as_deref().unwrap_or("失敗").lines().next().unwrap_or("")),
                    Style::new().fg(ERR),
                ),
                Status::Canceled => Line::styled("  キャンセル済み (r で再試行)", Style::new().fg(DIM)),
                Status::Queued => Line::styled(format!("  待機中 · {}kbps", j.bitrate), Style::new().fg(DIM)),
            };
            ListItem::new(Text::from(vec![l1, l2]))
        })
        .collect();

    let list = List::new(items)
        .block(block)
        .highlight_symbol("▌ ")
        .highlight_style(Style::new().bg(Color::Rgb(45, 35, 55)));
    let mut state = ListState::default().with_selected(Some(app.selected.min(app.jobs.len() - 1)));
    // 前回のオフセットを維持してスクロールがガタつかないようにする
    *state.offset_mut() = app.hit.jobs_offset;
    f.render_stateful_widget(list, area, &mut state);
    app.hit.jobs_offset = state.offset();
}

fn draw_log(f: &mut Frame, app: &mut App, area: Rect) {
    app.hit.log = area;
    let Some(job) = app.jobs.get(app.selected) else { return };
    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::styled(job.url.clone(), Style::new().fg(ACCENT2)));
    for o in &job.outputs {
        lines.push(Line::styled(format!("→ {}", o.display()), Style::new().fg(OK)));
    }
    for l in &job.logs {
        let style = if l.starts_with("ERROR") || l.starts_with('✖') {
            Style::new().fg(ERR)
        } else if l.starts_with("WARNING") || l.starts_with('⚠') {
            Style::new().fg(WARN)
        } else if l.starts_with('✔') || l.starts_with("保存") {
            Style::new().fg(OK)
        } else {
            Style::new().fg(Color::Gray)
        };
        lines.push(Line::styled(l.clone(), style));
    }
    let block = Block::new()
        .borders(Borders::TOP)
        .border_style(Style::new().fg(DIM))
        .title(Line::from(vec![
            Span::styled(format!(" ログ #{} ", job.id), Style::new().bold()),
            Span::styled(
                if app.log_scroll > 0 { format!("↑{} ", app.log_scroll) } else { String::new() },
                Style::new().fg(WARN),
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

fn draw_footer(f: &mut Frame, app: &App, area: Rect) {
    if let Some((msg, _)) = &app.status_msg {
        f.render_widget(
            Paragraph::new(Line::styled(format!(" {msg}"), Style::new().fg(WARN))).wrap(Wrap { trim: true }),
            area,
        );
        return;
    }
    let key = |k: &str| Span::styled(k.to_string(), Style::new().fg(Color::Black).bg(ACCENT2));
    let txt = |t: &str| Span::styled(format!("{t} "), Style::new().fg(DIM));
    let spans = match app.mode {
        Mode::Input => vec![
            key("Enter"), txt("追加"), key("Tab"), txt("プロファイル"), key("^B"), txt("音質"),
            key("Esc"), txt("一覧"), key("F1"), txt("ヘルプ"),
        ],
        _ => vec![
            key("↑↓"), txt("選択"), key("i"), txt("入力"), key("c"), txt("中止"), key("r"), txt("再試行"),
            key("o"), txt("再生"), key("l"), txt("ログ"), key("p"), txt("プロファイル"), key("?"), txt(""), key("q"), txt("終了"),
        ],
    };
    f.render_widget(Paragraph::new(Line::from(spans)).wrap(Wrap { trim: true }), area);
}

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
        Line::styled("自動判定", Style::new().bold()),
        Line::styled("  URL のドメインから選択", Style::new().fg(DIM)),
    ]))];
    for p in &app.cfg.profiles {
        items.push(ListItem::new(Text::from(vec![
            Line::from(vec![
                Span::styled(p.name.clone(), Style::new().bold().fg(ACCENT)),
                Span::styled(format!("  {}", p.audio.summary(p.audio.bitrate_kbps)), Style::new().fg(ACCENT2)),
            ]),
            Line::styled(format!("  {}", p.description), Style::new().fg(DIM)),
        ])));
    }
    let list = List::new(items)
        .block(
            Block::bordered()
                .border_type(BorderType::Double)
                .border_style(Style::new().fg(ACCENT))
                .title(" プロファイル選択 (Enter/Esc) "),
        )
        .highlight_symbol("▶ ")
        .highlight_style(Style::new().bg(Color::Rgb(45, 35, 55)).add_modifier(Modifier::BOLD));
    let mut st = ListState::default().with_selected(Some(app.picker_idx));
    f.render_stateful_widget(list, r, &mut st);
}

fn draw_help(f: &mut Frame, app: &App, area: Rect) {
    let r = popup_area(area, 94, area.height.saturating_sub(2));
    f.render_widget(Clear, r);
    let k = |s: &str| Span::styled(format!("{s:<10}"), Style::new().fg(ACCENT2).bold());
    let row = |key: &str, desc: &str| Line::from(vec![k(key), Span::raw(desc.to_string())]);
    let t = &app.tools;
    let mut lines = vec![
        Line::styled("■ 入力モード", Style::new().fg(ACCENT).bold()),
        row("Enter", "URLをキューに追加 (複数URL可)"),
        row("Tab", "プロファイル切替 (自動→各サイト)"),
        row("Ctrl+P/F2", "プロファイル一覧から選択"),
        row("Ctrl+B", "ビットレート切替"),
        row("Ctrl+U/W", "全消去 / 1語消去"),
        row("Esc / ↓", "ジョブ一覧へ"),
        Line::raw(""),
        Line::styled("■ 一覧モード", Style::new().fg(ACCENT).bold()),
        row("↑↓ / jk", "選択 (タップでも可)"),
        row("Enter / l", "ログ表示切替"),
        row("PgUp/PgDn", "ログスクロール (スワイプ可)"),
        row("c / x", "選択ジョブを中止"),
        row("r / R", "再試行 / 失敗を全部再試行"),
        row("d / C", "削除 / 完了済みを一掃"),
        row("o", "完了ファイルを Android アプリで開く"),
        row("i / Esc", "入力モードへ"),
        row("q", "終了"),
        Line::raw(""),
        Line::styled("■ 環境", Style::new().fg(ACCENT).bold()),
        Line::raw(format!("yt-dlp : {}", t.ytdlp.as_deref().unwrap_or("未検出 → pip install -U \"yt-dlp[default]\""))),
        Line::raw(format!("ffmpeg : {}", t.ffmpeg.as_deref().unwrap_or("未検出 → pkg install ffmpeg"))),
        Line::raw(format!("libopus: {}", if t.libopus { "OK" } else { "なし/未確認" })),
        Line::raw(format!("設定   : {}", app.config_path.display())),
        Line::raw(format!("出力先 : {}", app.cfg.output_root().display())),
        Line::raw(""),
        Line::styled("何かキーを押すと閉じます", Style::new().fg(DIM)),
    ];
    if t.ytdlp.is_none() || t.ffmpeg.is_none() {
        lines.insert(0, Line::styled("⚠ 必要なツールが不足しています (下の環境欄参照)", Style::new().fg(ERR).bold()));
    }
    f.render_widget(
        Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .block(Block::bordered().border_type(BorderType::Double).border_style(Style::new().fg(ACCENT)).title(" ヘルプ ")),
        r,
    );
}

fn draw_confirm(f: &mut Frame, app: &App, area: Rect) {
    let r = popup_area(area, 80, 5);
    f.render_widget(Clear, r);
    f.render_widget(
        Paragraph::new(vec![
            Line::raw(format!("実行中のジョブが {} 件あります。", app.running_count())),
            Line::styled("中止して終了しますか? (y / n)", Style::new().fg(WARN).bold()),
        ])
        .wrap(Wrap { trim: true })
        .block(Block::bordered().border_type(BorderType::Double).border_style(Style::new().fg(ERR)).title(" 終了確認 ")),
        r,
    );
}
