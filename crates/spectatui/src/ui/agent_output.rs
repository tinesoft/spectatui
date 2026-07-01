use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph};
use ratatui::Frame;

use spectatui_core::tmux::SessionStatus;

use crate::app::{App, Pane};

pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let focused = app.focused_pane == Pane::AgentOutput;
    let border_style = if focused {
        theme.border_focused
    } else {
        theme.border_unfocused
    };
    let title_style = if focused {
        theme.title_focused
    } else {
        theme.title_unfocused
    };

    let agent_name = app.default_agent_name();
    let title = Line::from(vec![
        Span::styled("─┤ ", border_style),
        Span::styled(format!("Agent · {agent_name}"), title_style),
        Span::styled(" ├", border_style),
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(border_style)
        .title(title)
        .padding(super::PANEL_PADDING);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let (status_dot, status_text, status_style) = match app.tmux_session.as_ref().map(|s| s.status) {
        Some(SessionStatus::Running) => ("●", "running", theme.good_style),
        Some(SessionStatus::Idle) => ("○", "idle", theme.faint_style),
        Some(SessionStatus::Exited) => ("○", "exited", theme.bad_style),
        _ => ("○", "no session", theme.faint_style),
    };

    // Render status indicator on the right side of the title bar
    let status_str = format!("{status_dot} {status_text}");
    let status_x = area.x + area.width.saturating_sub(status_str.len() as u16 + 2);
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(format!("{status_dot} "), status_style),
            Span::styled(status_text, status_style),
        ])),
        Rect::new(status_x, area.y, status_str.len() as u16, 1),
    );

    let mut lines: Vec<Line> = Vec::new();

    if let Some(session) = &app.tmux_session {
        let visible_lines = inner.height.saturating_sub(3) as usize;
        let snapshot = &session.last_snapshot;
        let start = snapshot.len().saturating_sub(visible_lines);
        for line in &snapshot[start..] {
            lines.push(Line::from(Span::styled(
                line.to_string(),
                theme.dim_style,
            )));
        }
    } else if !app.agent_lines.is_empty() {
        let visible_lines = inner.height.saturating_sub(3) as usize;
        let start = app.agent_lines.len().saturating_sub(visible_lines);
        for line in &app.agent_lines[start..] {
            lines.push(Line::from(Span::styled(
                line.to_string(),
                theme.dim_style,
            )));
        }
    } else {
        lines.push(Line::from(Span::styled(
            "No active session for this feature.",
            theme.dim_style,
        )));
        lines.push(Line::default());
        lines.push(Line::from(vec![
            Span::styled("Press ", theme.faint_style),
            Span::styled("[enter]", theme.accent_bold),
            Span::styled(" to start a coding-agent session.", theme.faint_style),
        ]));
    }

    // Footer hints
    let remaining = inner.height.saturating_sub(lines.len() as u16 + 1);
    for _ in 0..remaining {
        lines.push(Line::default());
    }
    lines.push(Line::from(vec![
        Span::styled("[", theme.faint_style),
        Span::styled("a", theme.accent_bold),
        Span::styled("] attach", theme.faint_style),
    ]));

    let content = Paragraph::new(lines).style(theme.base);
    frame.render_widget(content, inner);
}
