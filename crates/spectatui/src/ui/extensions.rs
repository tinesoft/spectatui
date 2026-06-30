use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Clear, Paragraph, Wrap};
use ratatui::Frame;

use spectatui_core::speckit::{ExtensionInfo, InstallStatus};

use crate::app::App;

pub fn draw(frame: &mut Frame, app: &App) {
    let theme = &app.theme;

    let full = frame.area();
    let w = full.width.saturating_sub(6).clamp(40, 108);
    let h = 30u16.min(full.height.saturating_sub(6)).max(8);

    let area = centered(w, h, full);
    frame.render_widget(Clear, area);

    let title_text = if app.filter_query.is_empty() {
        "Extensions".to_string()
    } else {
        format!(
            "Extensions  ·  {}/{}",
            app.filtered_extensions().len(),
            app.project.extensions.len()
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

    if inner.width < 10 || inner.height < 5 {
        return;
    }

    // Content starts one row below the title, leaving a blank line.
    let body = Rect::new(inner.x, inner.y + 1, inner.width, inner.height.saturating_sub(2));
    let cols =
        Layout::horizontal([Constraint::Percentage(45), Constraint::Percentage(55)]).split(body);
    draw_list(frame, app, cols[0], cols[1]);

    if area.height > 2 {
        let footer_y = area.y + area.height - 2;
        let footer_area = Rect::new(area.x + 2, footer_y, area.width.saturating_sub(4), 1);
        super::draw_search_footer(
            frame,
            theme,
            footer_area,
            "extensions",
            app.filter_active,
            &app.filter_query,
        );
    }
}

pub(super) fn draw_list(frame: &mut Frame, app: &App, list_area: Rect, detail_area: Rect) {
    let theme = &app.theme;
    let mut lines: Vec<Line> = Vec::new();

    let items = app.filtered_extensions();
    let per_page = list_area.height as usize;
    let offset = super::scroll_offset(app.ext_index, per_page);
    for (i, ext) in items.iter().enumerate().skip(offset).take(per_page) {
        let selected = i == app.ext_index;
        let row_style = if selected {
            Style::default().fg(theme.sel_fg).bg(theme.sel)
        } else {
            Style::default().fg(theme.fg)
        };

        let sel_bar = if selected {
            Span::styled("▌", theme.accent_style)
        } else {
            Span::raw(" ")
        };

        // Priority and version are right-aligned columns; "—" marks no priority.
        let pri = ext
            .priority
            .map(|p| format!("p{p}"))
            .unwrap_or_else(|| "—".to_string());
        let ver = format!("v{}", ext.version);
        let list_w = list_area.width as usize;
        let pad = list_w.saturating_sub(1 + 2 + ext.name.chars().count() + 5 + ver.chars().count() + 1);

        lines.push(Line::from(vec![
            sel_bar,
            status_dot(ext.status, theme),
            Span::styled(ext.name.clone(), row_style),
            Span::styled(" ".repeat(pad), theme.base),
            Span::styled(format!("{pri:<5}"), theme.dim_style),
            Span::styled(ver, theme.faint_style),
        ]));

        let row_y = list_area.y + (i - offset) as u16;
        if row_y < list_area.y + list_area.height {
            app.register_click(
                Rect::new(list_area.x, row_y, list_area.width, 1),
                crate::app::ClickAction::SelectExt(i),
            );
        }
    }

    if app.project.extensions.is_empty() {
        lines.push(Line::from(Span::styled(
            " No extensions installed",
            theme.faint_style,
        )));
    }

    let list = Paragraph::new(lines).style(theme.base);
    frame.render_widget(list, list_area);

    if let Some(&ext) = items.get(app.ext_index) {
        draw_detail(frame, app, detail_area, ext);
    } else if !app.filter_query.is_empty() {
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                format!(" No matches for \"{}\"", app.filter_query),
                theme.dim_style,
            ))),
            detail_area,
        );
    }
}

fn draw_detail(frame: &mut Frame, app: &App, area: Rect, ext: &ExtensionInfo) {
    let theme = &app.theme;
    let (status_icon, status_col) = match ext.status {
        InstallStatus::Enabled => ("● ", theme.good_style),
        InstallStatus::Disabled => ("◐ ", theme.warn_style),
        InstallStatus::Available => ("○ ", theme.faint_style),
    };
    let status_text = match ext.status {
        InstallStatus::Enabled => "enabled",
        InstallStatus::Disabled => "disabled",
        InstallStatus::Available => "available",
    };

    let mut lines = vec![
        Line::from(vec![
            Span::styled(format!(" {} ", ext.name), theme.accent_bold),
            Span::styled(" ·  id · ", theme.faint_style),
            Span::styled(ext.id.clone(), theme.dim_style),
            Span::styled(format!(" v{}", ext.version), theme.dim_style),
        ]),
        Line::default(),
        Line::from(vec![
            Span::styled(format!(" {status_icon}"), status_col),
            Span::styled(status_text, status_col),
            Span::styled(
                ext.priority
                    .map(|p| format!("  priority {p}"))
                    .unwrap_or_else(|| "  not installed".to_string()),
                theme.dim_style,
            ),
        ]),
    ];

    if let Some(author) = &ext.author {
        let source_text = match &ext.source {
            spectatui_core::speckit::ExtensionSource::Catalog(name) => format!("catalog · {name}"),
            spectatui_core::speckit::ExtensionSource::Local => "local".to_string(),
            _ => String::new(),
        };
        lines.push(Line::from(Span::styled(
            format!(" by {author}  ·  {source_text}"),
            theme.dim_style,
        )));
    }

    lines.push(Line::from(Span::styled(
        format!(" {} commands", ext.command_count),
        theme.info_style,
    )));
    lines.push(Line::default());

    if !ext.description.is_empty() {
        lines.push(Line::from(Span::styled(
            format!(" {}", ext.description),
            Style::default().fg(theme.fg),
        )));
        lines.push(Line::default());
    }

    lines.push(Line::from(Span::styled(" Actions", theme.dim_style)));

    match ext.status {
        InstallStatus::Enabled => {
            lines.push(action_line("x", "remove", theme));
            lines.push(action_line("d", "disable", theme));
            lines.push(action_line("p", "set-priority", theme));
            lines.push(action_line("u", "update", theme));
        }
        InstallStatus::Disabled => {
            lines.push(action_line("x", "remove", theme));
            lines.push(action_line("e", "enable", theme));
            lines.push(action_line("p", "set-priority", theme));
        }
        InstallStatus::Available => {
            lines.push(action_line("a", "add", theme));
        }
    }

    let detail = Paragraph::new(lines)
        .wrap(Wrap { trim: false })
        .style(theme.base);
    frame.render_widget(detail, area);
}

fn status_dot(status: InstallStatus, theme: &crate::theme::Theme) -> Span<'static> {
    match status {
        InstallStatus::Enabled => Span::styled("● ", theme.good_style),
        InstallStatus::Disabled => Span::styled("◐ ", theme.warn_style),
        InstallStatus::Available => Span::styled("○ ", theme.faint_style),
    }
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
