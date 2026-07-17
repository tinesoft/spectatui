use std::path::PathBuf;
use std::process::Stdio;

use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc;

use super::registry::CatalogTarget;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CliTarget {
    Extension,
    Preset,
}

impl CliTarget {
    pub fn cmd_noun(&self) -> &'static str {
        match self {
            Self::Extension => "extension",
            Self::Preset => "preset",
        }
    }
}

#[derive(Debug, Clone)]
pub enum CliAction {
    Search {
        target: CliTarget,
        query: Option<String>,
        tag: Option<String>,
        author: Option<String>,
    },
    Info {
        target: CliTarget,
        id: String,
    },
    List {
        target: CliTarget,
        available: bool,
    },
    Add {
        target: CliTarget,
        id: String,
        priority: Option<u8>,
        dev_path: Option<PathBuf>,
        from_url: Option<String>,
    },
    Remove {
        target: CliTarget,
        id: String,
        keep_config: bool,
        force: bool,
    },
    Enable {
        target: CliTarget,
        id: String,
    },
    Disable {
        target: CliTarget,
        id: String,
    },
    SetPriority {
        target: CliTarget,
        id: String,
        priority: u8,
    },
    Update {
        target: CliTarget,
        id: Option<String>,
    },
    Resolve {
        name: String,
    },
    CatalogList {
        target: CatalogTarget,
    },
    CatalogAdd {
        target: CatalogTarget,
        url: String,
        name: String,
        priority: Option<u8>,
        /// `Some(_)` explicitly passes `--install-allowed`/`--no-install-allowed`
        /// (used when re-adding a source to preserve its current flag across an
        /// edit); `None` omits the flag entirely, letting the CLI default to
        /// `--no-install-allowed` — the existing "add a new source" behavior.
        /// Only extension/preset catalogs accept this flag; integration/workflow
        /// catalog sources have no priority/install-allowed concept at all.
        install_allowed: Option<bool>,
    },
    CatalogRemove {
        target: CatalogTarget,
        name: String,
    },
    IntegrationList,
    IntegrationInstall {
        key: String,
    },
    IntegrationUninstall {
        key: String,
    },
    IntegrationUpgrade {
        key: Option<String>,
    },
    IntegrationUseDefault {
        key: String,
    },
    IntegrationSwitch {
        key: String,
    },
    IntegrationStatus {
        key: String,
    },
    IntegrationGetInfo {
        key: String,
    },
    WorkflowAdd {
        source: String,
    },
    WorkflowRemove {
        id: String,
    },
    WorkflowRun {
        source: String,
    },
    WorkflowResume {
        run_id: String,
    },
    WorkflowStatus {
        run_id: Option<String>,
    },
    WorkflowGetInfo {
        id: String,
    },
    WorkflowSearch {
        query: Option<String>,
    },
    SelfCheck,
    SelfUpgrade,
}

impl CliAction {
    pub fn is_destructive(&self) -> bool {
        matches!(
            self,
            Self::Add { .. }
                | Self::Remove { .. }
                | Self::Enable { .. }
                | Self::Disable { .. }
                | Self::SetPriority { .. }
                | Self::Update { .. }
                | Self::CatalogAdd { .. }
                | Self::CatalogRemove { .. }
                | Self::IntegrationInstall { .. }
                | Self::IntegrationUninstall { .. }
                | Self::IntegrationUpgrade { .. }
                | Self::IntegrationUseDefault { .. }
                | Self::IntegrationSwitch { .. }
                | Self::WorkflowAdd { .. }
                | Self::WorkflowRemove { .. }
                | Self::WorkflowRun { .. }
                | Self::SelfUpgrade
        )
    }

    pub fn to_command_line(&self) -> String {
        match self {
            Self::Search {
                target,
                query,
                tag,
                author,
            } => {
                let mut cmd = format!("specify {} search", target.cmd_noun());
                if let Some(q) = query {
                    cmd.push_str(&format!(" {q}"));
                }
                if let Some(t) = tag {
                    cmd.push_str(&format!(" --tag {t}"));
                }
                if let Some(a) = author {
                    cmd.push_str(&format!(" --author {a}"));
                }
                cmd
            }
            Self::Info { target, id } => {
                format!("specify {} info {id}", target.cmd_noun())
            }
            Self::List { target, available } => {
                let mut cmd = format!("specify {} list", target.cmd_noun());
                if *available {
                    cmd.push_str(" --available");
                }
                cmd
            }
            Self::Add {
                target,
                id,
                priority,
                dev_path,
                from_url,
            } => {
                let mut cmd = format!("specify {} add {id}", target.cmd_noun());
                if let Some(p) = priority {
                    cmd.push_str(&format!(" --priority {p}"));
                }
                if let Some(path) = dev_path {
                    cmd.push_str(&format!(" --dev {}", path.display()));
                }
                if let Some(url) = from_url {
                    cmd.push_str(&format!(" --from {url}"));
                }
                cmd
            }
            Self::Remove {
                target,
                id,
                keep_config,
                force,
            } => {
                let mut cmd = format!("specify {} remove {id}", target.cmd_noun());
                if *keep_config {
                    cmd.push_str(" --keep-config");
                }
                if *force {
                    cmd.push_str(" --force");
                }
                cmd
            }
            Self::Enable { target, id } => {
                format!("specify {} enable {id}", target.cmd_noun())
            }
            Self::Disable { target, id } => {
                format!("specify {} disable {id}", target.cmd_noun())
            }
            Self::SetPriority {
                target,
                id,
                priority,
            } => {
                format!("specify {} set-priority {id} {priority}", target.cmd_noun())
            }
            Self::Update { target, id } => {
                let mut cmd = format!("specify {} update", target.cmd_noun());
                if let Some(name) = id {
                    cmd.push_str(&format!(" {name}"));
                }
                cmd
            }
            Self::Resolve { name } => {
                format!("specify preset resolve {name}")
            }
            Self::CatalogList { target } => {
                format!("specify {} catalog list", target.cli())
            }
            Self::CatalogAdd {
                target,
                url,
                name,
                priority,
                install_allowed,
            } => {
                let mut cmd = format!("specify {} catalog add {url} {name}", target.cli());
                if let Some(p) = priority {
                    cmd.push_str(&format!(" --priority {p}"));
                }
                if let Some(allowed) = install_allowed {
                    cmd.push_str(if *allowed {
                        " --install-allowed"
                    } else {
                        " --no-install-allowed"
                    });
                }
                cmd
            }
            Self::CatalogRemove { target, name } => {
                format!("specify {} catalog remove {name}", target.cli())
            }
            Self::IntegrationList => "specify integration list".to_string(),
            Self::IntegrationInstall { key } => {
                format!("specify integration install {key}")
            }
            Self::IntegrationUninstall { key } => {
                format!("specify integration uninstall {key}")
            }
            Self::IntegrationUpgrade { key } => {
                let mut cmd = "specify integration upgrade".to_string();
                if let Some(k) = key {
                    cmd.push_str(&format!(" {k}"));
                }
                cmd
            }
            Self::IntegrationUseDefault { key } => {
                format!("specify integration use {key}")
            }
            Self::IntegrationSwitch { key } => {
                format!("specify integration switch {key}")
            }
            Self::IntegrationStatus { key } => {
                format!("specify integration status {key} --json")
            }
            Self::IntegrationGetInfo { key } => {
                format!("specify integration info {key}")
            }
            Self::WorkflowAdd { source } => {
                format!("specify workflow add {source}")
            }
            Self::WorkflowRemove { id } => {
                format!("specify workflow remove {id}")
            }
            Self::WorkflowRun { source } => {
                format!("specify workflow run {source}")
            }
            Self::WorkflowResume { run_id } => {
                format!("specify workflow resume {run_id}")
            }
            Self::WorkflowStatus { run_id } => {
                let mut cmd = "specify workflow status".to_string();
                if let Some(id) = run_id {
                    cmd.push_str(&format!(" {id}"));
                }
                cmd
            }
            Self::WorkflowGetInfo { id } => {
                format!("specify workflow info {id}")
            }
            Self::WorkflowSearch { query } => {
                let mut cmd = "specify workflow search".to_string();
                if let Some(q) = query {
                    cmd.push_str(&format!(" {q}"));
                }
                cmd
            }
            Self::SelfCheck => "specify self check".to_string(),
            Self::SelfUpgrade => "specify self upgrade".to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobStatus {
    Pending,
    Running,
    Succeeded,
    Failed,
}

#[derive(Debug, Clone)]
pub struct CliJob {
    pub action: CliAction,
    pub command_line: String,
    pub status: JobStatus,
    pub output: String,
}

impl CliJob {
    pub fn new(action: CliAction) -> Self {
        let command_line = action.to_command_line();
        Self {
            action,
            command_line,
            status: JobStatus::Pending,
            output: String::new(),
        }
    }
}

#[derive(Debug)]
pub enum CliEvent {
    OutputLine(String),
    Completed { success: bool },
}

pub struct SpecifyCliClient {
    project_root: PathBuf,
}

impl SpecifyCliClient {
    pub fn new(project_root: PathBuf) -> Self {
        Self { project_root }
    }

    pub fn spawn_job(&self, action: &CliAction) -> (CliJob, mpsc::UnboundedReceiver<CliEvent>) {
        let job = CliJob::new(action.clone());
        let (tx, rx) = mpsc::unbounded_channel();

        let cmd_line = job.command_line.clone();
        let root = self.project_root.clone();

        tokio::spawn(async move {
            let parts: Vec<&str> = cmd_line.split_whitespace().collect();
            if parts.is_empty() {
                let _ = tx.send(CliEvent::Completed { success: false });
                return;
            }

            let spawned = Command::new(parts[0])
                .args(&parts[1..])
                .current_dir(&root)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn();

            let mut child = match spawned {
                Ok(child) => child,
                Err(e) => {
                    let _ = tx.send(CliEvent::OutputLine(format!("error: {e}")));
                    let _ = tx.send(CliEvent::Completed { success: false });
                    return;
                }
            };

            // Stream stdout and stderr concurrently, emitting each line as it arrives.
            let stdout_task = child.stdout.take().map(|stdout| {
                let tx = tx.clone();
                let mut lines = BufReader::new(stdout).lines();
                tokio::spawn(async move {
                    while let Ok(Some(line)) = lines.next_line().await {
                        let _ = tx.send(CliEvent::OutputLine(line));
                    }
                })
            });
            let stderr_task = child.stderr.take().map(|stderr| {
                let tx = tx.clone();
                let mut lines = BufReader::new(stderr).lines();
                tokio::spawn(async move {
                    while let Ok(Some(line)) = lines.next_line().await {
                        let _ = tx.send(CliEvent::OutputLine(line));
                    }
                })
            });

            let success = child
                .wait()
                .await
                .map(|status| status.success())
                .unwrap_or(false);

            // Drain remaining buffered output before signaling completion.
            if let Some(task) = stdout_task {
                let _ = task.await;
            }
            if let Some(task) = stderr_task {
                let _ = task.await;
            }
            let _ = tx.send(CliEvent::Completed { success });
        });

        (job, rx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_list_command_line_covers_all_four_kinds() {
        assert_eq!(
            CliAction::CatalogList {
                target: CatalogTarget::Extension
            }
            .to_command_line(),
            "specify extension catalog list"
        );
        assert_eq!(
            CliAction::CatalogList {
                target: CatalogTarget::Preset
            }
            .to_command_line(),
            "specify preset catalog list"
        );
        assert_eq!(
            CliAction::CatalogList {
                target: CatalogTarget::Integration
            }
            .to_command_line(),
            "specify integration catalog list"
        );
        assert_eq!(
            CliAction::CatalogList {
                target: CatalogTarget::Workflow
            }
            .to_command_line(),
            "specify workflow catalog list"
        );
    }

    #[test]
    fn catalog_add_command_line_for_newly_supported_kinds() {
        assert_eq!(
            CliAction::CatalogAdd {
                target: CatalogTarget::Integration,
                url: "https://example.com/cat.json".to_string(),
                name: "community".to_string(),
                priority: None,
                install_allowed: None,
            }
            .to_command_line(),
            "specify integration catalog add https://example.com/cat.json community"
        );
        assert_eq!(
            CliAction::CatalogAdd {
                target: CatalogTarget::Workflow,
                url: "https://example.com/cat.json".to_string(),
                name: "community".to_string(),
                priority: Some(5),
                install_allowed: None,
            }
            .to_command_line(),
            "specify workflow catalog add https://example.com/cat.json community --priority 5"
        );
    }

    #[test]
    fn catalog_add_command_line_passes_install_allowed_flag_when_explicit() {
        assert_eq!(
            CliAction::CatalogAdd {
                target: CatalogTarget::Extension,
                url: "https://example.com/cat.json".to_string(),
                name: "community".to_string(),
                priority: Some(2),
                install_allowed: Some(true),
            }
            .to_command_line(),
            "specify extension catalog add https://example.com/cat.json community --priority 2 --install-allowed"
        );
        assert_eq!(
            CliAction::CatalogAdd {
                target: CatalogTarget::Preset,
                url: "https://example.com/cat.json".to_string(),
                name: "community".to_string(),
                priority: Some(2),
                install_allowed: Some(false),
            }
            .to_command_line(),
            "specify preset catalog add https://example.com/cat.json community --priority 2 --no-install-allowed"
        );
    }

    #[test]
    fn catalog_remove_command_line_for_newly_supported_kinds() {
        assert_eq!(
            CliAction::CatalogRemove {
                target: CatalogTarget::Integration,
                name: "community".to_string(),
            }
            .to_command_line(),
            "specify integration catalog remove community"
        );
        assert_eq!(
            CliAction::CatalogRemove {
                target: CatalogTarget::Workflow,
                name: "community".to_string(),
            }
            .to_command_line(),
            "specify workflow catalog remove community"
        );
    }
}
