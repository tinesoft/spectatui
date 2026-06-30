use std::collections::HashMap;
use std::path::Path;

use anyhow::{Context, Result};
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct ExtensionInfo {
    pub id: String,
    pub name: String,
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
    pub name: String,
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
                name: id.clone(),
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
                name: id.clone(),
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

// ---- Catalog discovery & fetching --------------------------------------------

/// Which catalog family to query via `specify <kind> catalog list`.
#[derive(Debug, Clone, Copy)]
enum CatalogKind {
    Extension,
    Preset,
    Integration,
    Workflow,
}

impl CatalogKind {
    fn cli(self) -> &'static str {
        match self {
            CatalogKind::Extension => "extension",
            CatalogKind::Preset => "preset",
            CatalogKind::Integration => "integration",
            CatalogKind::Workflow => "workflow",
        }
    }
}

#[derive(Debug, Clone)]
struct CatalogSource {
    name: String,
    url: String,
    /// `false` for "discovery only" (community) catalogs. Carried for a future
    /// UI affordance that gates the install action; not yet rendered.
    #[allow(dead_code)]
    install_allowed: bool,
}

/// Label shown in the UI for a catalog source: the built-in `default` catalog is
/// presented as `official`, matching the design.
fn catalog_label(name: &str) -> String {
    if name == "default" {
        "official".to_string()
    } else {
        name.to_string()
    }
}

/// Discover active catalog source URLs (default + community, in priority order) by
/// scraping `specify <kind> catalog list`. Run with `COLUMNS=4000` so the CLI emits
/// each URL on a single un-wrapped line.
async fn catalog_urls(root: &Path, kind: CatalogKind) -> Vec<CatalogSource> {
    let output = tokio::process::Command::new("specify")
        .args([kind.cli(), "catalog", "list"])
        .current_dir(root)
        .env("COLUMNS", "4000")
        .output()
        .await;

    match output {
        Ok(o) if o.status.success() => parse_catalog_urls(&String::from_utf8_lossy(&o.stdout)),
        _ => Vec::new(),
    }
}

fn parse_catalog_urls(output: &str) -> Vec<CatalogSource> {
    let allowed = |s: &str| !s.contains("discovery only");
    let mut out: Vec<CatalogSource> = Vec::new();
    let mut name: Option<String> = None;
    let mut ok = true;

    for raw in output.lines() {
        let line = raw.trim();
        if line.is_empty() {
            continue;
        }

        // Dialect B header: "- <name> — <policy>" or "[N] <name> — <policy>"
        let b = line.strip_prefix('-').map(str::trim).or_else(|| {
            line.starts_with('[')
                .then(|| line.split_once(']').map(|x| x.1).unwrap_or("").trim())
        });
        if let Some((n, tail)) = b.and_then(|r| r.split_once('—')) {
            name = Some(n.trim().to_string());
            ok = allowed(tail);
            continue;
        }

        // Dialect A header: "<name> (priority N)"
        if let Some((n, tail)) = line.split_once('(') {
            if tail.trim_start().starts_with("priority") {
                name = Some(n.trim().to_string());
                ok = true; // refined by the trailing "Install:" line
                continue;
            }
        }

        // Dialect A policy line (always trails its URL line)
        if let Some(p) = line.strip_prefix("Install:") {
            if let Some(last) = out.last_mut() {
                last.install_allowed = allowed(p);
            }
            continue;
        }

        // URL line: dialect A "URL: <url>" or dialect B bare "<url>"
        let url = line
            .strip_prefix("URL:")
            .map(str::trim)
            .or_else(|| line.starts_with("http").then_some(line));
        if let (Some(url), Some(n)) = (url.filter(|u| !u.is_empty()), name.clone()) {
            out.push(CatalogSource {
                name: n,
                url: url.to_string(),
                install_allowed: ok,
            });
        }
    }
    out
}

async fn fetch_catalog_json<T: serde::de::DeserializeOwned>(url: &str) -> Option<T> {
    let resp = reqwest::get(url).await.ok()?;
    if !resp.status().is_success() {
        return None;
    }
    resp.json::<T>().await.ok()
}

#[derive(Deserialize)]
struct ExtensionCatalog {
    #[serde(default)]
    extensions: HashMap<String, CatalogEntry>,
}

#[derive(Deserialize)]
struct PresetCatalog {
    #[serde(default)]
    presets: HashMap<String, CatalogEntry>,
}

#[derive(Deserialize)]
struct IntegrationCatalog {
    #[serde(default)]
    integrations: HashMap<String, CatalogEntry>,
}

#[derive(Deserialize)]
struct WorkflowCatalog {
    #[serde(default)]
    workflows: HashMap<String, CatalogEntry>,
}

/// Shared shape across all four catalog JSON files. Every field is optional and
/// unknown fields are ignored, so a sparse or evolving catalog still parses.
#[derive(Deserialize, Default)]
struct CatalogEntry {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    version: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    author: Option<String>,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    provides: Option<Provides>,
}

#[derive(Deserialize, Default)]
struct Provides {
    #[serde(default)]
    templates: u32,
}

pub async fn fetch_available_integrations(root: &Path) -> Vec<IntegrationInfo> {
    let mut items = Vec::new();
    for src in catalog_urls(root, CatalogKind::Integration).await {
        let Some(cat) = fetch_catalog_json::<IntegrationCatalog>(&src.url).await else {
            continue;
        };
        for (id, e) in cat.integrations {
            // IDE-based integrations don't require a CLI tool; everything else does.
            let cli_required = !e.tags.iter().any(|t| t == "ide");
            items.push(IntegrationInfo {
                name: e.name.unwrap_or_else(|| id.clone()),
                key: id,
                installed: false,
                is_default: false,
                cli_required,
                version: e.version,
                description: e.description.unwrap_or_default(),
            });
        }
    }
    items
}

pub async fn fetch_available_extensions(root: &Path) -> Vec<ExtensionInfo> {
    let mut items = Vec::new();
    for src in catalog_urls(root, CatalogKind::Extension).await {
        let Some(cat) = fetch_catalog_json::<ExtensionCatalog>(&src.url).await else {
            continue;
        };
        for (id, e) in cat.extensions {
            items.push(ExtensionInfo {
                name: e.name.unwrap_or_else(|| id.clone()),
                id,
                version: e.version.unwrap_or_default(),
                status: InstallStatus::Available,
                priority: None,
                command_count: 0,
                source: ExtensionSource::Catalog(catalog_label(&src.name)),
                author: e.author,
                description: e.description.unwrap_or_default(),
            });
        }
    }
    items
}

pub async fn fetch_available_presets(root: &Path) -> Vec<PresetInfo> {
    let mut items = Vec::new();
    for src in catalog_urls(root, CatalogKind::Preset).await {
        let Some(cat) = fetch_catalog_json::<PresetCatalog>(&src.url).await else {
            continue;
        };
        for (id, e) in cat.presets {
            items.push(PresetInfo {
                name: e.name.unwrap_or_else(|| id.clone()),
                id,
                version: e.version.unwrap_or_default(),
                status: InstallStatus::Available,
                priority: None,
                template_count: e.provides.map(|p| p.templates).unwrap_or(0),
                author: e.author,
                source_label: Some(format!("catalog · {}", catalog_label(&src.name))),
                description: e.description.unwrap_or_default(),
            });
        }
    }
    items
}

pub async fn fetch_workflows(root: &Path) -> Vec<WorkflowInfo> {
    // Installed workflows are authoritative for `installed`/`source`.
    let mut workflows = fetch_installed_workflows(root).await;

    // Catalog workflows: backfill metadata on installed entries, append the rest.
    for src in catalog_urls(root, CatalogKind::Workflow).await {
        let Some(cat) = fetch_catalog_json::<WorkflowCatalog>(&src.url).await else {
            continue;
        };
        for (id, e) in cat.workflows {
            if let Some(existing) = workflows.iter_mut().find(|w| w.id == id) {
                if existing.name.is_none() {
                    existing.name = e.name.clone();
                }
                if existing.description.is_empty() {
                    existing.description = e.description.clone().unwrap_or_default();
                }
                continue;
            }
            workflows.push(WorkflowInfo {
                id,
                name: e.name,
                version: e.version,
                source: Some(format!("catalog · {}", catalog_label(&src.name))),
                installed: false,
                description: e.description.unwrap_or_default(),
                last_run: None,
            });
        }
    }
    workflows
}

async fn fetch_installed_workflows(root: &Path) -> Vec<WorkflowInfo> {
    let output = tokio::process::Command::new("specify")
        .args(["workflow", "list"])
        .current_dir(root)
        .env("COLUMNS", "4000")
        .output()
        .await;

    match output {
        Ok(o) if o.status.success() => parse_installed_workflows(&String::from_utf8_lossy(&o.stdout)),
        _ => Vec::new(),
    }
}

/// Parse `specify workflow list`: blocks of `  <name> (<id>) v<ver>` followed by an
/// indented description line.
fn parse_installed_workflows(output: &str) -> Vec<WorkflowInfo> {
    let mut out: Vec<WorkflowInfo> = Vec::new();
    for raw in output.lines() {
        let line = raw.trim();
        if line.is_empty() {
            continue;
        }

        // Header "<name> (<id>) v<ver>" — id is the last parenthesised token before " v".
        if let Some((before_v, ver)) = line.rsplit_once(" v") {
            if before_v.ends_with(')')
                && ver.chars().next().is_some_and(|c| c.is_ascii_digit())
            {
                if let Some(open) = before_v.rfind('(') {
                    let id = before_v[open + 1..before_v.len() - 1].trim().to_string();
                    let name = before_v[..open].trim().to_string();
                    out.push(WorkflowInfo {
                        id,
                        name: Some(name),
                        version: Some(ver.to_string()),
                        source: Some("bundled".to_string()),
                        installed: true,
                        description: String::new(),
                        last_run: None,
                    });
                    continue;
                }
            }
        }

        // Indented description for the most recent workflow.
        if let Some(last) = out.last_mut() {
            if last.description.is_empty() {
                last.description = line.to_string();
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_dialect_a_catalog_urls() {
        // extension / preset `catalog list` format (run with COLUMNS=4000).
        let out = "\
Active Extension Catalogs:

  default (priority 1)
     Built-in catalog of installable extensions
     URL: https://example.com/extensions/catalog.json
     Install: install allowed

  community (priority 2)
     Community-contributed extensions (discovery only)
     URL: https://example.com/extensions/catalog.community.json
     Install: discovery only

Using built-in default catalog stack.
";
        let srcs = parse_catalog_urls(out);
        assert_eq!(srcs.len(), 2);
        assert_eq!(srcs[0].name, "default");
        assert_eq!(srcs[0].url, "https://example.com/extensions/catalog.json");
        assert!(srcs[0].install_allowed);
        assert_eq!(srcs[1].name, "community");
        assert!(!srcs[1].install_allowed);
    }

    #[test]
    fn parses_dialect_b_catalog_urls() {
        // integration / workflow `catalog list` format.
        let out = "\
Workflow Catalog Sources:

  [0] default — install allowed
      https://example.com/workflows/catalog.json
      Official workflows

  [1] community — discovery only
      https://example.com/workflows/catalog.community.json
      Community-contributed workflows (discovery only)
";
        let srcs = parse_catalog_urls(out);
        assert_eq!(srcs.len(), 2);
        assert_eq!(srcs[0].name, "default");
        assert_eq!(srcs[0].url, "https://example.com/workflows/catalog.json");
        assert!(srcs[0].install_allowed);
        assert_eq!(srcs[1].name, "community");
        assert!(!srcs[1].install_allowed);
    }

    #[test]
    fn parses_extension_catalog_json() {
        let json = r#"{
            "schema_version": "1.0",
            "extensions": {
                "agent-context": {
                    "name": "Coding Agent Context",
                    "id": "agent-context",
                    "version": "1.0.0",
                    "description": "Manages agent context files",
                    "author": "spec-kit-core",
                    "tags": ["agent", "context"]
                }
            }
        }"#;
        let cat: ExtensionCatalog = serde_json::from_str(json).unwrap();
        let e = &cat.extensions["agent-context"];
        assert_eq!(e.name.as_deref(), Some("Coding Agent Context"));
        assert_eq!(e.author.as_deref(), Some("spec-kit-core"));
    }

    #[test]
    fn parses_preset_catalog_json_templates() {
        let json = r#"{
            "presets": {
                "lean": {
                    "name": "Lean Workflow",
                    "version": "1.0.0",
                    "author": "github",
                    "provides": { "commands": 5, "templates": 3 },
                    "tags": ["lean"]
                }
            }
        }"#;
        let cat: PresetCatalog = serde_json::from_str(json).unwrap();
        let e = &cat.presets["lean"];
        assert_eq!(e.provides.as_ref().map(|p| p.templates), Some(3));
    }

    #[test]
    fn integration_cli_required_from_tags() {
        let derive = |tags: &[&str]| !tags.iter().any(|t| *t == "ide");
        assert!(derive(&["cli", "anthropic"]));
        assert!(!derive(&["ide", "github"]));
    }

    #[test]
    fn parses_installed_workflows() {
        let out = "\
Installed Workflows:

  Full SDD Cycle (speckit) v1.0.0
    Runs specify then plan then tasks then implement with review gates
";
        let wfs = parse_installed_workflows(out);
        assert_eq!(wfs.len(), 1);
        assert_eq!(wfs[0].id, "speckit");
        assert_eq!(wfs[0].name.as_deref(), Some("Full SDD Cycle"));
        assert_eq!(wfs[0].version.as_deref(), Some("1.0.0"));
        assert!(wfs[0].installed);
        assert!(!wfs[0].description.is_empty());
    }
}
