#![allow(dead_code)]

use std::cell::RefCell;
use std::collections::HashSet;

use ratatui::layout::Rect;
use spectatui_core::layout::CustomLayout;
use spectatui_core::speckit::cli::{CliAction, CliJob, JobStatus, CliEvent};
use spectatui_core::speckit::{ExtensionInfo, IntegrationInfo, PresetInfo, Project, TasksProgress, WorkflowInfo};
use spectatui_core::tmux::TmuxSession;
use tokio::sync::mpsc;

use crate::config::{self, AppConfig};
use crate::theme::{Accent, Theme, ThemeMode};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Dashboard,
    SpecBrowser,
    Constitution,
    ExtensionsPresets,
    Settings,
    SessionAttach,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DashboardLayout {
    Overview,
    Coding,
    Audit,
    Custom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pane {
    FeatureList,
    Workflow,
    AgentOutput,
    SpecBrowser,
    Constitution,
    ExtensionsPresets,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpecTab {
    Spec,
    Plan,
    Tasks,
    Research,
}

impl SpecTab {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Spec => "spec.md",
            Self::Plan => "plan.md",
            Self::Tasks => "tasks.md",
            Self::Research => "research.md",
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::Spec => Self::Plan,
            Self::Plan => Self::Tasks,
            Self::Tasks => Self::Research,
            Self::Research => Self::Spec,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Self::Spec => Self::Research,
            Self::Plan => Self::Spec,
            Self::Tasks => Self::Plan,
            Self::Research => Self::Tasks,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtTab {
    Extensions,
    Presets,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PopupKind {
    Integrations,
    Extensions,
    Presets,
    Features,
    Workflows,
    Help,
    QuitConfirm,
    CommandPalette,
    CliConfirm,
    CliOutput,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsRow {
    Theme,
    Accent,
    DashboardLayout,
    AgentTailFollow,
    MouseSupport,
    ConfirmForce,
    TmuxPrefix,
    CustomizePanes,
    AttachSession,
    ConfigPath,
}

impl SettingsRow {
    pub const ALL: &[Self] = &[
        Self::Theme,
        Self::Accent,
        Self::DashboardLayout,
        Self::AgentTailFollow,
        Self::MouseSupport,
        Self::ConfirmForce,
        Self::TmuxPrefix,
        Self::CustomizePanes,
        Self::AttachSession,
        Self::ConfigPath,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            Self::Theme => "Theme",
            Self::Accent => "Accent palette",
            Self::DashboardLayout => "Dashboard layout",
            Self::AgentTailFollow => "Agent tail follow",
            Self::MouseSupport => "Mouse support",
            Self::ConfirmForce => "Confirm before --force",
            Self::TmuxPrefix => "tmux session prefix",
            Self::CustomizePanes => "Customize panes",
            Self::AttachSession => "Attach agent session",
            Self::ConfigPath => "Config location",
        }
    }

    /// The selectable chip options for this row, if it is an option row.
    pub fn options(&self) -> &'static [&'static str] {
        match self {
            Self::Theme => &["dark", "light"],
            Self::Accent => &["indigo", "teal", "amber"],
            Self::DashboardLayout => &["overview", "coding", "audit"],
            Self::AgentTailFollow => &["on", "off"],
            Self::MouseSupport => &["on", "off"],
            Self::ConfirmForce => &["always", "never"],
            _ => &[],
        }
    }
}

pub struct PaletteState {
    pub input: String,
    pub selected: usize,
}

pub struct App {
    pub project: Project,
    pub screen: Screen,
    pub layout: DashboardLayout,
    pub focused_pane: Pane,
    pub feature_index: usize,
    pub spec_tab: SpecTab,
    pub spec_scroll: u16,
    pub theme_mode: ThemeMode,
    pub accent: Accent,
    pub theme: Theme,
    pub should_quit: bool,
    pub project_name: String,
    pub project_path: String,
    pub config: AppConfig,

    // Extensions/presets
    pub ext_tab: ExtTab,
    pub ext_index: usize,
    pub preset_index: usize,

    // Popups
    pub active_popup: Option<PopupKind>,

    // CLI jobs
    pub cli_job: Option<CliJob>,
    pub cli_rx: Option<mpsc::UnboundedReceiver<CliEvent>>,
    pub pending_action: Option<CliAction>,
    pub force_flag: bool,

    // Tmux
    pub tmux_available: bool,
    pub tmux_session: Option<TmuxSession>,

    // Settings
    pub settings_index: usize,

    // Command palette
    pub palette: Option<PaletteState>,

    // Layout editor
    pub custom_layout: CustomLayout,
    pub layout_editor_index: usize,
    pub layout_editor_active: bool,

    // Agent output
    pub agent_lines: Vec<String>,

    // Integration management
    pub integration_index: usize,

    // Workflow management
    pub wf_index: usize,

    // Async catalog indexing
    pub indexing: bool,
    pub indexing_tick: u8,

    // Feature running status
    pub running_features: HashSet<String>,

    // Session attach
    pub attach_input: String,
    pub attach_request: bool,

    // Click regions registered during the current frame (cleared each draw).
    pub click_regions: RefCell<Vec<(Rect, ClickAction)>>,
}

impl App {
    pub fn new(project: Project, config: AppConfig) -> Self {
        let mode = config.theme_mode();
        let accent = config.accent();
        let theme = Theme::new(mode, accent);

        let project_name = project
            .root
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let project_path = project.root.display().to_string();
        let custom_layout = config.custom_layout.clone().unwrap_or_default();

        App {
            project,
            screen: Screen::Dashboard,
            layout: DashboardLayout::Overview,
            focused_pane: Pane::FeatureList,
            feature_index: 0,
            spec_tab: SpecTab::Spec,
            spec_scroll: 0,
            theme_mode: mode,
            accent,
            theme,
            should_quit: false,
            project_name,
            project_path,
            config,

            ext_tab: ExtTab::Extensions,
            ext_index: 0,
            preset_index: 0,

            active_popup: None,

            cli_job: None,
            cli_rx: None,
            pending_action: None,
            force_flag: false,

            tmux_available: false,
            tmux_session: None,

            settings_index: 0,

            palette: None,

            custom_layout,
            layout_editor_index: 0,
            layout_editor_active: false,

            agent_lines: Vec::new(),

            integration_index: 0,
            wf_index: 0,
            indexing: true,
            indexing_tick: 0,
            running_features: HashSet::new(),
            attach_input: String::new(),
            attach_request: false,
            click_regions: RefCell::new(Vec::new()),
        }
    }

    /// Display name of the default integration's coding agent, e.g. "Claude Code".
    pub fn default_agent_name(&self) -> String {
        self.project
            .integrations
            .iter()
            .find(|i| i.is_default)
            .map(|i| i.name.clone())
            .unwrap_or_else(|| "agent".to_string())
    }

    /// Register a clickable region for the current frame. Last-registered wins
    /// on overlap (popups/overlays are drawn after the content beneath them).
    pub fn register_click(&self, rect: Rect, action: ClickAction) {
        self.click_regions.borrow_mut().push((rect, action));
    }

    pub fn clear_click_regions(&self) {
        self.click_regions.borrow_mut().clear();
    }

    /// Hit-test a terminal cell (column,row) against registered regions,
    /// returning the topmost match.
    pub fn hit_test(&self, col: u16, row: u16) -> Option<ClickAction> {
        self.click_regions
            .borrow()
            .iter()
            .rev()
            .find(|(r, _)| {
                col >= r.x && col < r.x + r.width && row >= r.y && row < r.y + r.height
            })
            .map(|(_, a)| *a)
    }

    pub fn selected_feature(&self) -> Option<&spectatui_core::speckit::Feature> {
        self.project.features.get(self.feature_index)
    }

    pub fn selected_tasks_progress(&self) -> Option<TasksProgress> {
        self.selected_feature()
            .and_then(|f| f.artifacts.tasks.as_ref())
            .and_then(|p| TasksProgress::from_file(p))
    }

    pub fn selected_doc_content(&self) -> Option<String> {
        let feature = self.selected_feature()?;
        let path = match self.spec_tab {
            SpecTab::Spec => feature.artifacts.spec.as_ref(),
            SpecTab::Plan => feature.artifacts.plan.as_ref(),
            SpecTab::Tasks => feature.artifacts.tasks.as_ref(),
            SpecTab::Research => feature.artifacts.research.as_ref(),
        };
        path.and_then(|p| std::fs::read_to_string(p).ok())
    }

    pub fn constitution_content(&self) -> Option<String> {
        self.project
            .constitution
            .as_ref()
            .and_then(|p| std::fs::read_to_string(p).ok())
    }

    fn rebuild_theme(&mut self) {
        self.theme = Theme::new(self.theme_mode, self.accent);
    }

    pub fn toggle_theme(&mut self) {
        self.theme_mode = match self.theme_mode {
            ThemeMode::Dark => ThemeMode::Light,
            ThemeMode::Light => ThemeMode::Dark,
        };
        self.rebuild_theme();
        self.config.set_theme(self.theme_mode);
        let _ = config::save_config(&self.config);
    }

    pub fn cycle_accent(&mut self) {
        self.accent = self.accent.next();
        self.rebuild_theme();
        self.config.set_accent(self.accent);
        let _ = config::save_config(&self.config);
    }

    pub fn select_next_feature(&mut self) {
        if !self.project.features.is_empty() {
            self.feature_index = (self.feature_index + 1) % self.project.features.len();
            self.spec_scroll = 0;
        }
    }

    pub fn select_prev_feature(&mut self) {
        if !self.project.features.is_empty() {
            self.feature_index = if self.feature_index == 0 {
                self.project.features.len() - 1
            } else {
                self.feature_index - 1
            };
            self.spec_scroll = 0;
        }
    }

    pub fn cycle_tab_forward(&mut self) {
        match self.screen {
            Screen::Dashboard => {
                self.focused_pane = match self.focused_pane {
                    Pane::FeatureList => Pane::Workflow,
                    Pane::Workflow => Pane::AgentOutput,
                    Pane::AgentOutput => Pane::FeatureList,
                    other => other,
                };
            }
            Screen::SpecBrowser => {
                self.spec_tab = self.spec_tab.next();
                self.spec_scroll = 0;
            }
            Screen::ExtensionsPresets => {
                self.ext_tab = match self.ext_tab {
                    ExtTab::Extensions => ExtTab::Presets,
                    ExtTab::Presets => ExtTab::Extensions,
                };
            }
            _ => {}
        }
    }

    pub fn cycle_tab_backward(&mut self) {
        match self.screen {
            Screen::Dashboard => {
                self.focused_pane = match self.focused_pane {
                    Pane::FeatureList => Pane::AgentOutput,
                    Pane::Workflow => Pane::FeatureList,
                    Pane::AgentOutput => Pane::Workflow,
                    other => other,
                };
            }
            Screen::SpecBrowser => {
                self.spec_tab = self.spec_tab.prev();
                self.spec_scroll = 0;
            }
            Screen::ExtensionsPresets => {
                self.ext_tab = match self.ext_tab {
                    ExtTab::Extensions => ExtTab::Presets,
                    ExtTab::Presets => ExtTab::Extensions,
                };
            }
            _ => {}
        }
    }

    pub fn scroll_down(&mut self) {
        self.spec_scroll = self.spec_scroll.saturating_add(1);
    }

    pub fn scroll_up(&mut self) {
        self.spec_scroll = self.spec_scroll.saturating_sub(1);
    }

    pub fn enter_spec_browser(&mut self) {
        if self.selected_feature().is_some() {
            self.screen = Screen::SpecBrowser;
            self.spec_scroll = 0;
        }
    }

    pub fn enter_constitution(&mut self) {
        if self.project.constitution.is_some() {
            self.screen = Screen::Constitution;
            self.spec_scroll = 0;
        }
    }

    pub fn enter_extensions(&mut self) {
        self.screen = Screen::ExtensionsPresets;
        self.ext_tab = ExtTab::Extensions;
        self.ext_index = 0;
    }

    pub fn enter_settings(&mut self) {
        self.screen = Screen::Settings;
        self.settings_index = 0;
    }

    pub fn go_back(&mut self) {
        if self.active_popup.is_some() {
            self.active_popup = None;
            return;
        }
        if self.palette.is_some() {
            self.palette = None;
            return;
        }
        match self.screen {
            Screen::SpecBrowser | Screen::Constitution | Screen::ExtensionsPresets | Screen::Settings | Screen::SessionAttach => {
                self.screen = Screen::Dashboard;
                self.spec_scroll = 0;
            }
            Screen::Dashboard => {}
        }
    }

    pub fn open_popup(&mut self, kind: PopupKind) {
        match kind {
            PopupKind::Extensions => {
                self.ext_tab = ExtTab::Extensions;
                self.ext_index = 0;
            }
            PopupKind::Presets => {
                self.ext_tab = ExtTab::Presets;
                self.preset_index = 0;
            }
            _ => {}
        }
        self.active_popup = Some(kind);
    }

    pub fn close_popup(&mut self) {
        self.active_popup = None;
    }

    pub fn open_palette(&mut self) {
        self.palette = Some(PaletteState {
            input: String::new(),
            selected: 0,
        });
    }

    pub fn ext_list_len(&self) -> usize {
        match self.ext_tab {
            ExtTab::Extensions => self.project.extensions.len(),
            ExtTab::Presets => self.project.presets.len(),
        }
    }

    pub fn ext_select_next(&mut self) {
        let len = self.ext_list_len();
        if len > 0 {
            match self.ext_tab {
                ExtTab::Extensions => self.ext_index = (self.ext_index + 1) % len,
                ExtTab::Presets => self.preset_index = (self.preset_index + 1) % len,
            }
        }
    }

    pub fn ext_select_prev(&mut self) {
        let len = self.ext_list_len();
        if len > 0 {
            match self.ext_tab {
                ExtTab::Extensions => {
                    self.ext_index = if self.ext_index == 0 { len - 1 } else { self.ext_index - 1 }
                }
                ExtTab::Presets => {
                    self.preset_index = if self.preset_index == 0 { len - 1 } else { self.preset_index - 1 }
                }
            }
        }
    }

    pub fn settings_next(&mut self) {
        let len = SettingsRow::ALL.len();
        self.settings_index = (self.settings_index + 1) % len;
    }

    pub fn settings_prev(&mut self) {
        let len = SettingsRow::ALL.len();
        self.settings_index = if self.settings_index == 0 {
            len - 1
        } else {
            self.settings_index - 1
        };
    }

    pub fn settings_cycle_value(&mut self) {
        let row = SettingsRow::ALL[self.settings_index];
        match row {
            SettingsRow::Theme => self.toggle_theme(),
            SettingsRow::Accent => self.cycle_accent(),
            SettingsRow::DashboardLayout => {
                self.layout = match self.layout {
                    DashboardLayout::Overview => DashboardLayout::Coding,
                    DashboardLayout::Coding => DashboardLayout::Audit,
                    DashboardLayout::Audit => DashboardLayout::Custom,
                    DashboardLayout::Custom => DashboardLayout::Overview,
                };
            }
            SettingsRow::AgentTailFollow => {
                self.config.agent_tail_follow = !self.config.agent_tail_follow;
                let _ = config::save_config(&self.config);
            }
            SettingsRow::MouseSupport => {
                self.config.mouse_support = !self.config.mouse_support;
                let _ = config::save_config(&self.config);
            }
            SettingsRow::ConfirmForce => {
                self.config.confirm_before_force = !self.config.confirm_before_force;
                let _ = config::save_config(&self.config);
            }
            SettingsRow::TmuxPrefix => {}
            SettingsRow::CustomizePanes => {
                self.layout_editor_active = true;
                self.layout_editor_index = 0;
            }
            SettingsRow::AttachSession => {
                self.screen = Screen::SessionAttach;
            }
            SettingsRow::ConfigPath => {}
        }
    }

    /// Set an option row directly to one of its chip values (used by mouse clicks).
    pub fn settings_set_value(&mut self, row: SettingsRow, value: &str) {
        match row {
            SettingsRow::Theme => {
                let want_dark = value == "dark";
                if (self.theme_mode == ThemeMode::Dark) != want_dark {
                    self.toggle_theme();
                }
            }
            SettingsRow::Accent => {
                while self.accent_label() != value {
                    self.cycle_accent();
                }
            }
            SettingsRow::DashboardLayout => {
                self.layout = match value {
                    "coding" => DashboardLayout::Coding,
                    "audit" => DashboardLayout::Audit,
                    "custom" => DashboardLayout::Custom,
                    _ => DashboardLayout::Overview,
                };
            }
            SettingsRow::AgentTailFollow => {
                self.config.agent_tail_follow = value == "on";
                let _ = config::save_config(&self.config);
            }
            SettingsRow::MouseSupport => {
                self.config.mouse_support = value == "on";
                let _ = config::save_config(&self.config);
            }
            SettingsRow::ConfirmForce => {
                self.config.confirm_before_force = value == "always";
                let _ = config::save_config(&self.config);
            }
            _ => {}
        }
    }

    pub fn settings_value_str(&self, row: SettingsRow) -> String {
        match row {
            SettingsRow::Theme => self.theme_label().to_string(),
            SettingsRow::Accent => self.accent_label().to_string(),
            SettingsRow::DashboardLayout => match self.layout {
                DashboardLayout::Overview => "overview".to_string(),
                DashboardLayout::Coding => "coding".to_string(),
                DashboardLayout::Audit => "audit".to_string(),
                DashboardLayout::Custom => "custom".to_string(),
            },
            SettingsRow::AgentTailFollow => if self.config.agent_tail_follow { "on" } else { "off" }.to_string(),
            SettingsRow::MouseSupport => if self.config.mouse_support { "on" } else { "off" }.to_string(),
            SettingsRow::ConfirmForce => if self.config.confirm_before_force { "always" } else { "never" }.to_string(),
            SettingsRow::TmuxPrefix => "spectatui-".to_string(),
            SettingsRow::CustomizePanes => "open layout editor →".to_string(),
            SettingsRow::AttachSession => format!(
                "{} →",
                self.selected_feature().map(|f| f.id.as_str()).unwrap_or("none")
            ),
            SettingsRow::ConfigPath => config::config_path_display(),
        }
    }

    /// Apply a tmux poll result: recompute which features have live sessions
    /// and store the snapshot for the selected feature.
    pub fn apply_tmux(&mut self, sessions: Vec<String>, session: Option<TmuxSession>) {
        self.running_features = self
            .project
            .features
            .iter()
            .filter(|f| sessions.iter().any(|s| s.contains(&f.id)))
            .map(|f| f.id.clone())
            .collect();
        self.tmux_session = session;
    }

    /// The tmux target (session name) for the currently attached session, if any.
    pub fn attach_target(&self) -> Option<String> {
        self.tmux_session.as_ref().map(|s| s.name.clone())
    }

    pub fn refresh_project(&mut self) {
        if let Ok(project) = Project::discover(std::path::Path::new(&self.project_path)) {
            self.project = project;
            if self.feature_index >= self.project.features.len() {
                self.feature_index = self.project.features.len().saturating_sub(1);
            }
        }
    }

    pub fn poll_cli_job(&mut self) {
        let Some(rx) = &mut self.cli_rx else { return };
        let mut should_refresh = false;
        loop {
            match rx.try_recv() {
                Ok(CliEvent::OutputLine(line)) => {
                    if let Some(job) = &mut self.cli_job {
                        job.status = JobStatus::Running;
                        if !job.output.is_empty() {
                            job.output.push('\n');
                        }
                        job.output.push_str(&line);
                    }
                }
                Ok(CliEvent::Completed { success }) => {
                    if let Some(job) = &mut self.cli_job {
                        job.status = if success {
                            JobStatus::Succeeded
                        } else {
                            JobStatus::Failed
                        };
                    }
                    if success {
                        should_refresh = true;
                    }
                }
                Err(_) => break,
            }
        }
        if should_refresh {
            self.refresh_project();
        }
    }

    pub fn integration_select_next(&mut self) {
        let len = self.project.integrations.len();
        if len > 0 {
            self.integration_index = (self.integration_index + 1) % len;
        }
    }

    pub fn integration_select_prev(&mut self) {
        let len = self.project.integrations.len();
        if len > 0 {
            self.integration_index = if self.integration_index == 0 {
                len - 1
            } else {
                self.integration_index - 1
            };
        }
    }

    pub fn wf_select_next(&mut self) {
        let len = self.project.workflows.len();
        if len > 0 {
            self.wf_index = (self.wf_index + 1) % len;
        }
    }

    pub fn wf_select_prev(&mut self) {
        let len = self.project.workflows.len();
        if len > 0 {
            self.wf_index = if self.wf_index == 0 {
                len - 1
            } else {
                self.wf_index - 1
            };
        }
    }

    pub fn merge_catalog_results(
        &mut self,
        available_integrations: Vec<IntegrationInfo>,
        available_extensions: Vec<ExtensionInfo>,
        available_presets: Vec<PresetInfo>,
        workflows: Vec<WorkflowInfo>,
    ) {
        // Merge integrations: keep installed entries, add available ones not already present
        for avail in available_integrations {
            if !self.project.integrations.iter().any(|i| i.key == avail.key) {
                self.project.integrations.push(avail);
            } else if let Some(existing) = self.project.integrations.iter_mut().find(|i| i.key == avail.key) {
                if existing.description.is_empty() && !avail.description.is_empty() {
                    existing.description = avail.description;
                }
            }
        }

        // Merge extensions
        for avail in available_extensions {
            if !self.project.extensions.iter().any(|e| e.id == avail.id) {
                self.project.extensions.push(avail);
            }
        }

        // Merge presets
        for avail in available_presets {
            if !self.project.presets.iter().any(|p| p.id == avail.id) {
                self.project.presets.push(avail);
            }
        }

        self.project.workflows = workflows;
        self.indexing = false;
    }

    pub fn accent_label(&self) -> &'static str {
        match self.accent {
            Accent::Indigo => "indigo",
            Accent::Teal => "teal",
            Accent::Amber => "amber",
        }
    }

    pub fn theme_label(&self) -> &'static str {
        match self.theme_mode {
            ThemeMode::Dark => "dark",
            ThemeMode::Light => "light",
        }
    }
}

pub struct PaletteCommand {
    pub label: &'static str,
    pub hint: &'static str,
    pub action: PaletteAction,
}

pub enum PaletteAction {
    SetLayout(DashboardLayout),
    SetScreen(Screen),
    ToggleTheme,
    CycleAccent,
    OpenPopup(PopupKind),
    Quit,
}

/// A clickable action, registered against a screen rect during rendering and
/// dispatched on a left mouse click. Mirrors the design mock's reg/onClick model.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClickAction {
    OpenPopup(PopupKind),
    SetScreen(Screen),
    EnterExtensions,
    OpenSettings,
    SetLayout(DashboardLayout),
    FocusPane(Pane),
    SelectFeature(usize),
    SelectExt(usize),
    SelectPreset(usize),
    SelectIntegration(usize),
    SelectWorkflow(usize),
    SetExtTab(ExtTab),
    SetSpecTab(SpecTab),
    SettingsSelect(usize),
    SettingsChip(SettingsRow, usize),
    LayoutEditorSelect(usize),
    PaletteRun(usize),
    JumpToFeature(usize),
}

pub fn palette_commands() -> Vec<PaletteCommand> {
    vec![
        PaletteCommand { label: "Go to Dashboard", hint: "d", action: PaletteAction::SetScreen(Screen::Dashboard) },
        PaletteCommand { label: "Go to Spec Browser", hint: "s", action: PaletteAction::SetScreen(Screen::SpecBrowser) },
        PaletteCommand { label: "Go to Constitution", hint: "c", action: PaletteAction::SetScreen(Screen::Constitution) },
        PaletteCommand { label: "Show Features", hint: "f", action: PaletteAction::OpenPopup(PopupKind::Features) },
        PaletteCommand { label: "Manage Integrations", hint: "i", action: PaletteAction::OpenPopup(PopupKind::Integrations) },
        PaletteCommand { label: "Manage Extensions", hint: "e", action: PaletteAction::OpenPopup(PopupKind::Extensions) },
        PaletteCommand { label: "Manage Presets", hint: "p", action: PaletteAction::OpenPopup(PopupKind::Presets) },
        PaletteCommand { label: "Manage Workflows", hint: "w", action: PaletteAction::OpenPopup(PopupKind::Workflows) },
        PaletteCommand { label: "Open Settings", hint: "⚙", action: PaletteAction::SetScreen(Screen::Settings) },
        PaletteCommand { label: "Layout: Overview", hint: "1", action: PaletteAction::SetLayout(DashboardLayout::Overview) },
        PaletteCommand { label: "Layout: Coding", hint: "2", action: PaletteAction::SetLayout(DashboardLayout::Coding) },
        PaletteCommand { label: "Layout: Audit", hint: "3", action: PaletteAction::SetLayout(DashboardLayout::Audit) },
        PaletteCommand { label: "Layout: Custom (edited)", hint: "4", action: PaletteAction::SetLayout(DashboardLayout::Custom) },
        PaletteCommand { label: "Attach agent session", hint: "a", action: PaletteAction::SetScreen(Screen::SessionAttach) },
        PaletteCommand { label: "Toggle theme (dark / light)", hint: "t", action: PaletteAction::ToggleTheme },
        PaletteCommand { label: "Cycle accent palette", hint: "T", action: PaletteAction::CycleAccent },
        PaletteCommand { label: "Help", hint: "?", action: PaletteAction::OpenPopup(PopupKind::Help) },
        PaletteCommand { label: "Quit", hint: "q", action: PaletteAction::Quit },
    ]
}
