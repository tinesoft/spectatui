use std::path::PathBuf;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::theme::{Accent, ThemeMode};
use spectatui_core::layout::CustomLayout;

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default = "default_accent")]
    pub accent: String,
    #[serde(default = "default_layout")]
    pub dashboard_layout: String,
    #[serde(default)]
    pub mouse_support: bool,
    #[serde(default = "default_true")]
    pub agent_tail_follow: bool,
    #[serde(default = "default_true")]
    pub confirm_before_force: bool,
    #[serde(default = "default_tmux_prefix")]
    pub tmux_prefix: String,
    #[serde(default = "default_config_location")]
    pub config_location: String,
    #[serde(default)]
    pub custom_layout: Option<CustomLayout>,
}

fn default_theme() -> String {
    "dark".to_string()
}
fn default_accent() -> String {
    "indigo".to_string()
}
fn default_layout() -> String {
    "overview".to_string()
}
fn default_true() -> bool {
    true
}
fn default_tmux_prefix() -> String {
    "spectatui-".to_string()
}
fn default_config_location() -> String {
    "./.spectatui.toml".to_string()
}

/// The selectable Config-location presets shown in Settings.
pub const CONFIG_LOCATIONS: &[&str] = &[
    "./.spectatui.toml",
    "~/.spectatui.toml",
    "~/.config/spectatui/config.toml",
];

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            theme: default_theme(),
            accent: default_accent(),
            dashboard_layout: default_layout(),
            mouse_support: false,
            agent_tail_follow: true,
            confirm_before_force: true,
            tmux_prefix: default_tmux_prefix(),
            config_location: default_config_location(),
            custom_layout: None,
        }
    }
}

impl AppConfig {
    pub fn theme_mode(&self) -> ThemeMode {
        match self.theme.as_str() {
            "light" => ThemeMode::Light,
            _ => ThemeMode::Dark,
        }
    }

    pub fn accent(&self) -> Accent {
        match self.accent.as_str() {
            "teal" => Accent::Teal,
            "amber" => Accent::Amber,
            _ => Accent::Indigo,
        }
    }

    pub fn set_theme(&mut self, mode: ThemeMode) {
        self.theme = match mode {
            ThemeMode::Dark => "dark",
            ThemeMode::Light => "light",
        }
        .to_string();
    }

    pub fn set_accent(&mut self, accent: Accent) {
        self.accent = match accent {
            Accent::Indigo => "indigo",
            Accent::Teal => "teal",
            Accent::Amber => "amber",
        }
        .to_string();
    }
}

/// Resolve one of the Config-location preset strings to an absolute path,
/// expanding a leading `~/` (home dir) or `./` (current dir).
pub fn resolve_config_location(loc: &str) -> Option<PathBuf> {
    if let Some(rest) = loc.strip_prefix("~/") {
        let home = directories::BaseDirs::new()?.home_dir().to_path_buf();
        Some(home.join(rest))
    } else if let Some(rest) = loc.strip_prefix("./") {
        Some(std::env::current_dir().ok()?.join(rest))
    } else {
        Some(PathBuf::from(loc))
    }
}

pub fn load_config(project_root: Option<&std::path::Path>) -> AppConfig {
    if let Some(root) = project_root {
        let project_config = root.join(".spectatui/config.toml");
        if let Ok(contents) = std::fs::read_to_string(&project_config) {
            if let Ok(config) = toml::from_str(&contents) {
                return config;
            }
        }
    }

    // Try each preset location in order; first that parses wins.
    for loc in CONFIG_LOCATIONS {
        if let Some(path) = resolve_config_location(loc) {
            if let Ok(contents) = std::fs::read_to_string(&path) {
                if let Ok(config) = toml::from_str(&contents) {
                    return config;
                }
            }
        }
    }

    AppConfig::default()
}

pub fn save_config(config: &AppConfig) -> Result<()> {
    let Some(path) = resolve_config_location(&config.config_location) else {
        return Ok(());
    };

    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }

    let content = toml::to_string_pretty(config)?;
    std::fs::write(&path, content)?;
    Ok(())
}
