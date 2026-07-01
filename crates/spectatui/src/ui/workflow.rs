use ratatui::layout::Rect;
use ratatui::style::Modifier;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph};
use ratatui::Frame;

use spectatui_core::speckit::WorkflowStage;

use crate::app::{App, Pane};

const STAGES: &[(&str, WorkflowStage)] = &[
    ("cons", WorkflowStage::NotStarted),
    ("spec", WorkflowStage::Specified),
    ("clar", WorkflowStage::Clarified),
    ("plan", WorkflowStage::Planned),
    ("task", WorkflowStage::TasksGenerated),
    ("anly", WorkflowStage::Analyzed),
    ("impl", WorkflowStage::Implementing),
];

pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let focused = app.focused_pane == Pane::Workflow;
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

    let feature_id = app
        .selected_feature()
        .map(|f| f.id.as_str())
        .unwrap_or("none");

    let title = Line::from(vec![
        Span::styled("─┤ ", border_style),
        Span::styled(format!("Workflow · {feature_id}"), title_style),
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

    let Some(feature) = app.selected_feature() else {
        let empty = Paragraph::new(Line::from(Span::styled(
            "No feature selected",
            theme.faint_style,
        )))
        .style(theme.base);
        frame.render_widget(empty, inner);
        return;
    };

    let current_stage = feature.stage;
    let done_style = theme.stepper_done_style(app.theme_mode);

    let mut stepper_spans: Vec<Span> = vec![Span::raw(" ")];

    for (i, (label, min_stage)) in STAGES.iter().enumerate() {
        if i > 0 {
            stepper_spans.push(Span::styled("─►", theme.faint_style));
        }

        let badge_text = format!(" {label} ");
        let style = if current_stage > *min_stage
            || (matches!(current_stage, WorkflowStage::Implemented)
                && *label == "impl")
        {
            done_style
        } else if current_stage == *min_stage
            || (current_stage == WorkflowStage::Implementing && *label == "impl")
        {
            theme
                .stage_badge(label, app.theme_mode)
                .add_modifier(Modifier::BOLD)
        } else {
            ratatui::style::Style::default()
                .fg(theme.faint)
                .bg(theme.bg)
        };

        stepper_spans.push(Span::styled(badge_text, style));
    }

    let stepper_line = Line::from(stepper_spans);

    let mut lines = vec![stepper_line, Line::default()];

    let available_height = inner.height as usize;

    let current_label = current_stage.label();
    let current_badge_style = theme.stage_badge(current_label, app.theme_mode);
    let stage_verb = stage_verb(current_stage);

    if available_height > 4 {
        lines.push(Line::from(vec![
            Span::styled("  Current stage: ", theme.dim_style),
            Span::styled(format!(" {current_label} "), current_badge_style),
            Span::styled(format!(" {stage_verb}"), theme.dim_style),
        ]));
    }

    if available_height > 5 {
        if let Some(progress) = app.selected_tasks_progress() {
            let bar_w = (inner.width as usize).saturating_sub(26).clamp(7, 40);
            let filled = if progress.total > 0 {
                (progress.done * bar_w) / progress.total
            } else {
                0
            };
            let empty = bar_w - filled;
            let bar = format!(
                "{}{}",
                "█".repeat(filled),
                "░".repeat(empty)
            );
            lines.push(Line::from(vec![
                Span::styled("  Tasks [", theme.dim_style),
                Span::styled(bar, theme.accent_style),
                Span::styled(
                    format!("] {}/{} {}%", progress.done, progress.total, progress.percent()),
                    theme.dim_style,
                ),
            ]));
        }
    }

    if available_height > 7 {
        if let Some(branch) = feature.branch.as_deref() {
            lines.push(Line::from(vec![
                Span::styled("  branch  ", theme.dim_style),
                Span::styled(branch.to_string(), theme.info_style),
            ]));
        }
    }

    // Fill remaining space before footer
    while lines.len() < (inner.height as usize).saturating_sub(1) {
        lines.push(Line::default());
    }
    lines.push(Line::from(Span::styled(
        "[enter] open spec   [a] sessions",
        theme.faint_style,
    )));

    let content = Paragraph::new(lines).style(theme.base);
    frame.render_widget(content, inner);
}

fn stage_verb(stage: WorkflowStage) -> &'static str {
    match stage {
        WorkflowStage::NotStarted => "constitution",
        WorkflowStage::Specified => "specify",
        WorkflowStage::Clarified => "clarify",
        WorkflowStage::Planned => "plan",
        WorkflowStage::TasksGenerated => "tasks",
        WorkflowStage::Analyzed => "analyze",
        WorkflowStage::Implementing => "implement",
        WorkflowStage::Implemented => "implement",
    }
}
