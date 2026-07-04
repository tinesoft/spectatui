use std::path::Path;
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
        let name = sessions.into_iter().find(|s| s.contains(feature_id))?;

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
        let pane_id = first_line.split(':').next().unwrap_or("").to_string();

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

    /// Send a line of text to the target pane/session, followed by Enter.
    ///
    /// The text is sent with `-l` (literal) so tmux does not interpret words
    /// like `Enter` inside it as key names; Enter is then sent separately.
    pub async fn send_keys(target: &str, text: &str) -> Result<()> {
        Command::new("tmux")
            .args(["send-keys", "-t", target, "-l", text])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .context("failed to send keys to tmux pane")?;

        Command::new("tmux")
            .args(["send-keys", "-t", target, "Enter"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .context("failed to send Enter to tmux pane")?;

        Ok(())
    }

    /// Create a new detached tmux session running `command` in `cwd`.
    ///
    /// The session is left detached (`-d`); the caller attaches separately.
    pub async fn launch_session(session_name: &str, cwd: &Path, command: &str) -> Result<()> {
        Command::new("tmux")
            .args(["new-session", "-d", "-s", session_name, "-c"])
            .arg(cwd)
            .arg(command)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .context("failed to create tmux session")?;
        Ok(())
    }

    /// Attach to a tmux session as a foreground process with inherited stdio.
    ///
    /// The caller must leave the alternate screen / raw mode before calling
    /// this and restore it after the future resolves (on detach).
    pub async fn attach(session_name: &str) -> Result<()> {
        Command::new("tmux")
            .args(["attach", "-t", session_name])
            .status()
            .await
            .context("failed to attach to tmux session")?;
        Ok(())
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
