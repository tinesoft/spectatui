use std::fs;

use super::FeatureArtifacts;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum WorkflowStage {
    NotStarted,
    Specified,
    Clarified,
    Planned,
    TasksGenerated,
    Analyzed,
    Implementing,
    Implemented,
    /// Artifacts exist but don't match any recognized Spec-Kit template convention
    /// (e.g. produced by an incompatible template version) — a terminal degraded
    /// display state, not a step in the lifecycle sequence.
    Unknown,
}

impl WorkflowStage {
    pub fn label(&self) -> &'static str {
        match self {
            Self::NotStarted => "new",
            Self::Specified => "spec",
            Self::Clarified => "clar",
            Self::Planned => "plan",
            Self::TasksGenerated => "task",
            Self::Analyzed => "anly",
            Self::Implementing => "impl",
            Self::Implemented => "impl",
            Self::Unknown => "unk",
        }
    }
}

/// A `tasks.md`/`spec.md` is considered recognizable if it's either empty (freshly
/// generated, not yet populated) or contains at least one Markdown heading or
/// checklist line — the minimal shape every Spec-Kit template produces. Content
/// matching neither means the file wasn't produced by a template this version knows
/// how to read.
fn has_recognized_markdown_structure(path: &std::path::Path) -> bool {
    let Ok(content) = fs::read_to_string(path) else {
        return true; // unreadable is a separate concern from unrecognized content
    };
    if content.trim().is_empty() {
        return true;
    }
    content.lines().any(|line| {
        let trimmed = line.trim_start();
        trimmed.starts_with('#')
            || trimmed.starts_with("- [ ]")
            || trimmed.starts_with("- [x]")
            || trimmed.starts_with("- [X]")
    })
}

pub fn infer_stage(artifacts: &FeatureArtifacts) -> WorkflowStage {
    if let Some(tasks_path) = &artifacts.tasks {
        if !has_recognized_markdown_structure(tasks_path) {
            return WorkflowStage::Unknown;
        }
    } else if let Some(spec_path) = &artifacts.spec {
        if artifacts.plan.is_none() && !has_recognized_markdown_structure(spec_path) {
            return WorkflowStage::Unknown;
        }
    }

    let tasks_status = artifacts
        .tasks
        .as_ref()
        .and_then(|p| parse_tasks_progress(p));

    if let Some((done, total)) = tasks_status {
        if total > 0 && done >= total {
            return WorkflowStage::Implemented;
        }
        if done > 0 {
            return WorkflowStage::Implementing;
        }
    }

    if artifacts.tasks.is_some() {
        return WorkflowStage::TasksGenerated;
    }

    if artifacts.plan.is_some() {
        return WorkflowStage::Planned;
    }

    if let Some(spec_path) = &artifacts.spec {
        if spec_has_clarifications(spec_path) {
            return WorkflowStage::Clarified;
        }
        return WorkflowStage::Specified;
    }

    WorkflowStage::NotStarted
}

fn spec_has_clarifications(path: &std::path::Path) -> bool {
    let Ok(content) = fs::read_to_string(path) else {
        return false;
    };
    content
        .lines()
        .any(|line| line.starts_with("## Clarification"))
}

fn parse_tasks_progress(path: &std::path::Path) -> Option<(usize, usize)> {
    let content = fs::read_to_string(path).ok()?;
    let mut done = 0usize;
    let mut total = 0usize;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("- [x]") || trimmed.starts_with("- [X]") {
            done += 1;
            total += 1;
        } else if trimmed.starts_with("- [ ]") {
            total += 1;
        }
    }

    if total == 0 {
        None
    } else {
        Some((done, total))
    }
}

pub struct TasksProgress {
    pub done: usize,
    pub total: usize,
}

impl TasksProgress {
    pub fn from_file(path: &std::path::Path) -> Option<Self> {
        let (done, total) = parse_tasks_progress(path)?;
        Some(TasksProgress { done, total })
    }

    pub fn percent(&self) -> u8 {
        if self.total == 0 {
            0
        } else {
            ((self.done as f64 / self.total as f64) * 100.0) as u8
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    fn write_file(dir: &std::path::Path, name: &str, content: &str) -> std::path::PathBuf {
        let path = dir.join(name);
        let mut f = std::fs::File::create(&path).unwrap();
        f.write_all(content.as_bytes()).unwrap();
        path
    }

    #[test]
    fn not_started_when_no_artifacts() {
        let artifacts = FeatureArtifacts::default();
        assert_eq!(infer_stage(&artifacts), WorkflowStage::NotStarted);
    }

    #[test]
    fn specified_when_spec_only() {
        let tmp = TempDir::new().unwrap();
        let spec = write_file(tmp.path(), "spec.md", "# Feature\n## Requirements\n");
        let artifacts = FeatureArtifacts {
            spec: Some(spec),
            ..Default::default()
        };
        assert_eq!(infer_stage(&artifacts), WorkflowStage::Specified);
    }

    #[test]
    fn clarified_when_spec_has_clarifications() {
        let tmp = TempDir::new().unwrap();
        let spec = write_file(
            tmp.path(),
            "spec.md",
            "# Feature\n## Clarifications\n\n1. Q: Foo? A: Bar.\n",
        );
        let artifacts = FeatureArtifacts {
            spec: Some(spec),
            ..Default::default()
        };
        assert_eq!(infer_stage(&artifacts), WorkflowStage::Clarified);
    }

    #[test]
    fn planned_when_plan_exists() {
        let tmp = TempDir::new().unwrap();
        let spec = write_file(tmp.path(), "spec.md", "# Feature\n");
        let plan = write_file(tmp.path(), "plan.md", "# Plan\n");
        let artifacts = FeatureArtifacts {
            spec: Some(spec),
            plan: Some(plan),
            ..Default::default()
        };
        assert_eq!(infer_stage(&artifacts), WorkflowStage::Planned);
    }

    #[test]
    fn tasks_generated_when_no_checkboxes_done() {
        let tmp = TempDir::new().unwrap();
        let spec = write_file(tmp.path(), "spec.md", "# Feature\n");
        let plan = write_file(tmp.path(), "plan.md", "# Plan\n");
        let tasks = write_file(
            tmp.path(),
            "tasks.md",
            "# Tasks\n- [ ] T001 Do thing\n- [ ] T002 Do other\n",
        );
        let artifacts = FeatureArtifacts {
            spec: Some(spec),
            plan: Some(plan),
            tasks: Some(tasks),
            ..Default::default()
        };
        assert_eq!(infer_stage(&artifacts), WorkflowStage::TasksGenerated);
    }

    #[test]
    fn implementing_when_partial_tasks() {
        let tmp = TempDir::new().unwrap();
        let tasks = write_file(
            tmp.path(),
            "tasks.md",
            "# Tasks\n- [x] T001 Done\n- [ ] T002 Pending\n",
        );
        let artifacts = FeatureArtifacts {
            spec: Some(tmp.path().join("spec.md")),
            tasks: Some(tasks),
            ..Default::default()
        };
        assert_eq!(infer_stage(&artifacts), WorkflowStage::Implementing);
    }

    #[test]
    fn implemented_when_all_tasks_done() {
        let tmp = TempDir::new().unwrap();
        let tasks = write_file(
            tmp.path(),
            "tasks.md",
            "# Tasks\n- [x] T001 Done\n- [x] T002 Also done\n",
        );
        let artifacts = FeatureArtifacts {
            spec: Some(tmp.path().join("spec.md")),
            tasks: Some(tasks),
            ..Default::default()
        };
        assert_eq!(infer_stage(&artifacts), WorkflowStage::Implemented);
    }

    #[test]
    fn unknown_when_tasks_file_unrecognized() {
        let tmp = TempDir::new().unwrap();
        let spec = write_file(tmp.path(), "spec.md", "# Feature\n");
        let tasks = write_file(
            tmp.path(),
            "tasks.md",
            "this is not a task list at all, just prose with no heading or checkboxes",
        );
        let artifacts = FeatureArtifacts {
            spec: Some(spec),
            tasks: Some(tasks),
            ..Default::default()
        };
        assert_eq!(infer_stage(&artifacts), WorkflowStage::Unknown);
    }

    #[test]
    fn unknown_when_spec_file_unrecognized() {
        let tmp = TempDir::new().unwrap();
        let spec = write_file(
            tmp.path(),
            "spec.md",
            "this is not a spec document, no headings or checklists here",
        );
        let artifacts = FeatureArtifacts {
            spec: Some(spec),
            ..Default::default()
        };
        assert_eq!(infer_stage(&artifacts), WorkflowStage::Unknown);
    }

    #[test]
    fn tasks_generated_when_empty_tasks_file() {
        let tmp = TempDir::new().unwrap();
        let spec = write_file(tmp.path(), "spec.md", "# Feature\n");
        let tasks = write_file(tmp.path(), "tasks.md", "");
        let artifacts = FeatureArtifacts {
            spec: Some(spec),
            tasks: Some(tasks),
            ..Default::default()
        };
        assert_eq!(infer_stage(&artifacts), WorkflowStage::TasksGenerated);
    }

    #[test]
    fn tasks_progress_percent() {
        let tmp = TempDir::new().unwrap();
        let path = write_file(
            tmp.path(),
            "tasks.md",
            "- [x] A\n- [x] B\n- [ ] C\n- [ ] D\n",
        );
        let progress = TasksProgress::from_file(&path).unwrap();
        assert_eq!(progress.done, 2);
        assert_eq!(progress.total, 4);
        assert_eq!(progress.percent(), 50);
    }
}
