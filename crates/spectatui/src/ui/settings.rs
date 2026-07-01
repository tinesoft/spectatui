use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph};
use ratatui::Frame;

use crate::app::{App, SettingsRow};

pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;

    let title = Line::from(vec![
        Span::styled("─┤ ", theme.border_focused),
        Span::styled("Settings", theme.title_focused),
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

    let mut lines: Vec<Line> = vec![
        Line::from(Span::styled(
            "  Adjust appearance, layout, and CLI behaviour. Persisted to config.toml.",
            theme.dim_style,
        )),
        Line::default(),
    ];

    // Content rows begin after the 2 header lines rendered above.
    let row0_y = inner.y + 2;
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

        if row.is_text() {
            // Inline-editable text field: a filled bar with a block cursor while editing.
            let editing = app.settings_editing == Some(i);
            let value = app.settings_value_str(*row);
            let bw = 24usize.max(value.chars().count() + 3);
            let field_bg = if editing {
                theme.panel_alt
            } else if selected {
                theme.sel
            } else {
                theme.panel
            };
            let value_style = if editing {
                Style::default().fg(theme.fg).bg(field_bg)
            } else {
                Style::default().fg(theme.info).bg(field_bg)
            };
            spans.push(Span::styled(format!(" {value}"), value_style));
            let mut used = 1 + value.chars().count();
            if editing {
                spans.push(Span::styled(
                    "█",
                    Style::default().fg(theme.accent).bg(field_bg),
                ));
                used += 1;
            }
            spans.push(Span::styled(
                " ".repeat(bw.saturating_sub(used)),
                Style::default().bg(field_bg),
            ));

            app.register_click(
                Rect::new(inner.x + value_col, row_y, bw as u16, 1),
                crate::app::ClickAction::SettingsEdit(i),
            );
        } else if !opts.is_empty() {
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
    let footer = if app.settings_editing.is_some() {
        vec![
            Span::styled("  [type]", theme.accent_bold),
            Span::styled(" edit   ", theme.dim_style),
            Span::styled("[enter]", theme.accent_bold),
            Span::styled(" save   ", theme.dim_style),
            Span::styled("[esc]", theme.accent_bold),
            Span::styled(" cancel", theme.dim_style),
        ]
    } else {
        vec![
            Span::styled("  [↑↓]", theme.accent_bold),
            Span::styled(" move   ", theme.dim_style),
            Span::styled("[enter/←→]", theme.accent_bold),
            Span::styled(" change   ", theme.dim_style),
            Span::styled("[esc]", theme.accent_bold),
            Span::styled(" back", theme.dim_style),
        ]
    };
    lines.push(Line::from(footer));

    let content = Paragraph::new(lines).style(theme.base);
    frame.render_widget(content, inner);
}
