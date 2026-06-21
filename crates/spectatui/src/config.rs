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

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            theme: default_theme(),
            accent: default_accent(),
            dashboard_layout: default_layout(),
            mouse_support: false,
            agent_tail_follow: true,
            confirm_before_force: true,
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

fn config_dir() -> Option<PathBuf> {
    directories::ProjectDirs::from("", "", "spectatui").map(|d| d.config_dir().to_path_buf())
}

fn config_path() -> Option<PathBuf> {
    config_dir().map(|d| d.join("config.toml"))
}

pub fn config_path_display() -> String {
    config_path()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| "unknown".to_string())
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

    if let Some(path) = config_path() {
        if let Ok(contents) = std::fs::read_to_string(&path) {
            if let Ok(config) = toml::from_str(&contents) {
                return config;
            }
        }
    }

    AppConfig::default()
}

pub fn save_config(config: &AppConfig) -> Result<()> {
    let Some(path) = config_path() else {
        return Ok(());
    };

    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }

    let content = toml::to_string_pretty(config)?;
    std::fs::write(&path, content)?;
    Ok(())
}
