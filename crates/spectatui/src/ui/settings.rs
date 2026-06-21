use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph};
use ratatui::Frame;

use crate::app::{App, SettingsRow};

pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;

    let title = Line::from(vec![
        Span::styled("┤ ", theme.border_focused),
        Span::styled("Settings ⚙", theme.title_focused),
        Span::styled(" ├", theme.border_focused),
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme.border_focused)
        .title(title);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut lines: Vec<Line> = vec![
        Line::default(),
        Line::from(Span::styled(
            "  Adjust appearance, layout, and CLI behaviour. Persisted to config.toml.",
            theme.dim_style,
        )),
        Line::default(),
    ];

    // Content rows begin after the 3 header lines rendered above.
    let row0_y = inner.y + 3;
    let label_width: usize = 24;
    let value_col = 2 + label_width as u16; // sel_bar(2) + " " + label area

    for (i, row) in SettingsRow::ALL.iter().enumerate() {
        let selected = i == app.settings_index;
        let row_style = if selected {
            Style::default().fg(theme.sel_fg).bg(theme.sel)
        } else {
            Style::default().fg(theme.fg)
        };

        let sel_bar = if selected {
            Span::styled(" ▌", theme.accent_style)
        } else {
            Span::raw("  ")
        };

        let label = row.label();
        let pad = label_width.saturating_sub(label.len());

        let mut spans = vec![
            sel_bar,
            Span::styled(format!(" {label}"), row_style),
            Span::styled(" ".repeat(pad), row_style),
        ];

        let opts = row.options();
        let row_y = row0_y + (i as u16) * 2;

        if !opts.is_empty() {
            // Inline selectable chips, active one highlighted.
            let current = app.settings_value_str(*row);
            let mut chip_x = inner.x + value_col;
            for (oi, opt) in opts.iter().enumerate() {
                let active = *opt == current;
                let chip = format!(" {opt} ");
                let chip_style = if active {
                    Style::default().fg(theme.accent).bg(theme.panel_alt)
                } else {
                    theme.dim_style
                };
                spans.push(Span::styled(chip.clone(), chip_style));
                spans.push(Span::raw(" "));
                app.register_click(
                    Rect::new(chip_x, row_y, chip.len() as u16, 1),
                    crate::app::ClickAction::SettingsChip(*row, oi),
                );
                chip_x += chip.len() as u16 + 1;
            }
        } else {
            // Value-only / action rows.
            let value = app.settings_value_str(*row);
            let is_action = matches!(row, SettingsRow::CustomizePanes | SettingsRow::AttachSession);
            let value_style = if is_action {
                theme.accent_bold
            } else {
                theme.info_style
            };
            spans.push(Span::styled(value, value_style));
        }

        lines.push(Line::from(spans));
        lines.push(Line::default());

        // Whole-row click selects the row.
        app.register_click(
            Rect::new(inner.x, row_y, inner.width, 1),
            crate::app::ClickAction::SettingsSelect(i),
        );
    }

    lines.push(Line::default());
    lines.push(Line::from(vec![
        Span::styled("  ←→", theme.accent_bold),
        Span::styled(" / ", theme.faint_style),
        Span::styled("Enter", theme.accent_bold),
        Span::styled(" cycle value  ", theme.dim_style),
        Span::styled("Esc", theme.accent_bold),
        Span::styled(" back", theme.dim_style),
    ]));

    let content = Paragraph::new(lines).style(theme.base);
    frame.render_widget(content, inner);
}
