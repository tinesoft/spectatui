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

    // The full manager keeps a single "Extensions & Presets" box title and draws
    // the tabs as a highlighted row inside; the single-category popup just names
    // the category.
    let title_line = if let Some(tab) = single {
        let label = match tab {
            ExtTab::Extensions => "Extensions",
            ExtTab::Presets => "Presets",
        };
        Line::from(vec![
            Span::styled("─┤ ", theme.border_focused),
            Span::styled(label, theme.accent_bold),
            Span::styled(" ├", theme.border_focused),
        ])
    } else {
        Line::from(vec![
            Span::styled("─┤ ", theme.border_focused),
            Span::styled("Extensions & Presets", theme.title_focused),
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

    if inner.width < 10 || inner.height < 5 {
        return;
    }

    // Tab row (tabbed mode only) on the first inner row, active tab highlighted.
    if single.is_none() {
        let mut spans: Vec<Span> = vec![Span::raw(" ")];
        let tabs = [
            (ExtTab::Extensions, format!("Extensions {ext_count}")),
            (ExtTab::Presets, format!("Presets {preset_count}")),
        ];
        let mut tx = inner.x + 1;
        for (i, (tab, label)) in tabs.iter().enumerate() {
            if i > 0 {
                spans.push(Span::raw(" "));
                tx += 1;
            }
            let chip = format!(" {label} ");
            let style = if *tab == active_tab {
                Style::default()
                    .fg(theme.accent)
                    .bg(theme.panel_alt)
                    .add_modifier(ratatui::style::Modifier::BOLD)
            } else {
                theme.dim_style
            };
            app.register_click(
                Rect::new(tx, inner.y, chip.len() as u16, 1),
                crate::app::ClickAction::SetExtTab(*tab),
            );
            tx += chip.len() as u16;
            spans.push(Span::styled(chip, style));
        }
        frame.render_widget(
            Paragraph::new(Line::from(spans)).style(theme.base),
            Rect::new(inner.x, inner.y, inner.width, 1),
        );
    }

    // Single-category popup has no tab row, so its content sits one row higher.
    let (top_off, shrink) = if single.is_some() { (1, 2) } else { (2, 3) };
    let body = Rect::new(
        inner.x,
        inner.y + top_off,
        inner.width,
        inner.height.saturating_sub(shrink),
    );

    let cols =
        Layout::horizontal([Constraint::Percentage(45), Constraint::Percentage(55)]).split(body);

    match active_tab {
        ExtTab::Extensions => draw_ext_list(frame, app, cols[0], cols[1]),
        ExtTab::Presets => draw_preset_list(frame, app, cols[0], cols[1]),
    }

    // Footer one row inside the bottom border.
    if area.height > 2 {
        let footer_y = area.y + area.height - 2;
        let footer_area = Rect::new(area.x + 2, footer_y, area.width.saturating_sub(4), 1);
        let is_compact = inner.width < 60;
        let footer_text = if is_compact {
            "[enter] manage"
        } else if single.is_some() {
            "[/] search catalog   [c] catalog list   esc close"
        } else {
            "[/] search catalog   [c] catalog list"
        };
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(footer_text, theme.faint_style))),
            footer_area,
        );
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

        // Priority and version are right-aligned columns; "—" marks no priority.
        let pri = ext
            .priority
            .map(|p| format!("p{p}"))
            .unwrap_or_else(|| "—".to_string());
        let ver = format!("v{}", ext.version);
        let list_w = list_area.width as usize;
        let pad = list_w.saturating_sub(1 + 2 + ext.id.chars().count() + 5 + ver.chars().count() + 1);

        lines.push(Line::from(vec![
            sel_bar,
            status_dot(ext.status, theme),
            Span::styled(ext.id.clone(), row_style),
            Span::styled(" ".repeat(pad), theme.base),
            Span::styled(format!("{pri:<5}"), theme.dim_style),
            Span::styled(ver, theme.faint_style),
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

        let pri = preset
            .priority
            .map(|p| format!("p{p}"))
            .unwrap_or_else(|| "—".to_string());
        let ver = format!("v{}", preset.version);
        let list_w = list_area.width as usize;
        let pad = list_w.saturating_sub(1 + 2 + preset.id.chars().count() + 5 + ver.chars().count() + 1);

        lines.push(Line::from(vec![
            sel_bar,
            status_dot(preset.status, theme),
            Span::styled(preset.id.clone(), row_style),
            Span::styled(" ".repeat(pad), theme.base),
            Span::styled(format!("{pri:<5}"), theme.dim_style),
            Span::styled(ver, theme.faint_style),
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
