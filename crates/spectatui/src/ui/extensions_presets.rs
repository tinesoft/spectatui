use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph};
use ratatui::Frame;

use crate::app::{App, ExtTab};

/// Combined Extensions & Presets manager, rendered as a dashboard pane (the Audit
/// layout and the optional custom-layout pane). The standalone popups live in
/// `extensions.rs` / `presets.rs`; this tabbed view reuses their list renderers.
pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;

    let ext_count = app.project.extensions.len();
    let preset_count = app.project.presets.len();
    let active_tab = app.ext_tab;

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme.border_focused)
        .title(Line::from(vec![
            Span::styled("─┤ ", theme.border_focused),
            Span::styled("Extensions & Presets", theme.title_focused),
            Span::styled(" ├", theme.border_focused),
        ]));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.width < 10 || inner.height < 5 {
        return;
    }

    // Tab row on the first inner row, active tab highlighted.
    {
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

    // Content sits below the tab row and a blank line.
    let body = Rect::new(inner.x, inner.y + 2, inner.width, inner.height.saturating_sub(3));
    let cols =
        Layout::horizontal([Constraint::Percentage(45), Constraint::Percentage(55)]).split(body);

    match active_tab {
        ExtTab::Extensions => super::extensions::draw_list(frame, app, cols[0], cols[1]),
        ExtTab::Presets => super::presets::draw_list(frame, app, cols[0], cols[1]),
    }

    if area.height > 2 {
        let footer_y = area.y + area.height - 2;
        let footer_area = Rect::new(area.x + 2, footer_y, area.width.saturating_sub(4), 1);
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled("[enter] manage", theme.faint_style))),
            footer_area,
        );
    }
}
