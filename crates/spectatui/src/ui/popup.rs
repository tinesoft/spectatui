use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, BorderType, Borders, Clear, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState,
    Wrap,
};
use ratatui::Frame;

use crate::app::{App, PopupKind};

pub fn draw(frame: &mut Frame, app: &App) {
    let Some(popup) = &app.active_popup else {
        return;
    };

    dim_backdrop(frame);

    match popup {
        PopupKind::Integrations => {
            super::integrations::draw(frame, app);
        }
        PopupKind::Workflows => {
            super::workflows::draw(frame, app);
        }
        PopupKind::Catalogs => {
            super::catalogs::draw(frame, app);
        }
        PopupKind::Features => draw_features(frame, app),
        PopupKind::Help => {
            let area = fixed_centered_rect(70, 26, frame.area());
            draw_help(frame, app, area);
        }
        PopupKind::QuitConfirm => {
            let area = fixed_centered_rect(44, 9, frame.area());
            draw_quit_confirm(frame, app, area);
        }
        PopupKind::Extensions => super::extensions::draw(frame, app),
        PopupKind::Presets => super::presets::draw(frame, app),
        PopupKind::CommandPalette => {}
        PopupKind::CliConfirm => {
            let area = fixed_centered_rect(72, 13, frame.area());
            draw_cli_confirm(frame, app, area);
        }
        PopupKind::CliOutput => {
            let area = fixed_centered_rect(84, 22, frame.area());
            draw_cli_output(frame, app, area);
        }
    }
}

fn dim_backdrop(frame: &mut Frame) {
    let area = frame.area();
    let buf = frame.buffer_mut();
    for y in area.y..area.y + area.height {
        for x in area.x..area.x + area.width {
            let cell = &mut buf[(x, y)];
            if let ratatui::style::Color::Rgb(r, g, b) = cell.fg {
                let dim = |v: u8| -> u8 { ((v as u16 * 55 + 16 * 45) / 100) as u8 };
                cell.set_fg(ratatui::style::Color::Rgb(dim(r), dim(g), dim(b)));
            }
        }
    }
}

fn fixed_centered_rect(w: u16, h: u16, r: Rect) -> Rect {
    let x = r.x + (r.width.saturating_sub(w)) / 2;
    let y = r.y + (r.height.saturating_sub(h)) / 2;
    Rect::new(x, y, w.min(r.width), h.min(r.height))
}

fn draw_features(frame: &mut Frame, app: &App) {
    let theme = &app.theme;

    let full = frame.area();
    let w = ((108.min(full.width.saturating_sub(6))) as f32 * 2.0 / 3.0).round() as u16;
    let h = 30u16.min(full.height.saturating_sub(6)).max(8);
    let area = fixed_centered_rect(w, h, full);
    frame.render_widget(Clear, area);

    let title = Line::from(vec![
        Span::styled("─┤ ", theme.border_focused),
        Span::styled("Features", theme.title_focused),
        Span::styled(" ├", theme.border_focused),
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme.border_focused)
        .title(title)
        .padding(super::PANEL_PADDING);

    let mut lines = Vec::new();
    if app.project.features.is_empty() {
        lines.push(Line::from(Span::styled(
            " No features found",
            theme.faint_style,
        )));
    } else {
        let inner_width = area.width.saturating_sub(4) as usize;

        // Each feature occupies 2 rows. The body spans from area.y+2 (after the
        // top border and the blank line) down to the row above the footer, so
        // only `per_page` features fit. Scroll the window to keep the selected
        // feature visible instead of drawing it off-screen.
        let body_rows = (area.height as usize).saturating_sub(4);
        let per_page = (body_rows / 2).max(1);
        let offset = super::scroll_offset(app.feature_index, per_page);

        for (i, feat) in app
            .project
            .features
            .iter()
            .enumerate()
            .skip(offset)
            .take(per_page)
        {
            let row = i - offset;
            let selected = i == app.feature_index;
            let badge_style = theme.stage_badge(feat.stage.label(), app.theme_mode);
            let running = app.running_features.contains(&feat.id);

            let sel_bar = if selected {
                Span::styled(" ▌", theme.accent_style)
            } else {
                Span::raw("  ")
            };

            let (run_dot, run_label, run_style) = if running {
                ("●", "running", theme.good_style)
            } else {
                ("○", "idle", theme.faint_style)
            };

            let id_style = if selected {
                Style::default().fg(theme.sel_fg).bg(theme.sel)
            } else {
                Style::default().fg(theme.fg)
            };

            let left = format!("{run_dot} {} {}", feat.stage.label(), feat.id);
            let pad =
                inner_width.saturating_sub(2 + left.chars().count() + 3 + run_label.len() + 1);

            lines.push(Line::from(vec![
                sel_bar,
                Span::styled(format!("{run_dot} "), run_style),
                Span::styled(format!(" {} ", feat.stage.label()), badge_style),
                Span::styled(format!(" {}", feat.id), id_style),
                Span::styled(" ".repeat(pad), theme.base),
                Span::styled(run_label, run_style),
            ]));
            lines.push(Line::from(Span::styled(
                format!("      {}", stage_note(feat.stage)),
                theme.dim_style,
            )));

            let row_y = area.y + 2 + (row as u16) * 2;
            app.register_click(
                Rect::new(area.x + 1, row_y, area.width.saturating_sub(2), 2),
                crate::app::ClickAction::JumpToFeature(i),
            );
        }
    }

    let content = Paragraph::new(lines).block(block).style(theme.base);
    frame.render_widget(content, area);

    if area.height > 2 {
        let footer_y = area.y + area.height - 2;
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                "[enter] jump to feature   esc close",
                theme.faint_style,
            ))),
            Rect::new(area.x + 3, footer_y, area.width.saturating_sub(6), 1),
        );
    }
}

fn stage_note(stage: spectatui_core::speckit::WorkflowStage) -> &'static str {
    use spectatui_core::speckit::WorkflowStage;
    match stage {
        WorkflowStage::NotStarted => "constitution pending",
        WorkflowStage::Specified => "spec written, needs clarification",
        WorkflowStage::Clarified => "clarified, ready to plan",
        WorkflowStage::Planned => "plan ready, generate tasks",
        WorkflowStage::TasksGenerated => "tasks generated, run analysis",
        WorkflowStage::Analyzed => "analyzed, ready to implement",
        WorkflowStage::Implementing => "implementation in progress",
        WorkflowStage::Implemented => "implemented",
        WorkflowStage::Unknown => "unrecognized artifact format",
    }
}

fn draw_help(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;

    // Paint a solid background so the dashboard beneath doesn't bleed through.
    frame.render_widget(Clear, area);

    let title = Line::from(vec![
        Span::styled("─┤ ", theme.border_focused),
        Span::styled("Keybindings", theme.title_focused),
        Span::styled(" ├", theme.border_focused),
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme.border_focused)
        .title(title)
        .padding(super::PANEL_PADDING);

    let groups: [(&str, &[(&str, &str)]); 3] = [
        (
            "Navigation",
            &[
                ("tab / shift-tab", "cycle pane focus"),
                ("↑↓ / j k", "move selection / scroll"),
                ("enter", "open / activate"),
                ("esc", "back / close popup"),
                ("q", "quit"),
            ],
        ),
        (
            "Screens",
            &[
                (":", "command palette"),
                ("e", "extensions & presets"),
                ("i / w / p", "status-bar popups"),
                ("1 · 2 · 3", "dashboard layout preset"),
            ],
        ),
        (
            "Appearance",
            &[
                ("t", "cycle theme (dark/light)"),
                ("T", "cycle accent palette"),
            ],
        ),
    ];

    let group_header = theme.accent_bold;
    let mut lines: Vec<Line> = Vec::new();
    for (gi, (name, keys)) in groups.iter().enumerate() {
        if gi > 0 {
            lines.push(Line::default());
        }
        lines.push(Line::from(vec![
            Span::styled("  ▍ ", theme.accent_style),
            Span::styled((*name).to_string(), group_header),
        ]));
        for (key, desc) in keys.iter() {
            lines.push(Line::from(vec![
                Span::styled(format!("  {key:>16}  "), theme.accent_bold),
                Span::styled((*desc).to_string(), theme.dim_style),
            ]));
        }
    }

    let content = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false })
        .style(theme.base);
    frame.render_widget(content, area);
}

fn draw_quit_confirm(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;

    frame.render_widget(Clear, area);

    let title = Line::from(vec![
        Span::styled("─┤ ", theme.border_focused),
        Span::styled("Quit Spectatui?", theme.title_focused),
        Span::styled(" ├", theme.border_focused),
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme.border_focused)
        .title(title)
        .padding(super::PANEL_PADDING);

    let lines = vec![
        Line::from(Span::styled(
            "  Sessions keep running in tmux.",
            Style::default().fg(theme.fg),
        )),
        Line::default(),
        Line::from(vec![
            Span::styled("  [q]", theme.accent_bold),
            Span::styled(" quit  ", theme.dim_style),
            Span::styled("[esc]", theme.accent_bold),
            Span::styled(" stay", theme.dim_style),
        ]),
    ];

    let content = Paragraph::new(lines).block(block).style(theme.base);
    frame.render_widget(content, area);
}

fn draw_cli_confirm(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    frame.render_widget(Clear, area);

    let title = Line::from(vec![
        Span::styled("─┤ ", theme.border_focused),
        Span::styled("Confirm action", theme.title_focused),
        Span::styled(" ├", theme.border_focused),
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme.border_focused)
        .title(title)
        .padding(super::PANEL_PADDING);

    let cmd_text = app
        .pending_action
        .as_ref()
        .map(|a| a.to_command_line())
        .unwrap_or_default();

    let force_line = if app.force_flag {
        Span::styled("  --force ON · prompts skipped", theme.warn_style)
    } else {
        Span::styled("  --force off · CLI prompts preserved", theme.dim_style)
    };

    // Command line highlighted on a panel_alt bar, padded to box width.
    let inner_w = area.width.saturating_sub(4) as usize;
    let cmd_bar = format!("$ {cmd_text}");
    let cmd_pad = inner_w.saturating_sub(cmd_bar.len() + 2);
    let panel_alt_bg = Style::default().fg(theme.fg).bg(theme.panel_alt);

    let lines = vec![
        Line::from(Span::styled(
            "  Spectatui will run this command:",
            theme.dim_style,
        )),
        Line::default(),
        Line::from(vec![
            Span::styled("  $ ", Style::default().fg(theme.good).bg(theme.panel_alt)),
            Span::styled(
                cmd_text,
                panel_alt_bg.add_modifier(ratatui::style::Modifier::BOLD),
            ),
            Span::styled(" ".repeat(cmd_pad), panel_alt_bg),
        ]),
        Line::default(),
        Line::from(Span::styled(
            "  The CLI runs its own confirmation & config backup.",
            theme.dim_style,
        )),
        Line::from(force_line),
        Line::default(),
        Line::from(vec![
            Span::styled("  [enter]", theme.accent_bold),
            Span::styled(" run  ", theme.dim_style),
            Span::styled("[f]", theme.accent_bold),
            Span::styled(" toggle --force  ", theme.dim_style),
            Span::styled("[esc]", theme.accent_bold),
            Span::styled(" cancel", theme.dim_style),
        ]),
    ];

    let content = Paragraph::new(lines).block(block).style(theme.base);
    frame.render_widget(content, area);
}

fn draw_cli_output(frame: &mut Frame, app: &App, area: Rect) {
    use spectatui_core::speckit::cli::JobStatus;
    let theme = &app.theme;
    frame.render_widget(Clear, area);

    let title = Line::from(vec![
        Span::styled("─┤ ", theme.border_focused),
        Span::styled("CLI Output · specify", theme.title_focused),
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

    let job = app.cli_job.as_ref();
    let cmd_line = job.map(|j| j.command_line.as_str()).unwrap_or_default();
    let output = job.map(|j| j.output.as_str()).unwrap_or("");

    // Top: command echo. Then output lines, leaving the last row for status.
    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(vec![
        Span::styled("$ ", theme.good_style),
        Span::styled(
            cmd_line,
            Style::default()
                .fg(theme.fg)
                .add_modifier(ratatui::style::Modifier::BOLD),
        ),
    ]));
    lines.push(Line::default());
    for l in output.lines() {
        lines.push(Line::from(Span::styled(l.to_string(), theme.dim_style)));
    }

    // Scrollable content sits above the pinned status row.
    let view_h = inner.height.saturating_sub(1);
    let content_area = Rect::new(inner.x, inner.y, inner.width, view_h);
    let wrap_w = content_area.width.max(1) as usize;
    let total_rows: usize = lines
        .iter()
        .map(|l| l.width().max(1).div_ceil(wrap_w))
        .sum();
    let max_scroll = (total_rows as u16).saturating_sub(view_h);
    app.cli_scroll_max.set(max_scroll);
    let scroll = app.cli_scroll.min(max_scroll);

    let content = Paragraph::new(lines)
        .wrap(Wrap { trim: false })
        .scroll((scroll, 0))
        .style(theme.base);
    frame.render_widget(content, content_area);

    if max_scroll > 0 {
        let mut scrollbar_state = ScrollbarState::new(total_rows).position(scroll as usize);
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .track_symbol(Some("│"))
            .thumb_symbol("┃");
        frame.render_stateful_widget(
            scrollbar
                .style(theme.border_unfocused)
                .thumb_style(theme.accent_style),
            content_area,
            &mut scrollbar_state,
        );
    }

    // Status row pinned to the bottom of the inner area.
    let (status_text, status_style) = match job.map(|j| j.status) {
        Some(JobStatus::Running) => {
            let spinner = "⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏";
            let idx = (app.indexing_tick as usize) % spinner.chars().count();
            let ch = spinner.chars().nth(idx).unwrap_or('⠋');
            (format!("{ch} running…"), theme.warn_style)
        }
        Some(JobStatus::Succeeded) => (
            "✓ succeeded · list refreshed   [esc] close".to_string(),
            theme.good_style,
        ),
        Some(JobStatus::Failed) => ("✗ failed   [esc] close".to_string(), theme.bad_style),
        _ => ("pending".to_string(), theme.faint_style),
    };
    if inner.height >= 1 {
        let status_y = inner.y + inner.height - 1;
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(status_text, status_style))).style(theme.base),
            Rect::new(inner.x, status_y, inner.width, 1),
        );
    }
}
