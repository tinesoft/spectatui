use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph};
use ratatui::Frame;

use spectatui_core::tmux::SessionStatus;

use crate::app::App;

pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let full = area;

    let feature = app.selected_feature();
    let feature_id = feature.map(|f| f.id.as_str()).unwrap_or("none");
    let branch = feature.and_then(|f| f.branch.as_deref()).unwrap_or("");
    let agent_name = app.default_agent_name();

    let is_running = match app.tmux_session.as_ref().map(|s| s.status) {
        Some(SessionStatus::Running) => true,
        _ => false,
    };

    // Top bar (row 0)
    let mut top_spans = vec![
        Span::styled(" ", theme.header_style),
        Span::styled("● ", if is_running { theme.good_style } else { theme.faint_style }),
        Span::styled("attached", ratatui::style::Style::default().fg(if is_running { theme.good } else { theme.faint }).bg(theme.header_bg).add_modifier(ratatui::style::Modifier::BOLD)),
        Span::styled(format!("  {feature_id}  ·  {agent_name}  ·  tmux"), ratatui::style::Style::default().fg(theme.dim).bg(theme.header_bg)),
    ];

    let right_text = "Ctrl-b d detach  ·  esc back to spectatui";
    let left_width: u16 = top_spans.iter().map(|s| s.width() as u16).sum();
    let pad = full.width.saturating_sub(left_width + right_text.len() as u16 + 1);
    top_spans.push(Span::styled(" ".repeat(pad as usize), theme.header_style));
    top_spans.push(Span::styled(right_text, ratatui::style::Style::default().fg(theme.dim).bg(theme.header_bg)));

    let top_bar = Paragraph::new(Line::from(top_spans)).style(theme.header_style);
    frame.render_widget(top_bar, Rect::new(full.x, full.y, full.width, 1));

    // Separator (row 1)
    let sep = Paragraph::new(Line::from(Span::styled(
        "─".repeat(full.width as usize),
        theme.faint_style,
    )));
    frame.render_widget(sep, Rect::new(full.x, full.y + 1, full.width, 1));

    // Transcript area (rows 3 to h-7)
    let transcript_y = full.y + 3;
    let input_y = full.height.saturating_sub(6);
    let transcript_h = input_y.saturating_sub(transcript_y);

    if let Some(session) = &app.tmux_session {
        let visible = transcript_h as usize;
        let snapshot = &session.last_snapshot;
        let start = snapshot.len().saturating_sub(visible);
        for (i, line) in snapshot[start..].iter().enumerate() {
            let y = transcript_y + i as u16;
            if y >= input_y {
                break;
            }
            frame.render_widget(
                Paragraph::new(Line::from(Span::styled(
                    format!("  {line}"),
                    theme.dim_style,
                ))),
                Rect::new(full.x, y, full.width, 1),
            );
        }
    } else {
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                "  No active session. Press esc to return to the dashboard.",
                theme.faint_style,
            ))),
            Rect::new(full.x, transcript_y, full.width, 1),
        );
    }

    // Input box
    let input_box_y = full.height.saturating_sub(6);
    let input_area = Rect::new(full.x + 2, input_box_y, full.width.saturating_sub(4), 3);

    let input_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme.border_focused);
    frame.render_widget(input_block, input_area);

    let input_line = if app.attach_input.is_empty() {
        Line::from(vec![
            Span::styled("›", theme.accent_bold),
            Span::styled(" type a follow-up + enter, or enter to ", theme.faint_style),
            Span::styled("attach", theme.accent_bold),
            Span::styled(" · ", theme.faint_style),
            Span::styled("esc", theme.accent_bold),
            Span::styled(" to detach", theme.faint_style),
        ])
    } else {
        Line::from(vec![
            Span::styled("› ", theme.accent_bold),
            Span::styled(
                &app.attach_input,
                ratatui::style::Style::default().fg(theme.fg),
            ),
            Span::styled("▏", theme.accent_bold),
        ])
    };
    frame.render_widget(
        Paragraph::new(input_line),
        Rect::new(input_area.x + 2, input_box_y + 1, input_area.width.saturating_sub(4), 1),
    );

    // Status bar (ROWS-2)
    let status_y = full.height.saturating_sub(2);
    let mut status_spans = vec![
        Span::styled(" ", theme.statusbar_style),
        Span::styled(agent_name, ratatui::style::Style::default().fg(theme.fg).bg(theme.panel).add_modifier(ratatui::style::Modifier::BOLD)),
        Span::styled(format!("  ·  {branch}"), ratatui::style::Style::default().fg(theme.info).bg(theme.panel)),
    ];
    let session_text = if is_running { "● session live in tmux" } else { "○ no session" };
    let session_style = if is_running { theme.good_style } else { theme.faint_style };
    let status_left_w: u16 = status_spans.iter().map(|s| s.width() as u16).sum();
    let status_pad = full.width.saturating_sub(status_left_w + session_text.len() as u16 + 1);
    status_spans.push(Span::styled(" ".repeat(status_pad as usize), theme.statusbar_style));
    status_spans.push(Span::styled(session_text, session_style));

    let status_bar = Paragraph::new(Line::from(status_spans)).style(theme.statusbar_style);
    frame.render_widget(status_bar, Rect::new(full.x, status_y, full.width, 1));

    // Key hints (ROWS-1)
    let hints_y = full.height.saturating_sub(1);
    let hints = vec![
        ("Ctrl-b d", "detach (keeps running)"),
        ("esc", "back to dashboard"),
        ("↑↓", "scroll"),
    ];
    let mut hint_spans: Vec<Span> = vec![Span::styled(" ", theme.base)];
    for (i, (key, desc)) in hints.iter().enumerate() {
        if i > 0 {
            hint_spans.push(Span::styled("  ·  ", theme.faint_style));
        }
        hint_spans.push(Span::styled(key.to_string(), theme.accent_bold));
        hint_spans.push(Span::styled(format!(" {desc}"), theme.dim_style));
    }
    let hint_bar = Paragraph::new(Line::from(hint_spans)).style(theme.base);
    frame.render_widget(hint_bar, Rect::new(full.x, hints_y, full.width, 1));
}
