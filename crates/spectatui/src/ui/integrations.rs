use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Clear, Paragraph, Wrap};
use ratatui::Frame;

use crate::app::App;

pub fn draw(frame: &mut Frame, app: &App) {
    let theme = &app.theme;

    let full = frame.area();
    let w = full.width.saturating_sub(6).clamp(40, 108);
    let h = 30u16.min(full.height.saturating_sub(6)).max(8);

    let area = centered(w, h, full);
    frame.render_widget(Clear, area);

    let title_text = if app.filter_query.is_empty() {
        "AI Integrations".to_string()
    } else {
        format!(
            "AI Integrations  ·  {}/{}",
            app.filtered_integrations().len(),
            app.project.integrations.len()
        )
    };
    let title = Line::from(vec![
        Span::styled("─┤ ", theme.border_focused),
        Span::styled(title_text, theme.accent_bold),
        Span::styled(" ├", theme.border_focused),
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme.border_focused)
        .title(title);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.width < 10 || inner.height < 3 {
        return;
    }

    let list_w = (inner.width as f32 * 0.42) as u16;
    let cols = Layout::horizontal([Constraint::Length(list_w), Constraint::Min(0)]).split(inner);

    // Content starts one row below the title, leaving a blank line.
    let list_area = Rect::new(cols[0].x, cols[0].y + 1, cols[0].width, cols[0].height.saturating_sub(1));
    draw_list(frame, app, list_area);

    // Vertical divider
    for y in cols[1].y..cols[1].y + cols[1].height {
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled("│", theme.border_focused))),
            Rect::new(cols[1].x, y, 1, 1),
        );
    }

    let detail_area = Rect::new(cols[1].x + 1, cols[1].y + 1, cols[1].width.saturating_sub(1), cols[1].height.saturating_sub(1));
    draw_detail(frame, app, detail_area);

    // Footer
    if area.height > 2 {
        let footer_y = area.y + area.height - 2;
        let footer_area = Rect::new(area.x + 3, footer_y, area.width.saturating_sub(6), 1);
        super::draw_search_footer(
            frame,
            theme,
            footer_area,
            "integrations",
            app.filter_active,
            &app.filter_query,
        );
    }
}

fn draw_list(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let mut lines: Vec<Line> = Vec::new();

    let items = app.filtered_integrations();
    if app.project.integrations.is_empty() {
        lines.push(Line::from(Span::styled(
            " No integrations found",
            theme.faint_style,
        )));
    } else {
        let col_w = area.width as usize;
        let right_max = 13; // "★ default" = 9, "available" = 9, padded
        let name_max = col_w.saturating_sub(right_max + 4); // sel_bar + dot + padding

        for (i, intg) in items.iter().enumerate() {
            let selected = i == app.integration_index;
            let bg = if selected { Some(theme.sel) } else { None };
            let row_style = if selected {
                Style::default().fg(theme.sel_fg).bg(theme.sel)
            } else {
                Style::default().fg(if intg.installed { theme.fg } else { theme.dim })
            };

            let sel_bar = if selected {
                Span::styled("▌", theme.accent_style)
            } else {
                Span::raw(" ")
            };

            let dot = if intg.installed {
                Span::styled("● ", theme.good_style)
            } else {
                Span::styled("○ ", theme.faint_style)
            };

            let name = if intg.name.chars().count() > name_max {
                let truncated: String =
                    intg.name.chars().take(name_max.saturating_sub(1)).collect();
                format!("{truncated}…")
            } else {
                intg.name.clone()
            };

            let right_text = if intg.is_default {
                "★ default".to_string()
            } else if intg.installed {
                intg.version
                    .as_deref()
                    .map(|v| format!("v{v}"))
                    .unwrap_or_default()
            } else {
                "available".to_string()
            };
            let right_style = if intg.is_default {
                theme.accent_style
            } else if intg.installed {
                theme.faint_style
            } else {
                theme.warn_style
            };

            let name_len = name.chars().count();
            let right_len = right_text.chars().count();
            let pad = col_w.saturating_sub(1 + 2 + name_len + right_len + 1);

            let pad_style = if let Some(bg) = bg {
                Style::default().bg(bg)
            } else {
                theme.base
            };

            lines.push(Line::from(vec![
                sel_bar,
                dot,
                Span::styled(name, row_style),
                Span::styled(" ".repeat(pad), pad_style),
                Span::styled(right_text, right_style),
            ]));

            let row_y = area.y + i as u16;
            if row_y < area.y + area.height {
                app.register_click(
                    Rect::new(area.x, row_y, area.width, 1),
                    crate::app::ClickAction::SelectIntegration(i),
                );
            }
        }
    }

    let list = Paragraph::new(lines).style(theme.base);
    frame.render_widget(list, area);
}

fn draw_detail(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;

    let items = app.filtered_integrations();
    let Some(intg) = items.get(app.integration_index) else {
        if !app.filter_query.is_empty() {
            frame.render_widget(
                Paragraph::new(Line::from(Span::styled(
                    format!(" No matches for \"{}\"", app.filter_query),
                    theme.dim_style,
                ))),
                area,
            );
        }
        return;
    };

    let mut name_spans = vec![
        Span::styled(format!(" {} ", intg.name), theme.accent_bold),
        Span::styled(
            intg.version
                .as_deref()
                .map(|v| format!("v{v}"))
                .unwrap_or_default(),
            theme.dim_style,
        ),
    ];
    if intg.is_default {
        name_spans.push(Span::styled("  ★ default", theme.accent_style));
    }
    let mut lines = vec![Line::from(name_spans), Line::default()];

    let (status_dot, status_text, status_style) = if intg.installed {
        ("● ", "installed", theme.good_style)
    } else {
        ("○ ", "not installed", theme.warn_style)
    };
    let cli_text = if intg.cli_required {
        "· CLI tool required"
    } else {
        "· IDE-based"
    };
    lines.push(Line::from(vec![
        Span::styled(format!(" {status_dot}"), status_style),
        Span::styled(status_text, status_style),
        Span::styled(format!(" {cli_text}"), theme.dim_style),
    ]));
    lines.push(Line::default());

    if !intg.description.is_empty() {
        lines.push(Line::from(Span::styled(
            format!(" {}", intg.description),
            Style::default().fg(theme.fg),
        )));
        lines.push(Line::default());
    }

    lines.push(Line::from(Span::styled(" Actions", theme.dim_style)));

    if !intg.installed {
        lines.push(action_line("i", "install", theme));
        lines.push(action_line("s", "switch to this", theme));
        lines.push(action_line("n", "info", theme));
    } else if intg.is_default {
        lines.push(action_line("g", "upgrade", theme));
        lines.push(action_line("x", "uninstall", theme));
        lines.push(action_line("v", "status · drift-check", theme));
        lines.push(action_line("n", "info", theme));
    } else {
        lines.push(action_line("d", "use as default", theme));
        lines.push(action_line("s", "switch to this", theme));
        lines.push(action_line("g", "upgrade", theme));
        lines.push(action_line("x", "uninstall", theme));
        lines.push(action_line("v", "status · drift-check", theme));
        lines.push(action_line("n", "info", theme));
    }

    let detail = Paragraph::new(lines)
        .wrap(Wrap { trim: false })
        .style(theme.base);
    frame.render_widget(detail, area);
}

fn action_line<'a>(key: &str, desc: &str, theme: &crate::theme::Theme) -> Line<'a> {
    Line::from(vec![
        Span::styled(format!("   [{key}] "), theme.accent_bold),
        Span::styled(desc.to_string(), theme.dim_style),
    ])
}

fn centered(w: u16, h: u16, r: Rect) -> Rect {
    let x = r.x + (r.width.saturating_sub(w)) / 2;
    let y = r.y + (r.height.saturating_sub(h)) / 2;
    Rect::new(x, y, w.min(r.width), h.min(r.height))
}
