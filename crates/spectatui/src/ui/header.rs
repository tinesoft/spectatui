use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::app::{App, Screen};

pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;

    let screen_name = if app.layout_editor_active {
        "Layout Editor"
    } else {
        match app.screen {
            Screen::Dashboard => "Dashboard",
            Screen::SpecBrowser => "Spec Browser",
            Screen::Constitution => "Constitution",
            Screen::Settings => "Settings",
            Screen::SessionAttach => "Attached",
        }
    };

    let project_name = app.project_name.as_str();

    let left = vec![
        Span::styled(" spectatui ", theme.accent_bold),
        Span::styled(
            "› ",
            ratatui::style::Style::default()
                .fg(theme.faint)
                .bg(theme.header_bg),
        ),
        Span::styled(
            format!(" {project_name}  "),
            ratatui::style::Style::default()
                .fg(theme.fg)
                .bg(theme.header_bg),
        ),
        Span::styled(
            &app.project_path,
            ratatui::style::Style::default()
                .fg(theme.dim)
                .bg(theme.header_bg),
        ),
    ];

    let theme_icon = match app.theme_mode {
        crate::theme::ThemeMode::Dark => "◖",
        crate::theme::ThemeMode::Light => "◗",
    };
    let right_text = format!(
        "{screen_name}   {theme_icon} {}  ◆ {} ",
        app.theme_label(),
        app.accent_label()
    );
    let right_width = right_text.len() as u16;

    let pad = area
        .width
        .saturating_sub(left.iter().map(|s| s.width() as u16).sum::<u16>() + right_width);

    let mut spans = left;
    spans.push(Span::styled(" ".repeat(pad as usize), theme.header_style));
    spans.push(Span::styled(
        right_text,
        ratatui::style::Style::default()
            .fg(theme.dim)
            .bg(theme.header_bg),
    ));

    let line = Line::from(spans);
    let header = Paragraph::new(line).style(theme.header_style);
    frame.render_widget(header, area);
}
