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
        "Automation Workflows".to_string()
    } else {
        format!(
            "Automation Workflows  ·  {}/{}",
            app.filtered_workflows().len(),
            app.project.workflows.len()
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

    let list_w = (inner.width as f32 * 0.44) as u16;
    let cols = Layout::horizontal([Constraint::Length(list_w), Constraint::Min(0)]).split(inner);

    // Content starts one row below the title, leaving a blank line.
    let list_area = Rect::new(cols[0].x, cols[0].y + 1, cols[0].width, cols[0].height.saturating_sub(1));
    draw_list(frame, app, list_area);

    for y in cols[1].y..cols[1].y + cols[1].height {
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled("│", theme.border_focused))),
            Rect::new(cols[1].x, y, 1, 1),
        );
    }

    let detail_area = Rect::new(cols[1].x + 1, cols[1].y + 1, cols[1].width.saturating_sub(1), cols[1].height.saturating_sub(1));
    draw_detail(frame, app, detail_area);

    if area.height > 2 {
        let footer_y = area.y + area.height - 2;
        let footer_area = Rect::new(area.x + 3, footer_y, area.width.saturating_sub(6), 1);
        super::draw_search_footer(
            frame,
            theme,
            footer_area,
            "workflows",
            app.filter_active,
            &app.filter_query,
        );
    }
}

fn draw_list(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let mut lines: Vec<Line> = Vec::new();

    let items = app.filtered_workflows();
    if app.project.workflows.is_empty() {
        lines.push(Line::from(Span::styled(
            " No workflows found",
            theme.faint_style,
        )));
    } else {
        let name_max = area.width.saturating_sub(10) as usize;

        for (i, wf) in items.iter().enumerate() {
            let selected = i == app.wf_index;
            let row_style = if selected {
                Style::default().fg(theme.sel_fg).bg(theme.sel)
            } else {
                Style::default().fg(if wf.installed { theme.fg } else { theme.dim })
            };

            let sel_bar = if selected {
                Span::styled("▌", theme.accent_style)
            } else {
                Span::raw(" ")
            };

            let dot = if wf.installed {
                Span::styled("● ", theme.good_style)
            } else {
                Span::styled("○ ", theme.faint_style)
            };

            let display_name = wf.name.as_deref().unwrap_or(&wf.id);
            let name = if display_name.chars().count() > name_max {
                let truncated: String =
                    display_name.chars().take(name_max.saturating_sub(1)).collect();
                format!("{truncated}…")
            } else {
                display_name.to_string()
            };

            let version_text = if wf.installed {
                wf.version
                    .as_deref()
                    .map(|v| format!("v{v}"))
                    .unwrap_or_default()
            } else {
                "available".to_string()
            };
            let version_style = if wf.installed {
                theme.faint_style
            } else {
                theme.warn_style
            };

            lines.push(Line::from(vec![
                sel_bar,
                dot,
                Span::styled(name, row_style),
                Span::raw(" "),
                Span::styled(version_text, version_style),
            ]));

            let row_y = area.y + i as u16;
            if row_y < area.y + area.height {
                app.register_click(
                    Rect::new(area.x, row_y, area.width, 1),
                    crate::app::ClickAction::SelectWorkflow(i),
                );
            }
        }
    }

    let list = Paragraph::new(lines).style(theme.base);
    frame.render_widget(list, area);
}

fn draw_detail(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;

    let items = app.filtered_workflows();
    let Some(wf) = items.get(app.wf_index) else {
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

    let display_name = wf.name.as_deref().unwrap_or(&wf.id);
    let mut name_spans = vec![Span::styled(format!(" {display_name} "), theme.accent_bold)];
    if let Some(v) = &wf.version {
        name_spans.push(Span::styled(format!("v{v}"), theme.dim_style));
    }
    let mut lines = vec![Line::from(name_spans), Line::default()];

    let (status_dot, status_text, status_style) = if wf.installed {
        ("● ", "installed", theme.good_style)
    } else {
        ("○ ", "not installed", theme.warn_style)
    };
    let mut status_spans = vec![
        Span::styled(format!(" {status_dot}"), status_style),
        Span::styled(status_text, status_style),
    ];
    if let Some(source) = &wf.source {
        status_spans.push(Span::styled(format!(" · {source}"), theme.dim_style));
    }
    lines.push(Line::from(status_spans));

    if let Some(last_run) = &wf.last_run {
        lines.push(Line::from(Span::styled(
            format!(" {last_run}"),
            theme.info_style,
        )));
    }
    lines.push(Line::default());

    if !wf.description.is_empty() {
        lines.push(Line::from(Span::styled(
            format!(" {}", wf.description),
            Style::default().fg(theme.fg),
        )));
        lines.push(Line::default());
    }

    lines.push(Line::from(Span::styled(" Actions", theme.dim_style)));

    if wf.installed {
        lines.push(action_line("r", "run", theme));
        lines.push(action_line("R", "resume last run", theme));
        lines.push(action_line("s", "status / run history", theme));
        lines.push(action_line("i", "info", theme));
        lines.push(action_line("x", "remove", theme));
    } else {
        lines.push(action_line("a", "add", theme));
        lines.push(action_line("i", "info", theme));
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
