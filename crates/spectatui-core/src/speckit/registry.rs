use std::collections::HashMap;
use std::path::Path;

use anyhow::{Context, Result};
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct ExtensionInfo {
    pub id: String,
    pub version: String,
    pub status: InstallStatus,
    pub priority: Option<u8>,
    pub command_count: u32,
    pub source: ExtensionSource,
    pub author: Option<String>,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct PresetInfo {
    pub id: String,
    pub version: String,
    pub status: InstallStatus,
    pub priority: Option<u8>,
    pub template_count: u32,
    pub author: Option<String>,
    pub source_label: Option<String>,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct IntegrationInfo {
    pub key: String,
    pub name: String,
    pub installed: bool,
    pub is_default: bool,
    pub cli_required: bool,
    pub version: Option<String>,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct WorkflowInfo {
    pub id: String,
    pub name: Option<String>,
    pub version: Option<String>,
    pub source: Option<String>,
    pub installed: bool,
    pub description: String,
    pub last_run: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallStatus {
    Enabled,
    Disabled,
    Available,
}

#[derive(Debug, Clone)]
pub enum ExtensionSource {
    Catalog(String),
    Dev(std::path::PathBuf),
    Url(String),
    Local,
}

#[derive(Deserialize)]
struct ExtensionRegistry {
    #[allow(dead_code)]
    schema_version: Option<String>,
    extensions: HashMap<String, ExtensionRegistryEntry>,
}

#[derive(Deserialize)]
struct ExtensionRegistryEntry {
    version: String,
    source: Option<String>,
    enabled: bool,
    priority: Option<u8>,
    registered_commands: Option<HashMap<String, Vec<String>>>,
    #[allow(dead_code)]
    registered_skills: Option<Vec<String>>,
}

#[derive(Deserialize)]
struct IntegrationState {
    #[allow(dead_code)]
    version: Option<String>,
    default_integration: Option<String>,
    installed_integrations: Vec<String>,
    integration_settings: Option<HashMap<String, serde_json::Value>>,
}

pub fn load_extensions(root: &Path) -> Result<Vec<ExtensionInfo>> {
    let registry_path = root.join(".specify/extensions/.registry");
    if !registry_path.is_file() {
        return Ok(Vec::new());
    }

    let content =
        std::fs::read_to_string(&registry_path).context("failed to read extensions registry")?;

    let registry: ExtensionRegistry =
        serde_json::from_str(&content).context("failed to parse extensions registry")?;

    let mut extensions: Vec<ExtensionInfo> = registry
        .extensions
        .into_iter()
        .map(|(id, entry)| {
            let command_count = entry
                .registered_commands
                .as_ref()
                .map(|cmds| cmds.values().map(|v| v.len() as u32).sum())
                .unwrap_or(0);

            let source = match entry.source.as_deref() {
                Some("local") => ExtensionSource::Local,
                Some(s) if s.starts_with("http") => ExtensionSource::Url(s.to_string()),
                Some(s) => ExtensionSource::Catalog(s.to_string()),
                None => ExtensionSource::Local,
            };

            let status = if entry.enabled {
                InstallStatus::Enabled
            } else {
                InstallStatus::Disabled
            };

            ExtensionInfo {
                id,
                version: entry.version,
                status,
                priority: entry.priority,
                command_count,
                source,
                author: None,
                description: String::new(),
            }
        })
        .collect();

    extensions.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(extensions)
}

pub fn load_presets(root: &Path) -> Result<Vec<PresetInfo>> {
    let registry_path = root.join(".specify/presets/.registry");
    if !registry_path.is_file() {
        return Ok(Vec::new());
    }

    let content =
        std::fs::read_to_string(&registry_path).context("failed to read presets registry")?;

    let registry: PresetRegistry =
        serde_json::from_str(&content).context("failed to parse presets registry")?;

    let mut presets: Vec<PresetInfo> = registry
        .presets
        .into_iter()
        .map(|(id, entry)| {
            let status = if entry.enabled {
                InstallStatus::Enabled
            } else {
                InstallStatus::Disabled
            };

            PresetInfo {
                id,
                version: entry.version,
                status,
                priority: entry.priority,
                template_count: entry.template_count.unwrap_or(0),
                author: None,
                source_label: None,
                description: String::new(),
            }
        })
        .collect();

    presets.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(presets)
}

#[derive(Deserialize)]
struct PresetRegistry {
    presets: HashMap<String, PresetRegistryEntry>,
}

#[derive(Deserialize)]
struct PresetRegistryEntry {
    version: String,
    enabled: bool,
    priority: Option<u8>,
    template_count: Option<u32>,
}

pub fn load_integrations(root: &Path) -> Result<Vec<IntegrationInfo>> {
    let path = root.join(".specify/integration.json");
    if !path.is_file() {
        return Ok(Vec::new());
    }

    let content = std::fs::read_to_string(&path).context("failed to read integration.json")?;
    let state: IntegrationState =
        serde_json::from_str(&content).context("failed to parse integration.json")?;

    let default_name = state.default_integration.as_deref().unwrap_or("");

    let integrations = state
        .installed_integrations
        .into_iter()
        .map(|name| {
            let settings = state
                .integration_settings
                .as_ref()
                .and_then(|s| s.get(&name));

            let version = settings
                .and_then(|v| v.get("version"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            let description = settings
                .and_then(|v| v.get("description"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let cli_required = settings
                .and_then(|v| v.get("cli_required"))
                .and_then(|v| v.as_bool())
                .unwrap_or(true);

            let is_default = name == default_name;

            IntegrationInfo {
                key: name.clone(),
                name,
                installed: true,
                is_default,
                cli_required,
                version,
                description,
            }
        })
        .collect();

    Ok(integrations)
}

pub async fn fetch_available_integrations(root: &Path) -> Vec<IntegrationInfo> {
    let output = tokio::process::Command::new("specify")
        .args(["integration", "list", "--catalog"])
        .current_dir(root)
        .output()
        .await;

    let output = match output {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).to_string(),
        _ => return Vec::new(),
    };

    parse_catalog_lines(&output, |name, desc| IntegrationInfo {
        key: name.clone(),
        name,
        installed: false,
        is_default: false,
        cli_required: true,
        version: None,
        description: desc,
    })
}

pub async fn fetch_available_extensions(root: &Path) -> Vec<ExtensionInfo> {
    let output = tokio::process::Command::new("specify")
        .args(["extension", "list", "--available"])
        .current_dir(root)
        .output()
        .await;

    let output = match output {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).to_string(),
        _ => return Vec::new(),
    };

    parse_catalog_lines(&output, |id, desc| ExtensionInfo {
        id,
        version: String::new(),
        status: InstallStatus::Available,
        priority: None,
        command_count: 0,
        source: ExtensionSource::Catalog(String::new()),
        author: None,
        description: desc,
    })
}

pub async fn fetch_available_presets(root: &Path) -> Vec<PresetInfo> {
    let output = tokio::process::Command::new("specify")
        .args(["preset", "search"])
        .current_dir(root)
        .output()
        .await;

    let output = match output {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).to_string(),
        _ => return Vec::new(),
    };

    parse_catalog_lines(&output, |id, desc| PresetInfo {
        id,
        version: String::new(),
        status: InstallStatus::Available,
        priority: None,
        template_count: 0,
        author: None,
        source_label: None,
        description: desc,
    })
}

pub async fn fetch_workflows(root: &Path) -> Vec<WorkflowInfo> {
    let output = tokio::process::Command::new("specify")
        .args(["workflow", "list"])
        .current_dir(root)
        .output()
        .await;

    let output = match output {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).to_string(),
        _ => return Vec::new(),
    };

    parse_catalog_lines(&output, |id, desc| WorkflowInfo {
        id,
        name: None,
        version: None,
        source: None,
        installed: true,
        description: desc,
        last_run: None,
    })
}

fn parse_catalog_lines<T>(output: &str, make: impl Fn(String, String) -> T) -> Vec<T> {
    let mut items = Vec::new();
    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("─") || trimmed.starts_with('=') {
            continue;
        }
        let parts: Vec<&str> = trimmed.splitn(2, ['\t', ' ']).collect();
        let id = parts[0].trim().to_string();
        let desc = parts.get(1).map(|s| s.trim().to_string()).unwrap_or_default();
        if !id.is_empty() {
            items.push(make(id, desc));
        }
    }
    items
}
