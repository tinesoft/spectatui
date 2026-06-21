use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph, Wrap};
use ratatui::Frame;

use spectatui_core::speckit::{ExtensionInfo, InstallStatus, PresetInfo};

use crate::app::{App, ExtTab};

/// Tabbed Extensions & Presets manager (full screen + audit pane).
pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    draw_impl(frame, app, area, None);
}

/// Single-category popup variant: no tab row, category-only title.
pub fn draw_single(frame: &mut Frame, app: &App, area: Rect, tab: ExtTab) {
    draw_impl(frame, app, area, Some(tab));
}

fn draw_impl(frame: &mut Frame, app: &App, area: Rect, single: Option<ExtTab>) {
    let theme = &app.theme;

    let ext_count = app.project.extensions.len();
    let preset_count = app.project.presets.len();
    let active_tab = single.unwrap_or(app.ext_tab);

    let title_line = if single.is_some() {
        let label = match active_tab {
            ExtTab::Extensions => "Extensions",
            ExtTab::Presets => "Presets",
        };
        Line::from(vec![
            Span::styled("┤ ", theme.border_focused),
            Span::styled(label, theme.accent_bold),
            Span::styled(" ├", theme.border_focused),
        ])
    } else {
        Line::from(vec![
            Span::styled("┤ ", theme.border_focused),
            tab_span(
                &format!("Extensions {ext_count}"),
                active_tab == ExtTab::Extensions,
                theme,
            ),
            Span::styled(" │ ", theme.faint_style),
            tab_span(
                &format!("Presets {preset_count}"),
                active_tab == ExtTab::Presets,
                theme,
            ),
            Span::styled(" ├", theme.border_focused),
        ])
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme.border_focused)
        .title(title_line);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Register clickable tab regions (tabbed mode only).
    if single.is_none() {
        let tx = area.x + 2;
        let ext_label = format!("Extensions {ext_count}");
        let preset_label = format!("Presets {preset_count}");
        app.register_click(
            Rect::new(tx, area.y, ext_label.len() as u16, 1),
            crate::app::ClickAction::SetExtTab(ExtTab::Extensions),
        );
        app.register_click(
            Rect::new(tx + ext_label.len() as u16 + 3, area.y, preset_label.len() as u16, 1),
            crate::app::ClickAction::SetExtTab(ExtTab::Presets),
        );
    }

    if inner.width < 10 || inner.height < 3 {
        return;
    }

    let cols =
        Layout::horizontal([Constraint::Percentage(45), Constraint::Percentage(55)]).split(inner);

    match active_tab {
        ExtTab::Extensions => draw_ext_list(frame, app, cols[0], cols[1]),
        ExtTab::Presets => draw_preset_list(frame, app, cols[0], cols[1]),
    }

    // Footer
    if area.height > 2 {
        let footer_y = area.y + area.height - 1;
        let footer_area = Rect::new(area.x + 2, footer_y, area.width.saturating_sub(4), 1);
        let is_compact = inner.width < 60;
        let footer_text = if is_compact {
            "[enter] manage"
        } else {
            "[/] search catalog   [c] catalog list"
        };
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(footer_text, theme.faint_style))),
            footer_area,
        );
    }
}

fn tab_span<'a>(label: &str, active: bool, theme: &crate::theme::Theme) -> Span<'a> {
    if active {
        Span::styled(label.to_string(), theme.accent_bold)
    } else {
        Span::styled(label.to_string(), theme.dim_style)
    }
}

fn status_dot(status: InstallStatus, theme: &crate::theme::Theme) -> Span<'static> {
    match status {
        InstallStatus::Enabled => Span::styled("● ", theme.good_style),
        InstallStatus::Disabled => Span::styled("◐ ", theme.warn_style),
        InstallStatus::Available => Span::styled("○ ", theme.faint_style),
    }
}

fn draw_ext_list(frame: &mut Frame, app: &App, list_area: Rect, detail_area: Rect) {
    let theme = &app.theme;
    let mut lines: Vec<Line> = Vec::new();

    for (i, ext) in app.project.extensions.iter().enumerate() {
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

        let priority = ext
            .priority
            .map(|p| format!("p{p} "))
            .unwrap_or_default();

        lines.push(Line::from(vec![
            sel_bar,
            status_dot(ext.status, theme),
            Span::styled(ext.id.clone(), row_style),
            Span::styled(format!(" {priority}"), theme.faint_style),
            Span::styled(format!("v{}", ext.version), theme.dim_style),
        ]));

        let row_y = list_area.y + i as u16;
        if row_y < list_area.y + list_area.height {
            app.register_click(
                Rect::new(list_area.x, row_y, list_area.width, 1),
                crate::app::ClickAction::SelectExt(i),
            );
        }
    }

    if lines.is_empty() {
        lines.push(Line::from(Span::styled(
            " No extensions installed",
            theme.faint_style,
        )));
    }

    let list = Paragraph::new(lines).style(theme.base);
    frame.render_widget(list, list_area);

    if let Some(ext) = app.project.extensions.get(app.ext_index) {
        draw_ext_detail(frame, app, detail_area, ext);
    }
}

fn draw_ext_detail(frame: &mut Frame, app: &App, area: Rect, ext: &ExtensionInfo) {
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
            Span::styled(format!(" {} ", ext.id), theme.accent_bold),
            Span::styled(format!("v{}", ext.version), theme.dim_style),
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
            if app.ext_tab == ExtTab::Extensions {
                lines.push(action_line("u", "update", theme));
            }
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

    if app.ext_tab == ExtTab::Presets && ext.status != InstallStatus::Available {
        lines.push(action_line("r", "resolve", theme));
    }

    let detail = Paragraph::new(lines)
        .wrap(Wrap { trim: false })
        .style(theme.base);
    frame.render_widget(detail, area);
}

fn draw_preset_list(frame: &mut Frame, app: &App, list_area: Rect, detail_area: Rect) {
    let theme = &app.theme;
    let mut lines: Vec<Line> = Vec::new();

    for (i, preset) in app.project.presets.iter().enumerate() {
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

        let priority = preset
            .priority
            .map(|p| format!("p{p} "))
            .unwrap_or_default();

        lines.push(Line::from(vec![
            sel_bar,
            status_dot(preset.status, theme),
            Span::styled(preset.id.clone(), row_style),
            Span::styled(format!(" {priority}"), theme.faint_style),
            Span::styled(format!("v{}", preset.version), theme.dim_style),
        ]));

        let row_y = list_area.y + i as u16;
        if row_y < list_area.y + list_area.height {
            app.register_click(
                Rect::new(list_area.x, row_y, list_area.width, 1),
                crate::app::ClickAction::SelectPreset(i),
            );
        }
    }

    if lines.is_empty() {
        lines.push(Line::from(Span::styled(
            " No presets installed",
            theme.faint_style,
        )));
    }

    let list = Paragraph::new(lines).style(theme.base);
    frame.render_widget(list, list_area);

    if let Some(preset) = app.project.presets.get(app.preset_index) {
        draw_preset_detail(frame, app, detail_area, preset);
    }
}

fn draw_preset_detail(frame: &mut Frame, _app: &App, area: Rect, preset: &PresetInfo) {
    let theme = &_app.theme;
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
            Span::styled(format!(" {} ", preset.id), theme.accent_bold),
            Span::styled(format!("v{}", preset.version), theme.dim_style),
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

fn action_line<'a>(key: &str, desc: &str, theme: &crate::theme::Theme) -> Line<'a> {
    Line::from(vec![
        Span::styled(format!("   [{key}] "), theme.accent_bold),
        Span::styled(desc.to_string(), theme.dim_style),
    ])
}
