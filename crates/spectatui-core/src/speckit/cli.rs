use std::path::PathBuf;

use tokio::process::Command;
use tokio::sync::mpsc;

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
        target: CliTarget,
    },
    CatalogAdd {
        target: CliTarget,
        url: String,
        name: String,
        priority: Option<u8>,
    },
    CatalogRemove {
        target: CliTarget,
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
                format!(
                    "specify {} set-priority {id} {priority}",
                    target.cmd_noun()
                )
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
                format!("specify {} catalog list", target.cmd_noun())
            }
            Self::CatalogAdd {
                target,
                url,
                name,
                priority,
            } => {
                let mut cmd = format!("specify {} catalog add {url} {name}", target.cmd_noun());
                if let Some(p) = priority {
                    cmd.push_str(&format!(" --priority {p}"));
                }
                cmd
            }
            Self::CatalogRemove { target, name } => {
                format!("specify {} catalog remove {name}", target.cmd_noun())
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

            let result = Command::new(parts[0])
                .args(&parts[1..])
                .current_dir(&root)
                .output()
                .await;

            match result {
                Ok(output) => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let stderr = String::from_utf8_lossy(&output.stderr);

                    for line in stdout.lines() {
                        let _ = tx.send(CliEvent::OutputLine(line.to_string()));
                    }
                    for line in stderr.lines() {
                        let _ = tx.send(CliEvent::OutputLine(line.to_string()));
                    }
                    let _ = tx.send(CliEvent::Completed {
                        success: output.status.success(),
                    });
                }
                Err(e) => {
                    let _ = tx.send(CliEvent::OutputLine(format!("error: {e}")));
                    let _ = tx.send(CliEvent::Completed { success: false });
                }
            }
        });

        (job, rx)
    }
}
