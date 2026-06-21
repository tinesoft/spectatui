use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph};
use ratatui::Frame;

use crate::app::App;

pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;

    let title = Line::from(vec![
        Span::styled("┤ ", theme.border_focused),
        Span::styled("Panes", theme.title_focused),
        Span::styled(" ├", theme.border_focused),
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme.border_focused)
        .title(title);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let cols = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(inner);

    draw_pane_list(frame, app, cols[0]);
    draw_preview(frame, app, cols[1]);
}

fn draw_pane_list(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let mut lines: Vec<Line> = vec![Line::default()];

    let mut sorted_panes: Vec<(usize, &spectatui_core::layout::PaneConfig)> =
        app.custom_layout.panes.iter().enumerate().collect();
    sorted_panes.sort_by_key(|(_, p)| p.order);

    for (list_idx, (_real_idx, pane)) in sorted_panes.iter().enumerate() {
        let selected = list_idx == app.layout_editor_index;
        let vis_icon = if pane.visible { "◉" } else { "○" };

        let sel_bar = if selected {
            Span::styled(" ▌", theme.accent_style)
        } else {
            Span::raw("  ")
        };

        let vis_style = if pane.visible {
            theme.good_style
        } else {
            theme.faint_style
        };

        let size_filled = pane.size as usize;
        let size_empty = 4usize.saturating_sub(size_filled);
        let size_bar = format!("{}{}", "▰".repeat(size_filled), "▱".repeat(size_empty));

        let row_style = if selected {
            Style::default().fg(theme.sel_fg).bg(theme.sel)
        } else {
            Style::default().fg(theme.fg)
        };

        let vis_label = if pane.visible {
            let vis_idx = app
                .custom_layout
                .panes
                .iter()
                .filter(|q| q.visible && q.order <= pane.order)
                .count();
            format!("{vis_idx}")
        } else {
            "·".to_string()
        };

        lines.push(Line::from(vec![
            sel_bar,
            Span::styled(format!(" {vis_icon}"), vis_style),
            Span::styled(format!(" {vis_label} "), theme.faint_style),
            Span::styled(pane.kind.label().to_string(), row_style),
            Span::styled("  size ", theme.dim_style),
            Span::styled(size_bar, theme.dim_style),
        ]));

        // Row list begins after the leading blank line (lines[0]).
        let row_y = area.y + 1 + list_idx as u16;
        app.register_click(
            Rect::new(area.x, row_y, area.width, 1),
            crate::app::ClickAction::LayoutEditorSelect(list_idx),
        );
    }

    lines.push(Line::default());
    lines.push(Line::from(vec![
        Span::styled("  Space", theme.accent_bold),
        Span::styled(" show/hide  ", theme.dim_style),
        Span::styled("< >", theme.accent_bold),
        Span::styled(" reorder", theme.dim_style),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  + -", theme.accent_bold),
        Span::styled(" resize  ", theme.dim_style),
        Span::styled("Enter", theme.accent_bold),
        Span::styled(" apply  ", theme.dim_style),
        Span::styled("Esc", theme.accent_bold),
        Span::styled(" back", theme.dim_style),
    ]));

    let content = Paragraph::new(lines).style(theme.base);
    frame.render_widget(content, area);
}

fn draw_preview(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;

    let title = Line::from(Span::styled("Live preview", theme.accent_bold));

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme.border_unfocused)
        .title(title);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let visible = app.custom_layout.visible_panes();
    if visible.is_empty() {
        let content = Paragraph::new(vec![
            Line::default(),
            Line::from(Span::styled("  toggle a pane on to preview the grid.", theme.faint_style)),
        ])
        .style(theme.base);
        frame.render_widget(content, inner);
        return;
    }

    // Caption row, then a real proportional mini-grid below it.
    let caption = Rect::new(inner.x, inner.y, inner.width, 1);
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            format!("  {} panes · custom grid", visible.len()),
            theme.dim_style,
        )))
        .style(theme.base),
        caption,
    );

    let grid_area = Rect::new(
        inner.x + 1,
        inner.y + 2,
        inner.width.saturating_sub(2),
        inner.height.saturating_sub(3),
    );
    if grid_area.width < 4 || grid_area.height < 3 {
        return;
    }

    // The selected pane (from the sorted list) for highlight.
    let mut sorted: Vec<&spectatui_core::layout::PaneConfig> = app.custom_layout.panes.iter().collect();
    sorted.sort_by_key(|p| p.order);
    let selected_kind = sorted
        .get(app.layout_editor_index)
        .map(|p| p.kind);

    for (kind, rect) in super::custom_pane_rects(&app.custom_layout, grid_area) {
        let is_sel = Some(kind) == selected_kind;
        let border = if is_sel { theme.border_focused } else { theme.border_unfocused };
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(border);
        let box_inner = block.inner(rect);
        frame.render_widget(block, rect);
        let label_style = if is_sel { theme.accent_bold } else { theme.dim_style };
        let bg = if is_sel {
            Style::default().bg(theme.sel)
        } else {
            theme.base
        };
        // Fill selected pane interior with sel bg for emphasis.
        if is_sel {
            for y in box_inner.y..box_inner.y + box_inner.height {
                frame.render_widget(
                    Paragraph::new(Line::from(Span::styled(" ".repeat(box_inner.width as usize), bg))),
                    Rect::new(box_inner.x, y, box_inner.width, 1),
                );
            }
        }
        if box_inner.height >= 1 {
            frame.render_widget(
                Paragraph::new(Line::from(Span::styled(format!(" {}", kind.label()), label_style))),
                Rect::new(box_inner.x, box_inner.y, box_inner.width, 1),
            );
        }
    }
}
