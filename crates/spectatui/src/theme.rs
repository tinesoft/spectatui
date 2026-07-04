#![allow(dead_code)]

use ratatui::style::{Color, Modifier, Style};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeMode {
    Dark,
    Light,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Accent {
    Indigo,
    Teal,
    Amber,
}

impl Accent {
    pub fn next(self) -> Self {
        match self {
            Self::Indigo => Self::Teal,
            Self::Teal => Self::Amber,
            Self::Amber => Self::Indigo,
        }
    }

    fn color(self, mode: ThemeMode) -> Color {
        match (self, mode) {
            (Self::Indigo, ThemeMode::Dark) => Color::Rgb(0x93, 0xa4, 0xff),
            (Self::Indigo, ThemeMode::Light) => Color::Rgb(0x51, 0x59, 0xd4),
            (Self::Teal, ThemeMode::Dark) => Color::Rgb(0x5f, 0xd6, 0xbf),
            (Self::Teal, ThemeMode::Light) => Color::Rgb(0x1c, 0x96, 0x85),
            (Self::Amber, ThemeMode::Dark) => Color::Rgb(0xe6, 0xb5, 0x52),
            (Self::Amber, ThemeMode::Light) => Color::Rgb(0xb0, 0x74, 0x14),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Theme {
    pub bg: Color,
    pub panel: Color,
    pub panel_alt: Color,
    pub fg: Color,
    pub dim: Color,
    pub faint: Color,
    pub border: Color,
    pub sel: Color,
    pub sel_fg: Color,
    pub good: Color,
    pub warn: Color,
    pub bad: Color,
    pub info: Color,
    pub header_bg: Color,
    pub accent: Color,

    // Pre-built styles
    pub base: Style,
    pub title_focused: Style,
    pub title_unfocused: Style,
    pub border_focused: Style,
    pub border_unfocused: Style,
    pub selected: Style,
    pub dim_style: Style,
    pub faint_style: Style,
    pub good_style: Style,
    pub warn_style: Style,
    pub bad_style: Style,
    pub info_style: Style,
    pub header_style: Style,
    pub accent_style: Style,
    pub accent_bold: Style,
    pub statusbar_style: Style,
}

impl Theme {
    pub fn new(mode: ThemeMode, accent: Accent) -> Self {
        let (
            bg,
            panel,
            panel_alt,
            fg,
            dim,
            faint,
            border,
            sel,
            sel_fg,
            good,
            warn,
            bad,
            info,
            header_bg,
        ) = match mode {
            ThemeMode::Dark => (
                Color::Rgb(0x10, 0x10, 0x13),
                Color::Rgb(0x16, 0x16, 0x1b),
                Color::Rgb(0x1c, 0x1c, 0x23),
                Color::Rgb(0xd7, 0xd7, 0xdc),
                Color::Rgb(0x8b, 0x8b, 0x95),
                Color::Rgb(0x5b, 0x5b, 0x65),
                Color::Rgb(0x2c, 0x2c, 0x35),
                Color::Rgb(0x23, 0x23, 0x2e),
                Color::Rgb(0xf0, 0xf0, 0xf5),
                Color::Rgb(0x84, 0xd4, 0x8f),
                Color::Rgb(0xe3, 0xb6, 0x73),
                Color::Rgb(0xef, 0x8c, 0x7d),
                Color::Rgb(0x7c, 0xc2, 0xe8),
                Color::Rgb(0x0b, 0x0b, 0x0e),
            ),
            ThemeMode::Light => (
                Color::Rgb(0xf4, 0xf1, 0xea),
                Color::Rgb(0xfb, 0xf9, 0xf4),
                Color::Rgb(0xef, 0xec, 0xe4),
                Color::Rgb(0x2c, 0x2a, 0x26),
                Color::Rgb(0x76, 0x72, 0x6a),
                Color::Rgb(0xa8, 0xa3, 0x97),
                Color::Rgb(0xdd, 0xd8, 0xcc),
                Color::Rgb(0xe8, 0xe3, 0xd6),
                Color::Rgb(0x1b, 0x1a, 0x17),
                Color::Rgb(0x2f, 0x8a, 0x3f),
                Color::Rgb(0xa9, 0x70, 0x1a),
                Color::Rgb(0xcb, 0x53, 0x41),
                Color::Rgb(0x2f, 0x76, 0xa8),
                Color::Rgb(0xeb, 0xe7, 0xdd),
            ),
        };

        let accent_color = accent.color(mode);

        Theme {
            bg,
            panel,
            panel_alt,
            fg,
            dim,
            faint,
            border,
            sel,
            sel_fg,
            good,
            warn,
            bad,
            info,
            header_bg,
            accent: accent_color,

            base: Style::default().fg(fg).bg(bg),
            title_focused: Style::default()
                .fg(accent_color)
                .add_modifier(Modifier::BOLD),
            title_unfocused: Style::default().fg(dim),
            border_focused: Style::default().fg(accent_color),
            border_unfocused: Style::default().fg(border),
            selected: Style::default().fg(sel_fg).bg(sel),
            dim_style: Style::default().fg(dim),
            faint_style: Style::default().fg(faint),
            good_style: Style::default().fg(good),
            warn_style: Style::default().fg(warn),
            bad_style: Style::default().fg(bad),
            info_style: Style::default().fg(info),
            header_style: Style::default().fg(fg).bg(header_bg),
            accent_style: Style::default().fg(accent_color),
            accent_bold: Style::default()
                .fg(accent_color)
                .add_modifier(Modifier::BOLD),
            statusbar_style: Style::default().fg(dim).bg(panel),
        }
    }

    pub fn stage_badge(&self, label: &str, mode: ThemeMode) -> Style {
        let (fg_color, bg_color) = match (label, mode) {
            ("cons", ThemeMode::Dark) => {
                (Color::Rgb(0x7f, 0xd8, 0xc2), Color::Rgb(0x0f, 0x3a, 0x32))
            }
            ("cons", ThemeMode::Light) => {
                (Color::Rgb(0x0b, 0x60, 0x52), Color::Rgb(0xcf, 0xee, 0xe6))
            }
            ("spec", ThemeMode::Dark) => {
                (Color::Rgb(0xf0, 0xa4, 0x7c), Color::Rgb(0x43, 0x21, 0x0f))
            }
            ("spec", ThemeMode::Light) => {
                (Color::Rgb(0xa5, 0x51, 0x2b), Color::Rgb(0xf6, 0xdd, 0xcf))
            }
            ("clar", ThemeMode::Dark) => {
                (Color::Rgb(0xe8, 0xa6, 0xc5), Color::Rgb(0x3d, 0x1b, 0x2d))
            }
            ("clar", ThemeMode::Light) => {
                (Color::Rgb(0x9c, 0x3a, 0x68), Color::Rgb(0xf4, 0xd9, 0xe6))
            }
            ("plan", ThemeMode::Dark) => {
                (Color::Rgb(0xf0, 0xc8, 0x79), Color::Rgb(0x42, 0x33, 0x0c))
            }
            ("plan", ThemeMode::Light) => {
                (Color::Rgb(0x8a, 0x63, 0x10), Color::Rgb(0xf3, 0xe7, 0xc4))
            }
            ("task", ThemeMode::Dark) => {
                (Color::Rgb(0x86, 0xb8, 0xec), Color::Rgb(0x10, 0x31, 0x4f))
            }
            ("task", ThemeMode::Light) => {
                (Color::Rgb(0x2d, 0x6b, 0xa3), Color::Rgb(0xd6, 0xe7, 0xf6))
            }
            ("anly", ThemeMode::Dark) => {
                (Color::Rgb(0xc4, 0xa7, 0xf0), Color::Rgb(0x2c, 0x1f, 0x47))
            }
            ("anly", ThemeMode::Light) => {
                (Color::Rgb(0x5d, 0x3a, 0xa0), Color::Rgb(0xe7, 0xdd, 0xf6))
            }
            ("impl", ThemeMode::Dark) => {
                (Color::Rgb(0x90, 0xd8, 0x90), Color::Rgb(0x12, 0x3a, 0x1d))
            }
            ("impl", ThemeMode::Light) => {
                (Color::Rgb(0x2f, 0x8a, 0x3f), Color::Rgb(0xd4, 0xee, 0xd9))
            }
            _ => (self.faint, self.bg),
        };
        Style::default().fg(fg_color).bg(bg_color)
    }

    pub fn stepper_done_style(&self, mode: ThemeMode) -> Style {
        let bg = match mode {
            ThemeMode::Dark => Color::Rgb(0x16, 0x27, 0x1b),
            ThemeMode::Light => Color::Rgb(0xdc, 0xef, 0xdf),
        };
        Style::default().fg(self.good).bg(bg)
    }
}
