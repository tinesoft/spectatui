use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph};
use ratatui::Frame;

use crate::app::{App, Pane};

pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let focused = app.focused_pane == Pane::FeatureList;
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

    let title = Line::from(vec![
        Span::styled("─┤ ", border_style),
        Span::styled("Features", title_style),
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

    if app.project.features.is_empty() {
        let message = if app.project.has_speckit_structure() {
            "No features found in specs/"
        } else {
            "Not a recognized Spec-Kit project (no .specify/ found)"
        };
        let empty = Paragraph::new(Line::from(vec![Span::styled(message, theme.faint_style)]))
            .style(theme.base);
        frame.render_widget(empty, inner);
        return;
    }

    let max_visible = ((inner.height as usize) / 2).max(1);
    let offset = super::scroll_offset(app.feature_index, max_visible);
    let mut lines: Vec<Line> = Vec::new();

    for (i, feature) in app
        .project
        .features
        .iter()
        .enumerate()
        .skip(offset)
        .take(max_visible)
    {
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

        let is_running = app.running_features.contains(&feature.id);
        let status_dot = if is_running {
            Span::styled(" ●", theme.good_style)
        } else {
            Span::styled(" ○", theme.faint_style)
        };

        let badge_text = format!(" {stage_label} ");
        let id_text = format!(" {}", feature.id);

        let remaining = inner
            .width
            .saturating_sub(1 + badge_text.len() as u16 + id_text.len() as u16 + 3);
        let pad = " ".repeat(remaining as usize);

        let row1 = Line::from(vec![
            sel_bar,
            Span::styled(badge_text, badge_style),
            Span::styled(id_text, id_style),
            Span::styled(pad, row_bg),
            status_dot,
            Span::styled(" ", row_bg),
        ]);

        let note = compute_feature_note(feature);
        let note_pad_len = inner.width.saturating_sub(note.len() as u16 + 9) as usize;
        let row2 = Line::from(vec![
            Span::styled(
                if selected { "▌" } else { " " },
                if selected {
                    theme.accent_style
                } else {
                    Style::default()
                },
            ),
            Span::styled("       ", row_bg),
            Span::styled(
                note.to_string(),
                Style::default()
                    .fg(theme.dim)
                    .bg(if selected { theme.sel } else { theme.bg }),
            ),
            Span::styled(" ".repeat(note_pad_len), row_bg),
        ]);

        let row_y = inner.y + (lines.len() as u16);
        app.register_click(
            Rect::new(inner.x, row_y, inner.width, 2),
            crate::app::ClickAction::SelectFeature(i),
        );

        lines.push(row1);
        lines.push(row2);
    }

    // Fill remaining space, then footer at the bottom of the inner area.
    let footer_row = (inner.height as usize).saturating_sub(1);
    while lines.len() < footer_row {
        lines.push(Line::default());
    }
    lines.push(Line::from(vec![
        Span::styled("[enter]", theme.accent_bold),
        Span::styled(" open spec", theme.faint_style),
    ]));

    let list = Paragraph::new(lines).style(theme.base);
    frame.render_widget(list, inner);
}

fn compute_feature_note(feature: &spectatui_core::speckit::Feature) -> String {
    use spectatui_core::speckit::{TasksProgress, WorkflowStage};
    if let Some(tasks_path) = &feature.artifacts.tasks {
        if let Some(progress) = TasksProgress::from_file(tasks_path) {
            return format!("tasks {}/{} complete", progress.done, progress.total);
        }
    }
    match feature.stage {
        WorkflowStage::NotStarted => "not started".to_string(),
        WorkflowStage::Specified => "spec.md drafted · needs /clarify".to_string(),
        WorkflowStage::Clarified => "clarified".to_string(),
        WorkflowStage::Planned => "plan.md ready · awaiting /tasks".to_string(),
        WorkflowStage::TasksGenerated => "tasks generated".to_string(),
        WorkflowStage::Analyzed => "analysis report clean".to_string(),
        WorkflowStage::Implementing => "in progress".to_string(),
        WorkflowStage::Implemented => "complete".to_string(),
        WorkflowStage::Unknown => "unrecognized artifact format".to_string(),
    }
}
