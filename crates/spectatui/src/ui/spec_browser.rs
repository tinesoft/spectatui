use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap};
use ratatui::Frame;

use crate::app::{App, SpecTab};
use crate::theme::Theme;

pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let cols = Layout::horizontal([Constraint::Length(30), Constraint::Min(0)]).split(area);

    draw_feature_sidebar(frame, app, cols[0]);
    draw_doc_pane(frame, app, cols[1], true);
}

pub fn draw_inline(frame: &mut Frame, app: &App, area: Rect) {
    draw_doc_pane(frame, app, area, false);
}

pub fn draw_constitution(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;

    let title = Line::from(vec![
        Span::styled("─┤ ", theme.border_focused),
        Span::styled("constitution.md", theme.title_focused),
        Span::styled(" ├", theme.border_focused),
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme.border_focused)
        .title(title);

    let content_text = app
        .constitution_content()
        .unwrap_or_else(|| "No constitution found.".to_string());

    let lines: Vec<Line> = content_text
        .lines()
        .map(|l| render_md_line(l, theme))
        .collect();

    let total_lines = lines.len() as u16;
    let inner_height = block.inner(area).height;

    let content = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false })
        .scroll((app.spec_scroll, 0))
        .style(theme.base);

    frame.render_widget(content, area);

    if total_lines > inner_height {
        let mut scrollbar_state = ScrollbarState::new(total_lines as usize)
            .position(app.spec_scroll as usize);
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .track_symbol(Some("│"))
            .thumb_symbol("┃");
        frame.render_stateful_widget(
            scrollbar.style(theme.border_unfocused).thumb_style(theme.accent_style),
            area,
            &mut scrollbar_state,
        );
    }
}

fn draw_feature_sidebar(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;

    let title = Line::from(vec![
        Span::styled("─┤ ", theme.border_unfocused),
        Span::styled("Features", theme.title_unfocused),
        Span::styled(" ├", theme.border_unfocused),
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme.border_unfocused)
        .title(title);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut lines: Vec<Line> = Vec::new();

    for (i, feature) in app.project.features.iter().enumerate() {
        let selected = i == app.feature_index;
        let stage_label = feature.stage.label();
        let badge_style = theme.stage_badge(stage_label, app.theme_mode);

        let sel_bar = if selected {
            Span::styled("▌", theme.accent_style)
        } else {
            Span::raw(" ")
        };

        let id_style = if selected {
            Style::default().fg(theme.sel_fg).bg(theme.sel)
        } else {
            Style::default().fg(theme.fg).bg(theme.bg)
        };

        let row_bg = if selected {
            Style::default().bg(theme.sel)
        } else {
            Style::default().bg(theme.bg)
        };

        let id_text = format!(" {}", feature.id);
        let remaining = inner.width.saturating_sub(1 + stage_label.len() as u16 + 2 + id_text.len() as u16);
        let pad = " ".repeat(remaining as usize);

        lines.push(Line::from(vec![
            sel_bar,
            Span::styled(format!(" {stage_label} "), badge_style),
            Span::styled(id_text, id_style),
            Span::styled(pad, row_bg),
        ]));

        let row_y = inner.y + i as u16;
        if row_y < inner.y + inner.height {
            app.register_click(
                Rect::new(inner.x, row_y, inner.width, 1),
                crate::app::ClickAction::SelectFeature(i),
            );
        }
    }

    let list = Paragraph::new(lines).style(theme.base);
    frame.render_widget(list, inner);
}

fn draw_doc_pane(frame: &mut Frame, app: &App, area: Rect, focused: bool) {
    let theme = &app.theme;
    let border_style = if focused {
        theme.border_focused
    } else {
        theme.border_unfocused
    };

    let tabs = build_tabs(app, border_style);
    let title = Line::from(tabs);

    // Register clickable tab regions (title starts with "─┤ " = 3 cells, tabs
    // separated by " │ " = 3 cells).
    {
        let tab_kinds = [SpecTab::Spec, SpecTab::Plan, SpecTab::Tasks, SpecTab::Research];
        let mut tx = area.x + 3;
        for tab in tab_kinds {
            let len = tab.label().len() as u16;
            app.register_click(
                Rect::new(tx, area.y, len, 1),
                crate::app::ClickAction::SetSpecTab(tab),
            );
            tx += len + 3;
        }
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(border_style)
        .title(title);

    let inner_area = block.inner(area);

    let content_text = app
        .selected_doc_content()
        .unwrap_or_else(|| "No content available for this tab.".to_string());

    let is_tasks = app.spec_tab == SpecTab::Tasks;

    let lines: Vec<Line> = if is_tasks {
        content_text
            .lines()
            .map(|l| render_tasks_line(l, theme))
            .collect()
    } else {
        content_text
            .lines()
            .map(|l| render_md_line(l, theme))
            .collect()
    };

    let total_lines = lines.len() as u16;

    let content = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false })
        .scroll((app.spec_scroll, 0))
        .style(theme.base);

    frame.render_widget(content, area);

    if total_lines > inner_area.height {
        let mut scrollbar_state = ScrollbarState::new(total_lines as usize)
            .position(app.spec_scroll as usize);
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .track_symbol(Some("│"))
            .thumb_symbol("┃");
        frame.render_stateful_widget(
            scrollbar.style(theme.border_unfocused).thumb_style(theme.accent_style),
            area,
            &mut scrollbar_state,
        );
    }
}

fn build_tabs<'a>(app: &App, border_style: Style) -> Vec<Span<'a>> {
    let theme = &app.theme;
    let tabs = [SpecTab::Spec, SpecTab::Plan, SpecTab::Tasks, SpecTab::Research];

    let mut spans = vec![Span::styled("─┤ ", border_style)];

    for (i, tab) in tabs.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled(" │ ", theme.faint_style));
        }

        let label = tab.label();
        let available = match tab {
            SpecTab::Spec => app.selected_feature().and_then(|f| f.artifacts.spec.as_ref()).is_some(),
            SpecTab::Plan => app.selected_feature().and_then(|f| f.artifacts.plan.as_ref()).is_some(),
            SpecTab::Tasks => app.selected_feature().and_then(|f| f.artifacts.tasks.as_ref()).is_some(),
            SpecTab::Research => app.selected_feature().and_then(|f| f.artifacts.research.as_ref()).is_some(),
        };

        let style = if *tab == app.spec_tab {
            theme.accent_bold
        } else if available {
            theme.dim_style
        } else {
            theme.faint_style
        };

        spans.push(Span::styled(label.to_string(), style));
    }

    spans.push(Span::styled(" ├", border_style));
    spans
}

fn render_md_line<'a>(line: &str, theme: &Theme) -> Line<'a> {
    if line.starts_with("# ") {
        Line::from(Span::styled(
            line[2..].to_string(),
            Style::default().fg(theme.fg).add_modifier(Modifier::BOLD),
        ))
    } else if line.starts_with("## ") {
        Line::from(vec![
            Span::styled("▍ ", theme.accent_style),
            Span::styled(
                line[3..].to_string(),
                Style::default().fg(theme.fg).add_modifier(Modifier::BOLD),
            ),
        ])
    } else if line.starts_with("### ") {
        Line::from(Span::styled(
            line[4..].to_string(),
            Style::default().fg(theme.fg).add_modifier(Modifier::BOLD),
        ))
    } else if line.starts_with("- ") || line.starts_with("* ") {
        Line::from(vec![
            Span::styled("  • ", theme.accent_style),
            Span::styled(line[2..].to_string(), theme.dim_style),
        ])
    } else if line.starts_with("  - ") || line.starts_with("  * ") {
        Line::from(vec![
            Span::styled("    • ", theme.accent_style),
            Span::styled(line[4..].to_string(), theme.dim_style),
        ])
    } else if line.starts_with("```") {
        Line::from(Span::styled(
            line.to_string(),
            Style::default().fg(theme.info),
        ))
    } else if line.starts_with("**") {
        Line::from(Span::styled(
            line.to_string(),
            Style::default().fg(theme.fg).add_modifier(Modifier::BOLD),
        ))
    } else if line.starts_with("---") {
        Line::from(Span::styled(
            "─".repeat(40),
            theme.faint_style,
        ))
    } else if line.trim().is_empty() {
        Line::default()
    } else {
        Line::from(Span::styled(line.to_string(), theme.dim_style))
    }
}

fn render_tasks_line<'a>(line: &str, theme: &Theme) -> Line<'a> {
    let trimmed = line.trim();

    if trimmed.starts_with("# ") {
        Line::from(Span::styled(
            trimmed[2..].to_string(),
            Style::default().fg(theme.fg).add_modifier(Modifier::BOLD),
        ))
    } else if trimmed.starts_with("## ") {
        Line::from(vec![
            Span::styled("▍ ", theme.accent_style),
            Span::styled(
                trimmed[3..].to_string(),
                Style::default().fg(theme.fg).add_modifier(Modifier::BOLD),
            ),
        ])
    } else if trimmed.starts_with("- [x]") || trimmed.starts_with("- [X]") {
        let rest = &trimmed[5..];
        let (task_id, desc) = split_task_id(rest);
        Line::from(vec![
            Span::styled("  [✓] ", theme.good_style),
            Span::styled(task_id.to_string(), theme.info_style),
            Span::styled(desc.to_string(), theme.dim_style),
        ])
    } else if trimmed.starts_with("- [ ]") {
        let rest = &trimmed[5..];
        let (task_id, desc) = split_task_id(rest);

        if desc.contains("[P]") {
            let desc_clean = desc.replace("[P]", "");
            Line::from(vec![
                Span::styled("  [ ] ", theme.faint_style),
                Span::styled(task_id.to_string(), theme.info_style),
                Span::styled("[P] ", theme.warn_style),
                Span::styled(desc_clean.trim().to_string(), theme.dim_style),
            ])
        } else {
            Line::from(vec![
                Span::styled("  [ ] ", theme.faint_style),
                Span::styled(task_id.to_string(), theme.info_style),
                Span::styled(desc.to_string(), theme.dim_style),
            ])
        }
    } else if trimmed.starts_with("- ") {
        Line::from(vec![
            Span::styled("  • ", theme.accent_style),
            Span::styled(trimmed[2..].to_string(), theme.dim_style),
        ])
    } else if trimmed.starts_with("**") {
        Line::from(Span::styled(
            trimmed.to_string(),
            Style::default().fg(theme.fg).add_modifier(Modifier::BOLD),
        ))
    } else if trimmed.starts_with("---") {
        Line::from(Span::styled("─".repeat(40), theme.faint_style))
    } else if trimmed.is_empty() {
        Line::default()
    } else {
        Line::from(Span::styled(line.to_string(), theme.dim_style))
    }
}

fn split_task_id(s: &str) -> (&str, &str) {
    let s = s.trim_start();
    if let Some(idx) = s.find(|c: char| c == ' ' || c == '\t') {
        let potential_id = &s[..idx];
        if potential_id.starts_with('T') && potential_id.len() <= 5 {
            return (potential_id, &s[idx..]);
        }
    }
    ("", s)
}
