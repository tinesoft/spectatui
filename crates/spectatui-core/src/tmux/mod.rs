use std::process::Stdio;

use anyhow::{Context, Result};
use tokio::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionStatus {
    Running,
    Idle,
    Exited,
    NotFound,
}

#[derive(Debug, Clone)]
pub struct TmuxSession {
    pub name: String,
    pub pane_id: String,
    pub status: SessionStatus,
    pub last_snapshot: Vec<String>,
}

pub struct TmuxClient;

impl TmuxClient {
    pub async fn list_sessions() -> Result<Vec<String>> {
        let output = Command::new("tmux")
            .args(["list-sessions", "-F", "#{session_name}"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await;

        match output {
            Ok(out) if out.status.success() => {
                let text = String::from_utf8_lossy(&out.stdout);
                Ok(text.lines().map(|l| l.to_string()).collect())
            }
            Ok(_) => Ok(Vec::new()),
            Err(_) => Ok(Vec::new()),
        }
    }

    pub async fn find_session(feature_id: &str) -> Option<TmuxSession> {
        let sessions = Self::list_sessions().await.ok()?;
        let name = sessions
            .into_iter()
            .find(|s| s.contains(feature_id))?;

        let panes = Command::new("tmux")
            .args([
                "list-panes",
                "-t",
                &name,
                "-F",
                "#{pane_id}:#{pane_current_command}",
            ])
            .stdout(Stdio::piped())
            .output()
            .await
            .ok()?;

        let pane_text = String::from_utf8_lossy(&panes.stdout);
        let first_line = pane_text.lines().next().unwrap_or("");
        let pane_id = first_line
            .split(':')
            .next()
            .unwrap_or("")
            .to_string();

        let status = if first_line.contains("bash") || first_line.contains("zsh") {
            SessionStatus::Idle
        } else {
            SessionStatus::Running
        };

        Some(TmuxSession {
            name,
            pane_id,
            status,
            last_snapshot: Vec::new(),
        })
    }

    pub async fn capture_pane(session_name: &str, lines: u16) -> Result<Vec<String>> {
        let output = Command::new("tmux")
            .args([
                "capture-pane",
                "-t",
                session_name,
                "-p",
                "-S",
                &format!("-{lines}"),
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .context("failed to capture tmux pane")?;

        if output.status.success() {
            let text = String::from_utf8_lossy(&output.stdout);
            Ok(text.lines().map(|l| l.to_string()).collect())
        } else {
            Ok(Vec::new())
        }
    }

    pub async fn has_tmux() -> bool {
        Command::new("tmux")
            .arg("-V")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
}
