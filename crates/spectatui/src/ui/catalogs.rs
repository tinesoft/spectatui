use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Clear, Paragraph, Wrap};
use ratatui::Frame;

use spectatui_core::speckit::registry::CatalogTarget;

use crate::app::App;

const CATALOG_TARGETS: [CatalogTarget; 4] = [
    CatalogTarget::Extension,
    CatalogTarget::Preset,
    CatalogTarget::Integration,
    CatalogTarget::Workflow,
];

fn tab_label(target: CatalogTarget) -> &'static str {
    match target {
        CatalogTarget::Extension => "Extensions",
        CatalogTarget::Preset => "Presets",
        CatalogTarget::Integration => "Integrations",
        CatalogTarget::Workflow => "Workflows",
    }
}

pub fn draw(frame: &mut Frame, app: &App) {
    let theme = &app.theme;

    let full = frame.area();
    let w = full.width.saturating_sub(6).clamp(40, 108);
    let h = 30u16.min(full.height.saturating_sub(6)).max(8);

    let area = centered(w, h, full);
    frame.render_widget(Clear, area);

    let title_text = format!(
        "Catalogs  ·  {} sources across 4 types",
        app.catalog_sources.total()
    );
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

    // First inner row: either the kind-tab row, or the inline add-source form.
    // The add-form uses one extra row (input + hint + a blank spacer row)
    // before the list/detail body than the tab row does (tab + blank).
    let header_rows: u16 = if let Some(input) = &app.cat_add_input {
        draw_add_form(frame, app, inner, input);
        3
    } else {
        draw_tab_row(frame, app, inner);
        2
    };

    let list_w = (inner.width as f32 * 0.46) as u16;
    let body = Rect::new(
        inner.x,
        inner.y + header_rows,
        inner.width,
        inner.height.saturating_sub(header_rows + 1),
    );
    let cols = Layout::horizontal([Constraint::Length(list_w), Constraint::Min(0)]).split(body);

    draw_list(frame, app, cols[0]);

    for y in cols[1].y..cols[1].y + cols[1].height {
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled("│", theme.border_focused))),
            Rect::new(cols[1].x, y, 1, 1),
        );
    }

    let detail_area = Rect::new(
        cols[1].x + 1,
        cols[1].y,
        cols[1].width.saturating_sub(1),
        cols[1].height,
    );
    draw_detail(frame, app, detail_area);

    // The add-form has its own hint line under the input; the bottom footer
    // is only for the tab-row/list view.
    if area.height > 2 && app.cat_add_input.is_none() {
        let footer_y = area.y + area.height - 2;
        let footer_area = Rect::new(area.x + 3, footer_y, area.width.saturating_sub(6), 1);
        draw_footer(frame, app, footer_area);
    }
}

fn draw_tab_row(frame: &mut Frame, app: &App, inner: Rect) {
    let theme = &app.theme;
    let active = app.cat_tab;

    let mut spans: Vec<Span> = vec![Span::raw(" ")];
    let mut tx = inner.x + 1;
    for (i, target) in CATALOG_TARGETS.iter().enumerate() {
        if i > 0 {
            spans.push(Span::raw(" "));
            tx += 1;
        }
        let chip = format!(" {} ", tab_label(*target));
        let style = if *target == active {
            Style::default()
                .fg(theme.accent)
                .bg(theme.panel_alt)
                .add_modifier(Modifier::BOLD)
        } else {
            theme.dim_style
        };
        let chip_w = chip.chars().count() as u16;
        app.register_click(
            Rect::new(tx, inner.y, chip_w, 1),
            crate::app::ClickAction::SetCatalogTab(*target),
        );
        tx += chip_w;
        spans.push(Span::styled(chip, style));
    }
    frame.render_widget(
        Paragraph::new(Line::from(spans)).style(theme.base),
        Rect::new(inner.x, inner.y, inner.width, 1),
    );
}

fn draw_add_form(frame: &mut Frame, app: &App, inner: Rect, input: &str) {
    let theme = &app.theme;
    let bg = Style::default().bg(theme.panel_alt);
    let row = Rect::new(inner.x, inner.y, inner.width, 1);
    frame.render_widget(
        Paragraph::new(Line::from(Span::raw(" ".repeat(row.width as usize)))).style(bg),
        row,
    );

    let prefix = format!("Add {} catalog:  ", app.cat_tab.cli());
    let prefix_span_width = 1 + prefix.chars().count() as u16; // " {prefix}"
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            format!(" {prefix}"),
            Style::default().fg(theme.fg).bg(theme.panel_alt),
        )))
        .style(bg),
        row,
    );

    // Everything after the fixed prefix is the scrollable text viewport:
    // one column reserved on each side for a `‹`/`›` overflow indicator, so
    // long input stays fully reachable (by moving the cursor) even though
    // the box itself never resizes.
    let available = row.width.saturating_sub(prefix_span_width);
    let text_width = available.saturating_sub(2);
    let text_x = inner.x + prefix_span_width + 1;

    let chars: Vec<char> = input.chars().collect();
    let char_count = chars.len();
    let (offset, visible_end) =
        scrolled_visible_range(char_count, app.cat_add_cursor, text_width as usize);
    let visible = &chars[offset..visible_end];
    let cursor_in_slice = app.cat_add_cursor.saturating_sub(offset);

    if offset > 0 {
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled("‹", theme.faint_style))).style(bg),
            Rect::new(text_x - 1, inner.y, 1, 1),
        );
    }
    if offset + visible.len() < char_count {
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled("›", theme.faint_style))).style(bg),
            Rect::new(text_x + text_width, inner.y, 1, 1),
        );
    }

    let text_style = Style::default().fg(theme.fg).bg(theme.panel_alt);
    let mut spans = Vec::new();
    if cursor_in_slice < visible.len() {
        let before: String = visible[..cursor_in_slice].iter().collect();
        let cursor_char: String = visible[cursor_in_slice..=cursor_in_slice].iter().collect();
        let after: String = visible[cursor_in_slice + 1..].iter().collect();
        spans.push(Span::styled(before, text_style));
        spans.push(Span::styled(
            cursor_char,
            Style::default().fg(theme.panel_alt).bg(theme.fg),
        ));
        spans.push(Span::styled(after, text_style));
    } else {
        // Cursor is at (or past) the end of the visible slice — which only
        // happens at the true end of the string, since scroll_offset keeps
        // the cursor within the viewport otherwise.
        let all: String = visible.iter().collect();
        spans.push(Span::styled(all, text_style));
        spans.push(Span::styled(
            "█",
            Style::default()
                .fg(theme.fg)
                .bg(theme.panel_alt)
                .add_modifier(Modifier::BOLD),
        ));
    }
    frame.render_widget(
        Paragraph::new(Line::from(spans)).style(bg),
        Rect::new(text_x, inner.y, text_width, 1),
    );

    // One click region per visible character (mapped back to its absolute
    // index in the full string, not its position in the visible slice), so
    // the mouse can position the cursor anywhere — including inside text
    // that's currently scrolled off-screen, by first scrolling it into view.
    for k in 0..visible.len() {
        app.register_click(
            Rect::new(text_x + k as u16, inner.y, 1, 1),
            crate::app::ClickAction::SetCatalogAddCursor(offset + k),
        );
    }
    // Clicking at/after the end of the visible text (including the reserved
    // indicator column) lands at the end of the visible slice.
    let end_x = text_x + visible.len() as u16;
    if end_x < inner.x + inner.width {
        app.register_click(
            Rect::new(end_x, inner.y, inner.x + inner.width - end_x, 1),
            crate::app::ClickAction::SetCatalogAddCursor(offset + visible.len()),
        );
    }

    // Aligned with where the input box's text itself begins (`text_x`, past
    // the reserved left scroll-indicator column) — not the prefix label, and
    // not the popup's left edge.
    let hint_row = Rect::new(
        text_x,
        inner.y + 1,
        inner.width.saturating_sub(text_x - inner.x),
        1,
    );
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("url name [priority] · ", theme.faint_style),
            Span::styled("enter", theme.accent_bold),
            Span::styled(" to add · ", theme.faint_style),
            Span::styled("ctrl+c", theme.accent_bold),
            Span::styled(" to clear · ", theme.faint_style),
            Span::styled("esc", theme.accent_bold),
            Span::styled(" to cancel", theme.faint_style),
        ])),
        hint_row,
    );
    // A blank row separates the hint from the source list/detail below.
}

fn draw_list(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let mut lines: Vec<Line> = Vec::new();

    let items = app.current_catalog_list();
    if items.is_empty() {
        lines.extend(super::empty_list_lines(
            app,
            "No catalog sources configured",
        ));
    } else {
        let name_max = area.width.saturating_sub(10) as usize;

        let per_page = area.height as usize;
        let offset = super::scroll_offset(app.cat_index, per_page);
        for (i, src) in items.iter().enumerate().skip(offset).take(per_page) {
            let selected = i == app.cat_index;
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

            let dot = if src.install_allowed {
                Span::styled("● ", theme.good_style)
            } else {
                Span::styled("○ ", theme.faint_style)
            };

            let name = if src.name.chars().count() > name_max {
                let truncated: String = src.name.chars().take(name_max.saturating_sub(1)).collect();
                format!("{truncated}…")
            } else {
                src.name.clone()
            };

            let priority_text = src
                .priority
                .map(|p| format!("p{p}"))
                .unwrap_or_else(|| "—".to_string());

            lines.push(Line::from(vec![
                sel_bar,
                dot,
                Span::styled(name, row_style),
                Span::raw(" "),
                Span::styled(priority_text, theme.dim_style),
            ]));

            let row_y = area.y + (i - offset) as u16;
            if row_y < area.y + area.height {
                app.register_click(
                    Rect::new(area.x, row_y, area.width, 1),
                    crate::app::ClickAction::SelectCatalogSource(i),
                );
            }
        }
    }

    let list = Paragraph::new(lines).style(theme.base);
    frame.render_widget(list, area);
}

fn draw_detail(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;

    let items = app.current_catalog_list();
    let selected = items.get(app.cat_index);

    let mut lines = Vec::new();
    if let Some(src) = selected {
        lines.push(Line::from(vec![
            Span::styled(format!(" {} ", src.name), theme.accent_bold),
            Span::styled(
                format!(" ·  {} catalog", app.cat_tab.cli()),
                theme.faint_style,
            ),
        ]));
        lines.push(Line::default());

        for chunk in wrap_chars(&src.url, area.width.saturating_sub(2) as usize) {
            lines.push(Line::from(Span::styled(
                format!(" {chunk}"),
                theme.dim_style,
            )));
        }
        lines.push(Line::default());

        let priority_text = src
            .priority
            .map(|p| p.to_string())
            .unwrap_or_else(|| "not set".to_string());
        lines.push(Line::from(vec![
            Span::styled(" priority ", theme.faint_style),
            Span::styled(priority_text, theme.dim_style),
        ]));

        let (dot, label, style) = if src.install_allowed {
            ("● ", "install allowed", theme.good_style)
        } else {
            ("○ ", "discovery only", theme.faint_style)
        };
        lines.push(Line::from(vec![
            Span::styled(format!(" {dot}"), style),
            Span::styled(label, style),
        ]));
        lines.push(Line::default());
    }

    // Actions render even with no source selected — adding is always possible,
    // even when the current tab has zero configured sources.
    lines.push(Line::from(Span::styled(" Actions", theme.dim_style)));
    lines.push(action_line("a", "add source", theme));
    if selected.is_some() {
        lines.push(action_line("x", "remove source", theme));
    }
    lines.push(action_line("r", "refresh", theme));

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

fn wrap_chars(s: &str, width: usize) -> Vec<String> {
    if width == 0 {
        return vec![s.to_string()];
    }
    let chars: Vec<char> = s.chars().collect();
    chars.chunks(width).map(|c| c.iter().collect()).collect()
}

/// Footer for the tab-row/list view only — the add-form has its own hint
/// line under the input instead (see `draw_add_form`).
fn draw_footer(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let hints: &[(&str, &str)] = &[
        ("tab", "switch type"),
        ("\u{2191}\u{2193}", "select"),
        ("esc", "close"),
    ];

    let mut spans: Vec<Span> = Vec::new();
    for (i, (key, desc)) in hints.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled("  ·  ", theme.faint_style));
        }
        spans.push(Span::styled(*key, theme.accent_bold));
        spans.push(Span::raw(" "));
        spans.push(Span::styled(*desc, theme.dim_style));
    }
    frame.render_widget(Paragraph::new(Line::from(spans)).style(theme.base), area);
}

fn centered(w: u16, h: u16, r: Rect) -> Rect {
    let x = r.x + (r.width.saturating_sub(w)) / 2;
    let y = r.y + (r.height.saturating_sub(h)) / 2;
    Rect::new(x, y, w.min(r.width), h.min(r.height))
}

/// The visible `[start, end)` char range for the add-form's scrollable text
/// viewport, given the full char count, the absolute cursor position, and the
/// viewport width — pure and side-effect-free so it's unit-testable without a
/// `Frame`. Reuses `super::scroll_offset`'s "keep the selected index in view"
/// logic, which applies identically whether "selected" means a list row or a
/// cursor position in a string.
fn scrolled_visible_range(char_count: usize, cursor: usize, text_width: usize) -> (usize, usize) {
    let cursor = cursor.min(char_count);
    let offset = super::scroll_offset(cursor, text_width).min(char_count);
    let end = (offset + text_width).min(char_count);
    (offset, end)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scrolled_visible_range_shows_from_start_when_it_fits() {
        assert_eq!(scrolled_visible_range(10, 10, 20), (0, 10));
        assert_eq!(scrolled_visible_range(10, 0, 20), (0, 10));
    }

    #[test]
    fn scrolled_visible_range_scrolls_to_keep_cursor_at_end_visible() {
        // 30 chars, viewport of 10, cursor at the very end (30).
        let (start, end) = scrolled_visible_range(30, 30, 10);
        assert_eq!((start, end), (21, 30));
        assert_eq!(
            end - start,
            9,
            "9 chars visible, cursor sits right after them"
        );
    }

    #[test]
    fn scrolled_visible_range_scrolls_to_keep_cursor_at_start_visible() {
        let (start, end) = scrolled_visible_range(30, 0, 10);
        assert_eq!((start, end), (0, 10));
    }

    #[test]
    fn scrolled_visible_range_scrolls_as_cursor_moves_through_middle() {
        let (start, end) = scrolled_visible_range(30, 15, 10);
        // scroll_offset(15, 10) = 15 - 10 + 1 = 6
        assert_eq!((start, end), (6, 16));
        assert!(
            15 >= start && 15 < end,
            "cursor stays within the visible range"
        );
    }

    #[test]
    fn scrolled_visible_range_clamps_cursor_beyond_char_count() {
        assert_eq!(scrolled_visible_range(5, 999, 10), (0, 5));
    }
}
