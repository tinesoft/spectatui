use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Clear, Paragraph, Wrap};
use ratatui::Frame;

use spectatui_core::speckit::{InstallStatus, PresetInfo};

use crate::app::App;

pub fn draw(frame: &mut Frame, app: &App) {
    let theme = &app.theme;

    let full = frame.area();
    let w = full.width.saturating_sub(6).clamp(40, 108);
    let h = 30u16.min(full.height.saturating_sub(6)).max(8);

    let area = centered(w, h, full);
    frame.render_widget(Clear, area);

    let title_text = if app.filter_query.is_empty() {
        "Presets".to_string()
    } else {
        format!(
            "Presets  ·  {}/{}",
            app.filtered_presets().len(),
            app.project.presets.len()
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
        .title(title)
        .padding(super::PANEL_PADDING);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.width < 10 || inner.height < 5 {
        return;
    }

    let list_w = (inner.width as f32 * 0.46) as u16;
    let cols = Layout::horizontal([Constraint::Length(list_w), Constraint::Min(0)]).split(inner);

    // Content starts one row below the title, leaving a blank line; height reserves the
    // footer row plus the 2-row gap above it.
    let list_area = Rect::new(
        cols[0].x,
        cols[0].y + 1,
        cols[0].width,
        cols[0].height.saturating_sub(3),
    );

    // Vertical divider between the list and the detail column.
    for y in cols[1].y..cols[1].y + cols[1].height {
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled("│", theme.border_focused))),
            Rect::new(cols[1].x, y, 1, 1),
        );
    }

    let detail_area = Rect::new(
        cols[1].x + 1,
        cols[1].y + 1,
        cols[1].width.saturating_sub(1),
        cols[1].height.saturating_sub(1),
    );
    draw_list(frame, app, list_area, detail_area);

    if area.height > 2 {
        let footer_y = area.y + area.height - 2;
        let footer_area = Rect::new(area.x + 2, footer_y, area.width.saturating_sub(4), 1);
        super::draw_search_footer(
            frame,
            theme,
            footer_area,
            "presets",
            app.filter_active,
            &app.filter_query,
        );
    }
}

pub(super) fn draw_list(frame: &mut Frame, app: &App, list_area: Rect, detail_area: Rect) {
    let theme = &app.theme;
    let mut lines: Vec<Line> = Vec::new();

    let items = app.filtered_presets();
    let per_page = list_area.height as usize;
    let offset = super::scroll_offset(app.preset_index, per_page);
    for (i, preset) in items.iter().enumerate().skip(offset).take(per_page) {
        let selected = i == app.preset_index;
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
        let pri = preset
            .priority
            .map(|p| format!("p{p}"))
            .unwrap_or_else(|| "—".to_string());
        let ver = format!("v{}", preset.version);
        let list_w = list_area.width as usize;
        let pad = list_w
            .saturating_sub(1 + 2 + preset.name.chars().count() + 5 + ver.chars().count() + 1);

        lines.push(Line::from(vec![
            sel_bar,
            status_dot(preset.status, theme),
            Span::styled(preset.name.clone(), row_style),
            Span::styled(" ".repeat(pad), theme.base),
            Span::styled(format!("{pri:<5}"), theme.dim_style),
            Span::styled(ver, theme.faint_style),
        ]));

        let row_y = list_area.y + (i - offset) as u16;
        if row_y < list_area.y + list_area.height {
            app.register_click(
                Rect::new(list_area.x, row_y, list_area.width, 1),
                crate::app::ClickAction::SelectPreset(i),
            );
        }
    }

    if app.project.presets.is_empty() {
        lines.extend(super::empty_list_lines(app, "No presets installed"));
    }

    let list = Paragraph::new(lines).style(theme.base);
    frame.render_widget(list, list_area);

    if let Some(&preset) = items.get(app.preset_index) {
        draw_detail(frame, app, detail_area, preset);
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

fn draw_detail(frame: &mut Frame, app: &App, area: Rect, preset: &PresetInfo) {
    let theme = &app.theme;
    let (status_icon, status_col) = match preset.status {
        InstallStatus::Enabled => ("● ", theme.good_style),
        InstallStatus::Disabled => ("◐ ", theme.warn_style),
        InstallStatus::Available => ("○ ", theme.faint_style),
    };
    let status_text = match preset.status {
        InstallStatus::Enabled => "enabled",
        InstallStatus::Disabled => "disabled",
        InstallStatus::Available => "available",
    };

    let mut lines = vec![
        Line::from(vec![
            Span::styled(format!(" {} ", preset.name), theme.accent_bold),
            Span::styled(" ·  id · ", theme.faint_style),
            Span::styled(preset.id.clone(), theme.dim_style),
            Span::styled(format!(" v{}", preset.version), theme.dim_style),
        ]),
        Line::default(),
        Line::from(vec![
            Span::styled(format!(" {status_icon}"), status_col),
            Span::styled(status_text, status_col),
            Span::styled(
                preset
                    .priority
                    .map(|p| format!("  priority {p}"))
                    .unwrap_or_else(|| "  not installed".to_string()),
                theme.dim_style,
            ),
        ]),
    ];

    if let Some(author) = &preset.author {
        let source_text = preset.source_label.clone().unwrap_or_default();
        lines.push(Line::from(Span::styled(
            format!(" by {author}  ·  {source_text}"),
            theme.dim_style,
        )));
    }

    lines.push(Line::from(Span::styled(
        format!(" {} templates", preset.template_count),
        theme.info_style,
    )));
    lines.push(Line::default());

    if !preset.description.is_empty() {
        lines.push(Line::from(Span::styled(
            format!(" {}", preset.description),
            Style::default().fg(theme.fg),
        )));
        lines.push(Line::default());
    }

    lines.push(Line::from(Span::styled(" Actions", theme.dim_style)));

    match preset.status {
        InstallStatus::Enabled => {
            lines.push(action_line("x", "remove", theme));
            lines.push(action_line("d", "disable", theme));
            lines.push(action_line("p", "set-priority", theme));
            lines.push(action_line("r", "resolve", theme));
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
