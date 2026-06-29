use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::app::{App, PopupKind, Screen};

pub fn draw_hints(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;

    let hints: Vec<(&str, &str)> = if app.filter_active {
        vec![
            ("type", "filter list"),
            ("↑↓", "select"),
            ("enter", "keep"),
            ("esc", "clear"),
        ]
    } else if app.layout_editor_active {
        vec![
            ("↑↓", "select pane"),
            ("space", "show/hide"),
            ("< >", "reorder"),
            ("+ -", "resize"),
            ("enter", "apply"),
            ("esc", "back"),
        ]
    } else if app.palette.is_some() {
        vec![
            ("↑↓", "navigate"),
            ("enter", "run"),
            ("esc", "close"),
            ("type", "filter"),
        ]
    } else if app.active_popup == Some(PopupKind::Integrations) {
        vec![
            ("↑↓", "select"),
            ("/", "filter"),
            ("i", "install"),
            ("x", "uninstall"),
            ("d", "default"),
            ("s", "switch"),
            ("esc", "close"),
        ]
    } else if app.active_popup == Some(PopupKind::Workflows) {
        vec![
            ("↑↓", "select"),
            ("/", "filter"),
            ("r", "run"),
            ("s", "status"),
            ("a", "add"),
            ("esc", "close"),
        ]
    } else if matches!(app.active_popup, Some(PopupKind::Extensions) | Some(PopupKind::Presets)) {
        vec![
            ("tab", "ext/presets"),
            ("↑↓", "select"),
            ("/", "filter"),
            ("a", "add"),
            ("x", "remove"),
            ("esc", "close"),
        ]
    } else if app.active_popup == Some(PopupKind::Features) {
        vec![
            ("↑↓", "select"),
            ("enter", "jump to feature"),
            ("esc", "close"),
        ]
    } else if app.active_popup == Some(PopupKind::CliConfirm) {
        vec![
            ("enter", "run command"),
            ("esc", "cancel"),
            ("f", "toggle --force"),
        ]
    } else if app.active_popup == Some(PopupKind::CliOutput) {
        vec![("esc", "close & refresh")]
    } else if app.active_popup.is_some() {
        // Extensions/Presets popup and any other popup use the generic fallback.
        vec![
            ("↑↓", "navigate"),
            ("enter", "select"),
            ("esc", "close"),
        ]
    } else {
        match app.screen {
            Screen::Dashboard => vec![
                ("tab", "focus"),
                ("↑↓", "select"),
                ("enter", "open"),
                ("1·2·3", "layout"),
                ("t", "theme"),
                (":", "commands"),
                ("?", "help"),
                ("q", "quit"),
            ],
            Screen::SpecBrowser => vec![
                ("tab", "switch doc"),
                ("↑↓", "scroll"),
                ("esc", "back"),
            ],
            Screen::Constitution => vec![
                ("↑↓", "navigate"),
                ("enter", "toggle"),
                ("esc", "back"),
            ],
            Screen::Settings => vec![
                ("↑↓", "navigate"),
                ("enter", "toggle"),
                ("esc", "back"),
            ],
            Screen::SessionAttach => vec![
                ("ctrl-b d", "detach"),
                ("esc", "back"),
                ("↑↓", "scroll"),
            ],
        }
    };

    let mut spans: Vec<Span> = vec![Span::styled(" ", theme.base)];

    for (i, (key, desc)) in hints.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled(" · ", theme.faint_style));
        }
        spans.push(Span::styled(key.to_string(), theme.accent_bold));
        spans.push(Span::styled(format!(" {desc}"), theme.dim_style));
    }

    let line = Line::from(spans);
    let hint_bar = Paragraph::new(line).style(theme.base);
    frame.render_widget(hint_bar, area);
}

pub fn draw_statusbar(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;

    use spectatui_core::speckit::InstallStatus;

    let int_count = app.project.integrations.iter().filter(|i| i.installed).count();
    let feat_count = app.project.features.len();
    let ext_count = app.project.extensions.iter().filter(|e| e.status != InstallStatus::Available).count();
    let preset_count = app.project.presets.iter().filter(|p| p.status != InstallStatus::Available).count();
    let wf_count = app.project.workflows.iter().filter(|w| w.installed).count();

    use crate::app::{ClickAction, PopupKind};

    let mut spans: Vec<Span> = vec![Span::styled(" ", theme.statusbar_style)];

    let stats: &[(&str, usize, &str, &str, ClickAction)] = &[
        ("◈", int_count, "integrations", "i", ClickAction::OpenPopup(PopupKind::Integrations)),
        ("❖", feat_count, "features", "f", ClickAction::OpenPopup(PopupKind::Features)),
        ("◰", ext_count, "extensions", "e", ClickAction::OpenPopup(PopupKind::Extensions)),
        ("≣", preset_count, "presets", "p", ClickAction::OpenPopup(PopupKind::Presets)),
        ("◷", wf_count, "workflows", "w", ClickAction::OpenPopup(PopupKind::Workflows)),
    ];

    // Track the cell x to register clickable regions per stat.
    let mut x = area.x + 1;
    for (i, (icon, count, label, key, action)) in stats.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled("   ", theme.statusbar_style));
            x += 3;
        }
        spans.push(Span::styled(format!("{icon} "), theme.accent_style));
        spans.push(Span::styled(
            format!("{count} {label} "),
            theme.statusbar_style,
        ));
        spans.push(Span::styled(*key, theme.faint_style));
        // width = "{icon} " (2) + "{count} {label} " + key (1)
        let w = 2 + format!("{count} {label} ").chars().count() as u16 + key.chars().count() as u16;
        app.register_click(Rect::new(x, area.y, w, 1), *action);
        x += w;
    }

    let left_width: u16 = spans.iter().map(|s| s.width() as u16).sum();

    let indexing_text = if app.indexing {
        let spinner_chars = "⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏";
        let idx = (app.indexing_tick as usize) % spinner_chars.chars().count();
        let ch = spinner_chars.chars().nth(idx).unwrap_or('⠋');
        format!("{ch} Indexing…  ")
    } else {
        String::new()
    };

    let settings_text = "⚙ Settings s ";
    let right_len = indexing_text.len() as u16 + settings_text.len() as u16;
    let pad = area.width.saturating_sub(left_width + right_len);
    spans.push(Span::styled(
        " ".repeat(pad as usize),
        theme.statusbar_style,
    ));

    if app.indexing {
        spans.push(Span::styled(indexing_text, theme.warn_style));
    }

    spans.push(Span::styled("⚙ ", theme.accent_style));
    spans.push(Span::styled("Settings ", theme.statusbar_style));
    spans.push(Span::styled("s ", theme.faint_style));

    let gear_w = settings_text.chars().count() as u16;
    let gear_x = area.x + area.width.saturating_sub(gear_w);
    app.register_click(
        Rect::new(gear_x, area.y, gear_w, 1),
        crate::app::ClickAction::OpenSettings,
    );

    let line = Line::from(spans);
    let statusbar = Paragraph::new(line).style(theme.statusbar_style);
    frame.render_widget(statusbar, area);
}
