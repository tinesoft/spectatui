mod agent_output;
mod extensions;
mod extensions_presets;
mod feature_list;
mod presets;
mod header;
mod integrations;
mod layout_editor;
mod palette;
mod popup;
mod session_attach;
mod settings;
mod spec_browser;
mod statusbar;
mod workflow;
mod workflows;

use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::app::{App, DashboardLayout, Screen};
use crate::theme::Theme;

/// Consistent inner padding for every bordered panel/popup so content does not
/// hug the frame borders.
pub(super) const PANEL_PADDING: ratatui::widgets::Padding =
    ratatui::widgets::Padding { left: 1, right: 1, top: 1, bottom: 1 };

/// First visible row index for a 1-row-per-item list so `selected` stays on screen.
pub(super) fn scroll_offset(selected: usize, visible_rows: usize) -> usize {
    if visible_rows == 0 || selected < visible_rows {
        0
    } else {
        selected - visible_rows + 1
    }
}

/// Footer for filterable popups: an active filter bar (`/ query ▌ … enter keep ·
/// esc clear`) or the idle `[/] filter <noun>   esc close` prompt.
pub(super) fn draw_search_footer(
    frame: &mut Frame,
    theme: &Theme,
    area: Rect,
    noun: &str,
    active: bool,
    query: &str,
) {
    if active {
        let bar = Style::default().bg(theme.panel_alt);
        let (text, text_style) = if query.is_empty() {
            (format!("filter {noun}…"), Style::default().fg(theme.faint).bg(theme.panel_alt))
        } else {
            (query.to_string(), Style::default().fg(theme.fg).bg(theme.panel_alt))
        };
        let tail = "enter keep · esc clear";
        let used = 2 + text.chars().count() + 1 + tail.chars().count();
        let pad = (area.width as usize).saturating_sub(used);
        let line = Line::from(vec![
            Span::styled("/", Style::default().fg(theme.accent).bg(theme.panel_alt).add_modifier(Modifier::BOLD)),
            Span::styled(" ", bar),
            Span::styled(text, text_style),
            Span::styled("▌", Style::default().fg(theme.accent).bg(theme.panel_alt)),
            Span::styled(" ".repeat(pad), bar),
            Span::styled(tail, Style::default().fg(theme.faint).bg(theme.panel_alt)),
        ]);
        frame.render_widget(Paragraph::new(line).style(bar), area);
    } else if !query.is_empty() {
        // Filter validated with Enter: keep the entered text visible, dimmed.
        let line = Line::from(vec![
            Span::styled("[/] ", theme.accent_bold),
            Span::styled(query.to_string(), theme.dim_style),
            Span::styled("   esc close", theme.faint_style),
        ]);
        frame.render_widget(Paragraph::new(line).style(theme.base), area);
    } else {
        let line = Line::from(vec![
            Span::styled("[/] ", theme.accent_bold),
            Span::styled(format!("filter {noun}"), Style::default().fg(theme.fg)),
            Span::styled("   esc close", theme.faint_style),
        ]);
        frame.render_widget(Paragraph::new(line).style(theme.base), area);
    }
}

pub fn draw(frame: &mut Frame, app: &App) {
    app.clear_click_regions();
    let theme = &app.theme;
    frame.render_widget(
        ratatui::widgets::Block::default().style(theme.base),
        frame.area(),
    );

    if app.screen == Screen::SessionAttach {
        session_attach::draw(frame, app, frame.area());
        if app.active_popup.is_some() {
            popup::draw(frame, app);
        }
        if app.palette.is_some() {
            palette::draw(frame, app);
        }
        return;
    }

    let outer = Layout::vertical([
        Constraint::Length(1), // header
        Constraint::Length(1), // spacer
        Constraint::Min(0),   // content
        Constraint::Length(1), // keybinding hints
        Constraint::Length(1), // spacer
        Constraint::Length(1), // status bar
    ])
    .split(frame.area());

    header::draw(frame, app, outer[0]);

    if app.layout_editor_active {
        layout_editor::draw(frame, app, outer[2]);
    } else {
        match app.screen {
            Screen::Dashboard => draw_dashboard(frame, app, outer[2]),
            Screen::SpecBrowser => spec_browser::draw(frame, app, outer[2]),
            Screen::Constitution => spec_browser::draw_constitution(frame, app, outer[2]),
            Screen::Settings => settings::draw(frame, app, outer[2]),
            Screen::SessionAttach => unreachable!(),
        }
    }

    statusbar::draw_hints(frame, app, outer[3]);
    statusbar::draw_statusbar(frame, app, outer[5]);

    // Popups render on top
    if app.active_popup.is_some() {
        popup::draw(frame, app);
    }

    // Command palette on top of everything
    if app.palette.is_some() {
        palette::draw(frame, app);
    }
}

fn draw_dashboard(frame: &mut Frame, app: &App, area: Rect) {
    use crate::app::{ClickAction, Pane};
    match app.layout {
        DashboardLayout::Overview => {
            let cols =
                Layout::horizontal([Constraint::Length(38), Constraint::Min(0)]).split(area);

            app.register_click(cols[0], ClickAction::FocusPane(Pane::FeatureList));
            feature_list::draw(frame, app, cols[0]);

            let right =
                Layout::vertical([Constraint::Length(13), Constraint::Min(0)]).split(cols[1]);

            app.register_click(right[0], ClickAction::FocusPane(Pane::Workflow));
            app.register_click(right[1], ClickAction::FocusPane(Pane::AgentOutput));
            workflow::draw(frame, app, right[0]);
            agent_output::draw(frame, app, right[1]);
        }
        DashboardLayout::Coding => {
            let cols = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(area);

            app.register_click(cols[0], ClickAction::FocusPane(Pane::SpecBrowser));
            app.register_click(cols[1], ClickAction::FocusPane(Pane::AgentOutput));
            spec_browser::draw_inline(frame, app, cols[0]);
            agent_output::draw(frame, app, cols[1]);
        }
        DashboardLayout::Audit => {
            let cols = Layout::horizontal([Constraint::Percentage(54), Constraint::Min(0)])
                .split(area);

            app.register_click(cols[0], ClickAction::FocusPane(Pane::ExtensionsPresets));
            app.register_click(cols[1], ClickAction::FocusPane(Pane::Constitution));
            extensions_presets::draw(frame, app, cols[0]);
            draw_constitution_inline(frame, app, cols[1]);
        }
        DashboardLayout::Custom => {
            draw_custom_layout(frame, app, area);
        }
    }
}

fn draw_custom_layout(frame: &mut Frame, app: &App, area: Rect) {
    let rects = custom_pane_rects(&app.custom_layout, area);
    if rects.is_empty() {
        let theme = &app.theme;
        let msg = ratatui::widgets::Paragraph::new(ratatui::text::Line::from(
            ratatui::text::Span::styled("  No visible panes. Press 4 or s for settings.", theme.faint_style),
        ))
        .style(theme.base);
        frame.render_widget(msg, area);
        return;
    }
    for (kind, rect) in rects {
        draw_pane_by_kind(frame, app, kind, rect);
    }
}

/// Compute the rects for the visible panes of a custom layout: first visible
/// pane is a sidebar, the rest stack vertically by their relative size.
/// Shared by the dashboard renderer and the layout-editor live preview.
pub(crate) fn custom_pane_rects(
    custom_layout: &spectatui_core::layout::CustomLayout,
    area: Rect,
) -> Vec<(spectatui_core::layout::PaneKind, Rect)> {
    let visible = custom_layout.visible_panes();
    if visible.is_empty() {
        return Vec::new();
    }
    if visible.len() == 1 {
        return vec![(visible[0].kind, area)];
    }

    let cols = Layout::horizontal([Constraint::Length(38), Constraint::Min(0)]).split(area);
    let mut out = vec![(visible[0].kind, cols[0])];

    let rest = &visible[1..];
    let total_size: u8 = rest.iter().map(|p| p.size).sum();
    let constraints: Vec<Constraint> = rest
        .iter()
        .map(|p| {
            if total_size > 0 {
                Constraint::Ratio(p.size as u32, total_size as u32)
            } else {
                Constraint::Min(0)
            }
        })
        .collect();

    let right_rows = Layout::vertical(constraints).split(cols[1]);
    for (i, pane) in rest.iter().enumerate() {
        if i < right_rows.len() {
            out.push((pane.kind, right_rows[i]));
        }
    }
    out
}

fn draw_pane_by_kind(
    frame: &mut Frame,
    app: &App,
    kind: spectatui_core::layout::PaneKind,
    area: Rect,
) {
    use spectatui_core::layout::PaneKind;
    match kind {
        PaneKind::FeatureList => feature_list::draw(frame, app, area),
        PaneKind::WorkflowTimeline => workflow::draw(frame, app, area),
        PaneKind::AgentOutput => agent_output::draw(frame, app, area),
        PaneKind::SpecBrowser => spec_browser::draw_inline(frame, app, area),
        PaneKind::Constitution => draw_constitution_inline(frame, app, area),
        PaneKind::ExtensionsPresets => extensions_presets::draw(frame, app, area),
        PaneKind::CliOutputLog => draw_cli_output_pane(frame, app, area),
    }
}

fn draw_cli_output_pane(frame: &mut Frame, app: &App, area: Rect) {
    use ratatui::text::{Line, Span};
    use ratatui::widgets::{Block, BorderType, Borders, Paragraph, Wrap};

    let theme = &app.theme;

    let title = Line::from(vec![
        Span::styled("─┤ ", theme.border_unfocused),
        Span::styled("CLI Output", theme.title_unfocused),
        Span::styled(" ├", theme.border_unfocused),
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme.border_unfocused)
        .title(title)
        .padding(PANEL_PADDING);

    let output = app
        .cli_job
        .as_ref()
        .map(|j| j.output.as_str())
        .unwrap_or("No CLI output.");

    let lines: Vec<Line> = output
        .lines()
        .map(|l| Line::from(Span::styled(l.to_string(), theme.dim_style)))
        .collect();

    let content = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false })
        .style(theme.base);
    frame.render_widget(content, area);
}

fn draw_constitution_inline(frame: &mut Frame, app: &App, area: Rect) {
    use ratatui::text::{Line, Span};
    use ratatui::widgets::{Block, BorderType, Borders, Paragraph, Wrap};

    let theme = &app.theme;
    let border_style = theme.border_unfocused;
    let title_style = theme.title_unfocused;

    let title = Line::from(vec![
        Span::styled("─┤ ", border_style),
        Span::styled("constitution.md", title_style),
        Span::styled(" ├", border_style),
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(border_style)
        .title(title)
        .padding(PANEL_PADDING);

    let content_text = app
        .constitution_content()
        .unwrap_or_else(|| "No constitution found.".to_string());

    let lines: Vec<Line> = content_text
        .lines()
        .map(|l| render_md_line(l, theme))
        .collect();

    let content = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false })
        .style(theme.base);

    frame.render_widget(content, area);
}

fn render_md_line<'a>(line: &str, theme: &crate::theme::Theme) -> ratatui::text::Line<'a> {
    use ratatui::style::Modifier;
    use ratatui::text::{Line, Span};

    if line.starts_with("# ") {
        Line::from(Span::styled(
            line[2..].to_string(),
            ratatui::style::Style::default()
                .fg(theme.fg)
                .add_modifier(Modifier::BOLD),
        ))
    } else if line.starts_with("## ") {
        Line::from(vec![
            Span::styled("▍ ", theme.accent_style),
            Span::styled(
                line[3..].to_string(),
                ratatui::style::Style::default()
                    .fg(theme.fg)
                    .add_modifier(Modifier::BOLD),
            ),
        ])
    } else if line.starts_with("### ") {
        Line::from(Span::styled(
            line[4..].to_string(),
            ratatui::style::Style::default()
                .fg(theme.fg)
                .add_modifier(Modifier::BOLD),
        ))
    } else if line.starts_with("- ") || line.starts_with("* ") {
        Line::from(vec![
            Span::styled("  • ", theme.accent_style),
            Span::styled(line[2..].to_string(), theme.dim_style),
        ])
    } else if line.trim().is_empty() {
        Line::default()
    } else {
        Line::from(Span::styled(line.to_string(), theme.dim_style))
    }
}
