use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Clear, Paragraph};
use ratatui::Frame;

use crate::app::{palette_commands, App};

pub fn draw(frame: &mut Frame, app: &App) {
    let Some(palette) = &app.palette else {
        return;
    };

    let theme = &app.theme;

    let commands = palette_commands();
    let filtered: Vec<_> = commands
        .iter()
        .filter(|c| {
            palette.input.is_empty()
                || c.label
                    .to_lowercase()
                    .contains(&palette.input.to_lowercase())
        })
        .collect();

    let h = (filtered.len() as u16 + 4).min(frame.area().height);
    let area = fixed_centered_rect(64, h, frame.area());
    frame.render_widget(Clear, area);

    let title = Line::from(vec![
        Span::styled("┤ ", theme.border_focused),
        Span::styled("Command Palette", theme.title_focused),
        Span::styled(" ├", theme.border_focused),
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme.border_focused)
        .title(title);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let rows = Layout::vertical([Constraint::Length(1), Constraint::Length(1), Constraint::Min(0)])
        .split(inner);

    // Input line
    let input_line = if palette.input.is_empty() {
        Line::from(vec![
            Span::styled(" › ", theme.accent_style),
            Span::styled("type to filter…", theme.faint_style),
        ])
    } else {
        Line::from(vec![
            Span::styled(" › ", theme.accent_style),
            Span::styled(&palette.input, Style::default().fg(theme.fg)),
            Span::styled("│", theme.accent_style),
        ])
    };
    frame.render_widget(Paragraph::new(input_line).style(theme.base), rows[0]);

    // Separator
    let sep = Line::from(Span::styled(
        "─".repeat(inner.width as usize),
        theme.faint_style,
    ));
    frame.render_widget(Paragraph::new(sep), rows[1]);

    // Commands
    let mut lines: Vec<Line> = Vec::new();
    for (i, cmd) in filtered.iter().enumerate() {
        let selected = i == palette.selected;
        let row_y = rows[2].y + i as u16;
        if row_y < rows[2].y + rows[2].height {
            app.register_click(
                Rect::new(rows[2].x, row_y, rows[2].width, 1),
                crate::app::ClickAction::PaletteRun(i),
            );
        }
        let style = if selected {
            Style::default().fg(theme.sel_fg).bg(theme.sel)
        } else {
            theme.dim_style
        };
        let sel_bar = if selected {
            Span::styled(" ▌", theme.accent_style)
        } else {
            Span::raw("  ")
        };
        let hint = cmd.hint;
        if !hint.is_empty() {
            let label_len = cmd.label.chars().count() + 3;
            let avail = rows[2].width as usize;
            let pad = avail.saturating_sub(label_len + hint.chars().count() + 1);
            lines.push(Line::from(vec![
                sel_bar,
                Span::styled(format!(" {}", cmd.label), style),
                Span::styled(" ".repeat(pad), style),
                Span::styled(hint.to_string(), theme.faint_style),
            ]));
        } else {
            lines.push(Line::from(vec![
                sel_bar,
                Span::styled(format!(" {}", cmd.label), style),
            ]));
        }
    }

    let list = Paragraph::new(lines).style(theme.base);
    frame.render_widget(list, rows[2]);
}

fn fixed_centered_rect(w: u16, h: u16, r: Rect) -> Rect {
    let x = r.x + (r.width.saturating_sub(w)) / 2;
    let y = r.y + (r.height.saturating_sub(h)) / 2;
    Rect::new(x, y, w.min(r.width), h.min(r.height))
}
