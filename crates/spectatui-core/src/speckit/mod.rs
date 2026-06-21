pub mod cli;
pub mod registry;
pub mod watch;
mod workflow;

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
pub use registry::{
    ExtensionInfo, ExtensionSource, InstallStatus, IntegrationInfo, PresetInfo, WorkflowInfo,
};
pub use workflow::{TasksProgress, WorkflowStage};

#[derive(Debug, Clone)]
pub struct Project {
    pub root: PathBuf,
    pub constitution: Option<PathBuf>,
    pub features: Vec<Feature>,
    pub extensions: Vec<ExtensionInfo>,
    pub presets: Vec<PresetInfo>,
    pub integrations: Vec<IntegrationInfo>,
    pub workflows: Vec<WorkflowInfo>,
}

#[derive(Debug, Clone)]
pub struct Feature {
    pub id: String,
    pub branch: Option<String>,
    pub dir: PathBuf,
    pub artifacts: FeatureArtifacts,
    pub stage: WorkflowStage,
}

#[derive(Debug, Clone, Default)]
pub struct FeatureArtifacts {
    pub spec: Option<PathBuf>,
    pub plan: Option<PathBuf>,
    pub tasks: Option<PathBuf>,
    pub research: Option<PathBuf>,
    pub data_model: Option<PathBuf>,
    pub quickstart: Option<PathBuf>,
    pub contracts_dir: Option<PathBuf>,
}

impl Project {
    pub fn discover(root: &Path) -> Result<Self> {
        let root = root.canonicalize().context("project root not found")?;
        let constitution = {
            let p = root.join(".specify/memory/constitution.md");
            p.is_file().then_some(p)
        };

        let features = discover_features(&root)?;
        let extensions = registry::load_extensions(&root)?;
        let presets = registry::load_presets(&root)?;
        let integrations = registry::load_integrations(&root)?;

        Ok(Project {
            root,
            constitution,
            features,
            extensions,
            presets,
            integrations,
            workflows: Vec::new(),
        })
    }
}

fn discover_features(root: &Path) -> Result<Vec<Feature>> {
    let specs_dir = root.join("specs");
    if !specs_dir.is_dir() {
        return Ok(Vec::new());
    }

    let mut entries: Vec<_> = std::fs::read_dir(&specs_dir)
        .context("failed to read specs/")?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .collect();

    entries.sort_by_key(|e| e.file_name());

    let mut features = Vec::new();
    for entry in entries {
        let dir = entry.path();
        let id = dir
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        if id.starts_with('.') {
            continue;
        }

        let artifacts = discover_artifacts(&dir);
        let stage = workflow::infer_stage(&artifacts);

        let branch = Some(id.clone());

        features.push(Feature {
            id,
            branch,
            dir,
            artifacts,
            stage,
        });
    }

    Ok(features)
}

fn discover_artifacts(dir: &Path) -> FeatureArtifacts {
    let file_if_exists = |name: &str| {
        let p = dir.join(name);
        p.is_file().then_some(p)
    };
    let dir_if_exists = |name: &str| {
        let p = dir.join(name);
        p.is_dir().then_some(p)
    };

    FeatureArtifacts {
        spec: file_if_exists("spec.md"),
        plan: file_if_exists("plan.md"),
        tasks: file_if_exists("tasks.md"),
        research: file_if_exists("research.md"),
        data_model: file_if_exists("data-model.md"),
        quickstart: file_if_exists("quickstart.md"),
        contracts_dir: dir_if_exists("contracts"),
    }
}
