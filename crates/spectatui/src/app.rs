#![allow(dead_code)]

use std::cell::{Cell, RefCell};
use std::collections::HashSet;

use ratatui::layout::Rect;
use spectatui_core::layout::CustomLayout;
use spectatui_core::speckit::cli::{CliAction, CliEvent, CliJob, JobStatus};
use spectatui_core::speckit::registry::{CatalogSource, CatalogTarget};
use spectatui_core::speckit::{
    ExtensionInfo, IntegrationInfo, PresetInfo, Project, TasksProgress, WorkflowInfo,
};
use spectatui_core::tmux::TmuxSession;
use tokio::sync::mpsc;

use crate::config::{self, AppConfig};
use crate::theme::{Accent, Theme, ThemeMode};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Dashboard,
    SpecBrowser,
    Constitution,
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
    Catalogs,
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
            Self::ConfigPath => config::CONFIG_LOCATIONS,
            _ => &[],
        }
    }

    /// Whether this row is a free-text field (edited inline) rather than a
    /// chip/option or action row.
    pub fn is_text(&self) -> bool {
        matches!(self, Self::TmuxPrefix)
    }
}

pub struct PaletteState {
    pub input: String,
    pub selected: usize,
}

/// Snapshot of the one-shot async catalog fetch, re-applied after every project
/// re-discovery so the popups keep their "available" items and workflows.
#[derive(Clone, Default)]
pub struct CatalogResults {
    pub integrations: Vec<IntegrationInfo>,
    pub extensions: Vec<ExtensionInfo>,
    pub presets: Vec<PresetInfo>,
    pub workflows: Vec<WorkflowInfo>,
}

/// Catalog *sources* for the Catalog Manager popup — one list per resource kind.
/// Distinct from `CatalogResults` above, which caches "available items" fetched
/// through those sources, not the sources themselves.
#[derive(Clone, Default)]
pub struct CatalogSourcesState {
    pub extensions: Vec<CatalogSource>,
    pub presets: Vec<CatalogSource>,
    pub integrations: Vec<CatalogSource>,
    pub workflows: Vec<CatalogSource>,
}

impl CatalogSourcesState {
    fn for_target(&self, target: CatalogTarget) -> &[CatalogSource] {
        match target {
            CatalogTarget::Extension => &self.extensions,
            CatalogTarget::Preset => &self.presets,
            CatalogTarget::Integration => &self.integrations,
            CatalogTarget::Workflow => &self.workflows,
        }
    }

    fn for_target_mut(&mut self, target: CatalogTarget) -> &mut Vec<CatalogSource> {
        match target {
            CatalogTarget::Extension => &mut self.extensions,
            CatalogTarget::Preset => &mut self.presets,
            CatalogTarget::Integration => &mut self.integrations,
            CatalogTarget::Workflow => &mut self.workflows,
        }
    }

    pub fn total(&self) -> usize {
        self.extensions.len() + self.presets.len() + self.integrations.len() + self.workflows.len()
    }
}

pub struct App {
    pub project: Project,
    pub screen: Screen,
    pub layout: DashboardLayout,
    pub focused_pane: Pane,
    pub feature_index: usize,
    pub spec_tab: SpecTab,
    pub spec_scroll: u16,
    /// Max `spec_scroll` for the doc view rendered last frame (`total_lines - viewport`).
    pub doc_scroll_max: Cell<u16>,
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
    pub cli_scroll: u16,
    /// Max `cli_scroll` for the CLI output popup rendered last frame.
    pub cli_scroll_max: Cell<u16>,
    pub pending_action: Option<CliAction>,
    pub force_flag: bool,

    // Tmux
    pub tmux_available: bool,
    pub tmux_session: Option<TmuxSession>,

    // Settings
    pub settings_index: usize,
    /// Row index currently being text-edited, if any.
    pub settings_editing: Option<usize>,

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

    // Catalog source management (Catalog Manager popup)
    pub cat_tab: CatalogTarget,
    pub cat_index: usize,
    /// `Some(buffer)` while the inline "add source" form is active; `None` otherwise.
    pub cat_add_input: Option<String>,
    /// Char index (not byte index) into `cat_add_input`. Only meaningful while
    /// `cat_add_input` is `Some`.
    pub cat_add_cursor: usize,
    pub catalog_sources: CatalogSourcesState,
    /// Set by the `r` (refresh) key; the main loop spawns the fetch and clears this.
    pub catalog_refresh_request: Option<CatalogTarget>,

    // Inline list filter (management popups)
    pub filter_query: String,
    pub filter_active: bool,

    // Async catalog indexing
    pub indexing: bool,
    pub indexing_tick: u8,
    /// Whether the `specify` CLI was runnable during catalog indexing. `false` means
    /// the "available" items couldn't be loaded — the popups show a notice instead of
    /// a silently empty list. Starts `true` so no error flashes before the probe runs.
    pub cli_available: bool,
    /// Last async catalog results, re-applied after every project refresh so a
    /// re-discovery (file watcher / CLI job) doesn't drop the "available" items.
    pub catalog_cache: Option<CatalogResults>,

    // Feature running status
    pub running_features: HashSet<String>,

    // Session attach
    pub attach_input: String,
    pub attach_request: bool,
    pub launch_request: bool,

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
            doc_scroll_max: Cell::new(0),
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
            cli_scroll: 0,
            cli_scroll_max: Cell::new(0),
            pending_action: None,
            force_flag: false,

            tmux_available: false,
            tmux_session: None,

            settings_index: 0,
            settings_editing: None,

            palette: None,

            custom_layout,
            layout_editor_index: 0,
            layout_editor_active: false,

            agent_lines: Vec::new(),

            integration_index: 0,
            wf_index: 0,
            cat_tab: CatalogTarget::Extension,
            cat_index: 0,
            cat_add_input: None,
            cat_add_cursor: 0,
            catalog_sources: CatalogSourcesState::default(),
            catalog_refresh_request: None,
            filter_query: String::new(),
            filter_active: false,
            indexing: true,
            indexing_tick: 0,
            cli_available: true,
            catalog_cache: None,
            running_features: HashSet::new(),
            attach_input: String::new(),
            attach_request: false,
            launch_request: false,
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

    /// Invokable command for the default integration's coding agent, e.g. "claude".
    pub fn default_agent_key(&self) -> Option<String> {
        self.project
            .integrations
            .iter()
            .find(|i| i.is_default)
            .map(|i| i.key.clone())
    }

    /// tmux session name spectatui creates/looks for a feature's coding-agent session.
    pub fn session_name_for(&self, feature_id: &str) -> String {
        format!("{}{}", self.config.tmux_prefix, feature_id)
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
            .find(|(r, _)| col >= r.x && col < r.x + r.width && row >= r.y && row < r.y + r.height)
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
            _ => {}
        }
    }

    pub fn scroll_down(&mut self) {
        self.spec_scroll = self
            .spec_scroll
            .saturating_add(1)
            .min(self.doc_scroll_max.get());
    }

    pub fn scroll_up(&mut self) {
        self.spec_scroll = self.spec_scroll.saturating_sub(1);
    }

    pub fn cli_scroll_down(&mut self) {
        self.cli_scroll = self
            .cli_scroll
            .saturating_add(1)
            .min(self.cli_scroll_max.get());
    }

    pub fn cli_scroll_up(&mut self) {
        self.cli_scroll = self.cli_scroll.saturating_sub(1);
    }

    /// Show the CLI output popup for a freshly spawned job, resetting scroll.
    pub fn show_cli_job(&mut self, job: CliJob, rx: mpsc::UnboundedReceiver<CliEvent>) {
        self.cli_job = Some(job);
        self.cli_rx = Some(rx);
        self.cli_scroll = 0;
        self.active_popup = Some(PopupKind::CliOutput);
    }

    /// At most one CLI-mediated action may run at a time (spec FR-019a) — `false` while
    /// a previous job hasn't reached a terminal status yet. A freshly spawned job starts
    /// as `Pending` and only flips to `Running` once its first output line arrives (or
    /// never, if it completes with no output), so both non-terminal states must block.
    pub fn can_start_cli_action(&self) -> bool {
        !matches!(
            &self.cli_job,
            Some(job) if matches!(job.status, JobStatus::Pending | JobStatus::Running)
        )
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

    pub fn enter_settings(&mut self) {
        self.screen = Screen::Settings;
        self.settings_index = 0;
        self.settings_editing = None;
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
            Screen::SpecBrowser
            | Screen::Constitution
            | Screen::Settings
            | Screen::SessionAttach => {
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
            PopupKind::Catalogs => {
                self.cat_index = 0;
                self.cat_add_input = None;
                self.cat_add_cursor = 0;
            }
            _ => {}
        }
        self.reset_filter();
        self.active_popup = Some(kind);
    }

    pub fn close_popup(&mut self) {
        self.reset_filter();
        self.active_popup = None;
    }

    /// Open the Catalog Manager, always resetting to the Extensions tab
    /// (FR-014) — used by the command-palette entry, which the mockup opens on
    /// a fixed, predictable tab regardless of what was last viewed.
    pub fn open_catalogs_reset_to_extensions(&mut self) {
        self.cat_tab = CatalogTarget::Extension;
        self.open_popup(PopupKind::Catalogs);
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
        let len = self.filtered_ext_len();
        if len > 0 {
            match self.ext_tab {
                ExtTab::Extensions => self.ext_index = (self.ext_index + 1) % len,
                ExtTab::Presets => self.preset_index = (self.preset_index + 1) % len,
            }
        }
    }

    pub fn ext_select_prev(&mut self) {
        let len = self.filtered_ext_len();
        if len > 0 {
            match self.ext_tab {
                ExtTab::Extensions => {
                    self.ext_index = if self.ext_index == 0 {
                        len - 1
                    } else {
                        self.ext_index - 1
                    }
                }
                ExtTab::Presets => {
                    self.preset_index = if self.preset_index == 0 {
                        len - 1
                    } else {
                        self.preset_index - 1
                    }
                }
            }
        }
    }

    pub fn settings_next(&mut self) {
        self.settings_editing = None;
        let len = SettingsRow::ALL.len();
        self.settings_index = (self.settings_index + 1) % len;
    }

    pub fn settings_prev(&mut self) {
        self.settings_editing = None;
        let len = SettingsRow::ALL.len();
        self.settings_index = if self.settings_index == 0 {
            len - 1
        } else {
            self.settings_index - 1
        };
    }

    /// Cycle the selected option row by `delta` (+1 forward, -1 back). No-op for
    /// text/action rows.
    pub fn settings_adjust(&mut self, delta: i32) {
        let row = SettingsRow::ALL[self.settings_index];
        let opts = row.options();
        if opts.is_empty() {
            return;
        }
        let current = self.settings_value_str(row);
        let idx = opts.iter().position(|o| *o == current).unwrap_or(0) as i32;
        let n = opts.len() as i32;
        let next = (idx + delta).rem_euclid(n) as usize;
        self.settings_set_value(row, opts[next]);
    }

    /// Enter/→ on the selected row: text → begin editing, option → cycle forward,
    /// action → run its action.
    pub fn settings_primary_action(&mut self) {
        let row = SettingsRow::ALL[self.settings_index];
        if row.is_text() {
            self.settings_begin_edit();
        } else if !row.options().is_empty() {
            self.settings_adjust(1);
        } else {
            match row {
                SettingsRow::CustomizePanes => {
                    self.layout_editor_active = true;
                    self.layout_editor_index = 0;
                }
                SettingsRow::AttachSession => {
                    self.screen = Screen::SessionAttach;
                }
                _ => {}
            }
        }
    }

    pub fn settings_begin_edit(&mut self) {
        if SettingsRow::ALL[self.settings_index].is_text() {
            self.settings_editing = Some(self.settings_index);
        }
    }

    pub fn settings_end_edit(&mut self) {
        let _ = config::save_config(&self.config);
        self.settings_editing = None;
    }

    pub fn settings_edit_push(&mut self, c: char) {
        if self.settings_editing.is_some()
            && SettingsRow::ALL[self.settings_index] == SettingsRow::TmuxPrefix
        {
            self.config.tmux_prefix.push(c);
        }
    }

    pub fn settings_edit_backspace(&mut self) {
        if self.settings_editing.is_some()
            && SettingsRow::ALL[self.settings_index] == SettingsRow::TmuxPrefix
        {
            self.config.tmux_prefix.pop();
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
            SettingsRow::ConfigPath => {
                if value == self.config.config_location {
                    return;
                }
                let old_path = config::resolve_config_location(&self.config.config_location);
                self.config.config_location = value.to_string();
                let _ = config::save_config(&self.config);
                // Migrate: drop the previous config file so loading stays deterministic.
                let new_path = config::resolve_config_location(&self.config.config_location);
                if let Some(old) = old_path {
                    if Some(&old) != new_path.as_ref() && old.exists() {
                        let _ = std::fs::remove_file(&old);
                    }
                }
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
            SettingsRow::AgentTailFollow => if self.config.agent_tail_follow {
                "on"
            } else {
                "off"
            }
            .to_string(),
            SettingsRow::MouseSupport => if self.config.mouse_support {
                "on"
            } else {
                "off"
            }
            .to_string(),
            SettingsRow::ConfirmForce => if self.config.confirm_before_force {
                "always"
            } else {
                "never"
            }
            .to_string(),
            SettingsRow::TmuxPrefix => self.config.tmux_prefix.clone(),
            SettingsRow::CustomizePanes => "open layout editor →".to_string(),
            SettingsRow::AttachSession => format!(
                "{} →",
                self.selected_feature()
                    .map(|f| f.id.as_str())
                    .unwrap_or("none")
            ),
            SettingsRow::ConfigPath => self.config.config_location.clone(),
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

        // When "agent tail follow" is off, freeze the visible tail: keep the previous
        // snapshot for the same session instead of adopting the freshly captured one.
        let session = match session {
            Some(mut new) if !self.config.agent_tail_follow => {
                if let Some(old) = &self.tmux_session {
                    if old.name == new.name {
                        new.last_snapshot = old.last_snapshot.clone();
                    }
                }
                Some(new)
            }
            other => other,
        };
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
            // Re-discovery loads installed-only data (and empties workflows); re-apply
            // the catalog results so the popups keep their "available" items.
            self.apply_catalog_cache();
        }
    }

    /// Polls the in-flight CLI job's event stream. Returns `Some(target)` when a
    /// `CatalogAdd`/`CatalogRemove` job for that catalog kind just succeeded — the
    /// generic `refresh_project()` below covers extensions/presets/integrations/
    /// workflows, but not `catalog_sources`, which the caller must re-fetch itself
    /// (see `main.rs`'s main loop).
    pub fn poll_cli_job(&mut self) -> Option<CatalogTarget> {
        let Some(rx) = &mut self.cli_rx else {
            return None;
        };
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
            if let Some(
                CliAction::CatalogAdd { target, .. } | CliAction::CatalogRemove { target, .. },
            ) = self.cli_job.as_ref().map(|j| &j.action)
            {
                return Some(*target);
            }
        }
        None
    }

    pub fn integration_select_next(&mut self) {
        let len = self.filtered_integrations().len();
        if len > 0 {
            self.integration_index = (self.integration_index + 1) % len;
        }
    }

    pub fn integration_select_prev(&mut self) {
        let len = self.filtered_integrations().len();
        if len > 0 {
            self.integration_index = if self.integration_index == 0 {
                len - 1
            } else {
                self.integration_index - 1
            };
        }
    }

    pub fn wf_select_next(&mut self) {
        let len = self.filtered_workflows().len();
        if len > 0 {
            self.wf_index = (self.wf_index + 1) % len;
        }
    }

    pub fn wf_select_prev(&mut self) {
        let len = self.filtered_workflows().len();
        if len > 0 {
            self.wf_index = if self.wf_index == 0 {
                len - 1
            } else {
                self.wf_index - 1
            };
        }
    }

    /// The catalog sources for the currently active `cat_tab`. Unlike the other
    /// manager popups, the Catalog Manager has no `/` filter, so this is a plain
    /// slice, not a filtered `Vec`.
    pub fn current_catalog_list(&self) -> &[CatalogSource] {
        self.catalog_sources.for_target(self.cat_tab)
    }

    pub fn cat_select_next(&mut self) {
        let len = self.current_catalog_list().len();
        if len > 0 {
            self.cat_index = (self.cat_index + 1) % len;
        }
    }

    pub fn cat_select_prev(&mut self) {
        let len = self.current_catalog_list().len();
        if len > 0 {
            self.cat_index = if self.cat_index == 0 {
                len - 1
            } else {
                self.cat_index - 1
            };
        }
    }

    /// Clamp `cat_index` after the active tab's list shrinks (e.g. a successful
    /// removal or a refresh returning fewer sources) so it never indexes out of
    /// bounds.
    fn clamp_cat_index(&mut self) {
        let len = self.current_catalog_list().len();
        if len == 0 {
            self.cat_index = 0;
        } else if self.cat_index >= len {
            self.cat_index = len - 1;
        }
    }

    /// Replace the stored sources for `target` (initial load or manual/post-mutation
    /// refresh), clamping selection if the active tab's list just shrank.
    pub fn set_catalog_sources(&mut self, target: CatalogTarget, sources: Vec<CatalogSource>) {
        *self.catalog_sources.for_target_mut(target) = sources;
        if target == self.cat_tab {
            self.clamp_cat_index();
        }
    }

    // ---- "add source" input editing (cursor-aware; char, not byte, indices) ----

    fn cat_add_char_count(&self) -> usize {
        self.cat_add_input
            .as_deref()
            .map(|s| s.chars().count())
            .unwrap_or(0)
    }

    /// Byte offset in `cat_add_input` corresponding to char index `idx`.
    fn cat_add_byte_offset(&self, idx: usize) -> usize {
        let input = self.cat_add_input.as_deref().unwrap_or("");
        input
            .char_indices()
            .nth(idx)
            .map(|(i, _)| i)
            .unwrap_or(input.len())
    }

    /// Insert `s` at the cursor, advancing it past the inserted text. `\n`/`\r`
    /// are stripped — this is a single-line field — so both regular typing (a
    /// 1-char string) and pasted clipboard text can share this one method.
    pub fn cat_add_insert_str(&mut self, s: &str) {
        let cleaned: String = s.chars().filter(|c| *c != '\n' && *c != '\r').collect();
        if cleaned.is_empty() {
            return;
        }
        let inserted = cleaned.chars().count();
        let offset = self.cat_add_byte_offset(self.cat_add_cursor);
        if let Some(input) = &mut self.cat_add_input {
            input.insert_str(offset, &cleaned);
        }
        self.cat_add_cursor += inserted;
    }

    pub fn cat_add_backspace(&mut self) {
        if self.cat_add_cursor == 0 || self.cat_add_input.is_none() {
            return;
        }
        let end = self.cat_add_byte_offset(self.cat_add_cursor);
        self.cat_add_cursor -= 1;
        let start = self.cat_add_byte_offset(self.cat_add_cursor);
        if let Some(input) = &mut self.cat_add_input {
            input.replace_range(start..end, "");
        }
    }

    pub fn cat_add_delete_forward(&mut self) {
        if self.cat_add_cursor >= self.cat_add_char_count() {
            return;
        }
        let start = self.cat_add_byte_offset(self.cat_add_cursor);
        let end = self.cat_add_byte_offset(self.cat_add_cursor + 1);
        if let Some(input) = &mut self.cat_add_input {
            input.replace_range(start..end, "");
        }
    }

    pub fn cat_add_move_left(&mut self) {
        self.cat_add_cursor = self.cat_add_cursor.saturating_sub(1);
    }

    pub fn cat_add_move_right(&mut self) {
        if self.cat_add_cursor < self.cat_add_char_count() {
            self.cat_add_cursor += 1;
        }
    }

    pub fn cat_add_move_home(&mut self) {
        self.cat_add_cursor = 0;
    }

    pub fn cat_add_move_end(&mut self) {
        self.cat_add_cursor = self.cat_add_char_count();
    }

    /// Clamp to `[0, char_count]` — used by the mouse click handler.
    pub fn cat_add_set_cursor(&mut self, pos: usize) {
        self.cat_add_cursor = pos.min(self.cat_add_char_count());
    }

    /// Clear the add-source input's text and reset the cursor to the start —
    /// the form itself stays open (unlike `Esc`, which cancels it entirely).
    pub fn cat_add_clear(&mut self) {
        self.cat_add_input = Some(String::new());
        self.cat_add_cursor = 0;
    }

    /// Open the "add source" input, prefilled from the currently selected
    /// source in `current_catalog_list()` as `"url name [priority]"` — but
    /// only when that source is "discovery only" (`install_allowed == false`,
    /// i.e. not yet installed), since prefilling an already-installed source
    /// would just offer to re-add itself. Falls back to an empty field
    /// otherwise (nothing selected, empty list, or an installed source).
    pub fn cat_add_open(&mut self) {
        let prefill = self
            .current_catalog_list()
            .get(self.cat_index)
            .filter(|src| !src.install_allowed)
            .map(|src| match src.priority {
                Some(p) => format!("{} {} {}", src.url, src.name, p),
                None => format!("{} {}", src.url, src.name),
            })
            .unwrap_or_default();
        self.cat_add_cursor = prefill.chars().count();
        self.cat_add_input = Some(prefill);
    }

    // ---- inline list filter ----
    fn matches_filter(&self, haystack: &str) -> bool {
        let q = self.filter_query.trim().to_lowercase();
        q.is_empty() || haystack.to_lowercase().contains(&q)
    }

    pub fn filtered_integrations(&self) -> Vec<&IntegrationInfo> {
        self.project
            .integrations
            .iter()
            .filter(|it| self.matches_filter(&format!("{} {} {}", it.name, it.key, it.description)))
            .collect()
    }

    pub fn filtered_workflows(&self) -> Vec<&WorkflowInfo> {
        self.project
            .workflows
            .iter()
            .filter(|wf| {
                let name = wf.name.as_deref().unwrap_or("");
                let src = wf.source.as_deref().unwrap_or("");
                self.matches_filter(&format!("{} {} {} {}", name, wf.id, src, wf.description))
            })
            .collect()
    }

    pub fn filtered_extensions(&self) -> Vec<&ExtensionInfo> {
        self.project
            .extensions
            .iter()
            .filter(|e| {
                let by = e.author.as_deref().unwrap_or("");
                self.matches_filter(&format!("{} {} {} {}", e.name, e.id, by, e.description))
            })
            .collect()
    }

    pub fn filtered_presets(&self) -> Vec<&PresetInfo> {
        self.project
            .presets
            .iter()
            .filter(|p| {
                let by = p.author.as_deref().unwrap_or("");
                let src = p.source_label.as_deref().unwrap_or("");
                self.matches_filter(&format!(
                    "{} {} {} {} {}",
                    p.name, p.id, by, src, p.description
                ))
            })
            .collect()
    }

    pub fn filtered_ext_len(&self) -> usize {
        match self.ext_tab {
            ExtTab::Extensions => self.filtered_extensions().len(),
            ExtTab::Presets => self.filtered_presets().len(),
        }
    }

    pub fn reset_filter(&mut self) {
        self.filter_query.clear();
        self.filter_active = false;
    }

    pub fn merge_catalog_results(
        &mut self,
        available_integrations: Vec<IntegrationInfo>,
        available_extensions: Vec<ExtensionInfo>,
        available_presets: Vec<PresetInfo>,
        workflows: Vec<WorkflowInfo>,
    ) {
        self.catalog_cache = Some(CatalogResults {
            integrations: available_integrations,
            extensions: available_extensions,
            presets: available_presets,
            workflows,
        });
        self.apply_catalog_cache();
        self.indexing = false;
    }

    /// Merge the cached catalog results into the current project. Idempotent: safe
    /// to call after every `refresh_project` so a re-discovery keeps "available"
    /// items (and workflows, which `Project::discover` never populates).
    pub fn apply_catalog_cache(&mut self) {
        let Some(cache) = self.catalog_cache.clone() else {
            return;
        };

        // Merge integrations: append new, else backfill catalog metadata into the
        // installed entry (name/description) without overriding local state.
        for avail in cache.integrations {
            if let Some(existing) = self
                .project
                .integrations
                .iter_mut()
                .find(|i| i.key == avail.key)
            {
                if existing.name == existing.key && !avail.name.is_empty() {
                    existing.name = avail.name;
                }
                if existing.description.is_empty() && !avail.description.is_empty() {
                    existing.description = avail.description;
                }
            } else {
                self.project.integrations.push(avail);
            }
        }

        // Merge extensions: append new, else backfill catalog metadata.
        for avail in cache.extensions {
            if let Some(existing) = self
                .project
                .extensions
                .iter_mut()
                .find(|e| e.id == avail.id)
            {
                if existing.name == existing.id {
                    existing.name = avail.name;
                }
                if existing.author.is_none() {
                    existing.author = avail.author;
                }
                if existing.description.is_empty() {
                    existing.description = avail.description;
                }
                // Registry marks bundled extensions as `Local`; prefer the catalog
                // provenance when we have it.
                if matches!(
                    existing.source,
                    spectatui_core::speckit::ExtensionSource::Local
                ) {
                    existing.source = avail.source;
                }
            } else {
                self.project.extensions.push(avail);
            }
        }

        // Merge presets: append new, else backfill catalog metadata.
        for avail in cache.presets {
            if let Some(existing) = self.project.presets.iter_mut().find(|p| p.id == avail.id) {
                if existing.name == existing.id {
                    existing.name = avail.name;
                }
                if existing.author.is_none() {
                    existing.author = avail.author;
                }
                if existing.description.is_empty() {
                    existing.description = avail.description;
                }
                if existing.source_label.is_none() {
                    existing.source_label = avail.source_label;
                }
                if existing.template_count == 0 && avail.template_count > 0 {
                    existing.template_count = avail.template_count;
                }
            } else {
                self.project.presets.push(avail);
            }
        }

        self.project.workflows = cache.workflows;
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

/// Parse the Catalog Manager's inline "add source" input, `"url name [priority]"`,
/// into `(url, name, priority)`. Returns `None` when the name is missing (a bare
/// URL with nothing else) or the input is empty — there is no reasonable default
/// for a source name, so this is treated as malformed rather than guessed at.
pub fn parse_catalog_add_input(input: &str) -> Option<(String, String, Option<u8>)> {
    let mut parts = input.split_whitespace();
    let url = parts.next()?.to_string();
    let name = parts.next()?.to_string();
    let priority = parts.next().and_then(|p| p.parse::<u8>().ok());
    Some((url, name, priority))
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
    OpenSettings,
    SetLayout(DashboardLayout),
    FocusPane(Pane),
    SelectFeature(usize),
    SelectExt(usize),
    SelectPreset(usize),
    SelectIntegration(usize),
    SelectWorkflow(usize),
    SelectCatalogSource(usize),
    SetCatalogTab(CatalogTarget),
    SetCatalogAddCursor(usize),
    SetExtTab(ExtTab),
    SetSpecTab(SpecTab),
    SettingsSelect(usize),
    SettingsChip(SettingsRow, usize),
    SettingsEdit(usize),
    LayoutEditorSelect(usize),
    PaletteRun(usize),
    JumpToFeature(usize),
}

pub fn palette_commands() -> Vec<PaletteCommand> {
    vec![
        PaletteCommand {
            label: "Go to Dashboard",
            hint: "d",
            action: PaletteAction::SetScreen(Screen::Dashboard),
        },
        PaletteCommand {
            label: "Go to Spec Browser",
            hint: "s",
            action: PaletteAction::SetScreen(Screen::SpecBrowser),
        },
        PaletteCommand {
            label: "Go to Constitution",
            hint: "C",
            action: PaletteAction::SetScreen(Screen::Constitution),
        },
        PaletteCommand {
            label: "Show Features",
            hint: "f",
            action: PaletteAction::OpenPopup(PopupKind::Features),
        },
        PaletteCommand {
            label: "Manage Integrations",
            hint: "i",
            action: PaletteAction::OpenPopup(PopupKind::Integrations),
        },
        PaletteCommand {
            label: "Manage Extensions",
            hint: "e",
            action: PaletteAction::OpenPopup(PopupKind::Extensions),
        },
        PaletteCommand {
            label: "Manage Presets",
            hint: "p",
            action: PaletteAction::OpenPopup(PopupKind::Presets),
        },
        PaletteCommand {
            label: "Manage Workflows",
            hint: "w",
            action: PaletteAction::OpenPopup(PopupKind::Workflows),
        },
        PaletteCommand {
            label: "Manage Catalogs",
            hint: "c",
            action: PaletteAction::OpenPopup(PopupKind::Catalogs),
        },
        PaletteCommand {
            label: "Open Settings",
            hint: "⚙",
            action: PaletteAction::SetScreen(Screen::Settings),
        },
        PaletteCommand {
            label: "Layout: Overview",
            hint: "1",
            action: PaletteAction::SetLayout(DashboardLayout::Overview),
        },
        PaletteCommand {
            label: "Layout: Coding",
            hint: "2",
            action: PaletteAction::SetLayout(DashboardLayout::Coding),
        },
        PaletteCommand {
            label: "Layout: Audit",
            hint: "3",
            action: PaletteAction::SetLayout(DashboardLayout::Audit),
        },
        PaletteCommand {
            label: "Layout: Custom (edited)",
            hint: "4",
            action: PaletteAction::SetLayout(DashboardLayout::Custom),
        },
        PaletteCommand {
            label: "Attach agent session",
            hint: "a",
            action: PaletteAction::SetScreen(Screen::SessionAttach),
        },
        PaletteCommand {
            label: "Toggle theme (dark / light)",
            hint: "t",
            action: PaletteAction::ToggleTheme,
        },
        PaletteCommand {
            label: "Cycle accent palette",
            hint: "T",
            action: PaletteAction::CycleAccent,
        },
        PaletteCommand {
            label: "Help",
            hint: "?",
            action: PaletteAction::OpenPopup(PopupKind::Help),
        },
        PaletteCommand {
            label: "Quit",
            hint: "q",
            action: PaletteAction::Quit,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use spectatui_core::speckit::cli::CliAction;
    use spectatui_core::speckit::Project;

    fn test_app() -> App {
        let project = Project {
            root: std::path::PathBuf::from("."),
            constitution: None,
            features: Vec::new(),
            extensions: Vec::new(),
            presets: Vec::new(),
            integrations: Vec::new(),
            workflows: Vec::new(),
        };
        App::new(project, AppConfig::default())
    }

    #[test]
    fn can_start_when_no_job() {
        let app = test_app();
        assert!(app.can_start_cli_action());
    }

    #[test]
    fn cannot_start_when_job_pending_or_running() {
        let mut app = test_app();
        // Pending: freshly spawned, before the first output line (or completion) has
        // been polled — must already block, not just once status flips to Running.
        app.cli_job = Some(CliJob::new(CliAction::SelfCheck));
        assert!(!app.can_start_cli_action());

        let mut job = CliJob::new(CliAction::SelfCheck);
        job.status = JobStatus::Running;
        app.cli_job = Some(job);
        assert!(!app.can_start_cli_action());
    }

    #[test]
    fn can_start_again_after_job_succeeded_or_failed() {
        let mut app = test_app();
        let mut job = CliJob::new(CliAction::SelfCheck);
        job.status = JobStatus::Succeeded;
        app.cli_job = Some(job);
        assert!(app.can_start_cli_action());

        let mut job = CliJob::new(CliAction::SelfCheck);
        job.status = JobStatus::Failed;
        app.cli_job = Some(job);
        assert!(app.can_start_cli_action());
    }

    fn test_source(name: &str) -> CatalogSource {
        CatalogSource {
            name: name.to_string(),
            url: format!("https://example.com/{name}.json"),
            priority: None,
            install_allowed: true,
        }
    }

    #[test]
    fn cat_select_next_prev_wrap_around() {
        let mut app = test_app();
        app.set_catalog_sources(
            CatalogTarget::Extension,
            vec![test_source("a"), test_source("b"), test_source("c")],
        );
        assert_eq!(app.cat_index, 0);
        app.cat_select_next();
        assert_eq!(app.cat_index, 1);
        app.cat_select_next();
        assert_eq!(app.cat_index, 2);
        app.cat_select_next();
        assert_eq!(app.cat_index, 0, "wraps past the end");
        app.cat_select_prev();
        assert_eq!(app.cat_index, 2, "wraps before the start");
    }

    #[test]
    fn cat_select_next_prev_noop_on_empty_list() {
        let mut app = test_app();
        assert!(app.current_catalog_list().is_empty());
        app.cat_select_next();
        assert_eq!(app.cat_index, 0);
        app.cat_select_prev();
        assert_eq!(app.cat_index, 0);
    }

    #[test]
    fn parse_catalog_add_input_with_priority() {
        let (url, name, priority) =
            parse_catalog_add_input("https://example.com/cat.json community 5").unwrap();
        assert_eq!(url, "https://example.com/cat.json");
        assert_eq!(name, "community");
        assert_eq!(priority, Some(5));
    }

    #[test]
    fn parse_catalog_add_input_without_priority() {
        let (url, name, priority) =
            parse_catalog_add_input("https://example.com/cat.json community").unwrap();
        assert_eq!(url, "https://example.com/cat.json");
        assert_eq!(name, "community");
        assert_eq!(priority, None);
    }

    #[test]
    fn parse_catalog_add_input_rejects_missing_name() {
        assert!(parse_catalog_add_input("https://example.com/cat.json").is_none());
        assert!(parse_catalog_add_input("").is_none());
        assert!(parse_catalog_add_input("   ").is_none());
    }

    #[test]
    fn cat_add_insert_str_appends_and_advances_cursor() {
        let mut app = test_app();
        app.cat_add_input = Some(String::new());
        app.cat_add_insert_str("abc");
        assert_eq!(app.cat_add_input.as_deref(), Some("abc"));
        assert_eq!(app.cat_add_cursor, 3);
    }

    #[test]
    fn cat_add_insert_str_inserts_at_cursor_not_just_at_end() {
        let mut app = test_app();
        app.cat_add_input = Some("ac".to_string());
        app.cat_add_cursor = 1;
        app.cat_add_insert_str("b");
        assert_eq!(app.cat_add_input.as_deref(), Some("abc"));
        assert_eq!(app.cat_add_cursor, 2);
    }

    #[test]
    fn cat_add_insert_str_strips_newlines() {
        let mut app = test_app();
        app.cat_add_input = Some(String::new());
        app.cat_add_insert_str("a\nb\r\nc");
        assert_eq!(app.cat_add_input.as_deref(), Some("abc"));
        assert_eq!(app.cat_add_cursor, 3);
    }

    #[test]
    fn cat_add_backspace_removes_char_before_cursor() {
        let mut app = test_app();
        app.cat_add_input = Some("abc".to_string());
        app.cat_add_cursor = 2; // between 'b' and 'c'
        app.cat_add_backspace();
        assert_eq!(app.cat_add_input.as_deref(), Some("ac"));
        assert_eq!(app.cat_add_cursor, 1);
    }

    #[test]
    fn cat_add_backspace_noop_at_start() {
        let mut app = test_app();
        app.cat_add_input = Some("abc".to_string());
        app.cat_add_cursor = 0;
        app.cat_add_backspace();
        assert_eq!(app.cat_add_input.as_deref(), Some("abc"));
        assert_eq!(app.cat_add_cursor, 0);
    }

    #[test]
    fn cat_add_delete_forward_removes_char_at_cursor() {
        let mut app = test_app();
        app.cat_add_input = Some("abc".to_string());
        app.cat_add_cursor = 1; // before 'b'
        app.cat_add_delete_forward();
        assert_eq!(app.cat_add_input.as_deref(), Some("ac"));
        assert_eq!(
            app.cat_add_cursor, 1,
            "cursor doesn't move on forward-delete"
        );
    }

    #[test]
    fn cat_add_delete_forward_noop_at_end() {
        let mut app = test_app();
        app.cat_add_input = Some("abc".to_string());
        app.cat_add_cursor = 3;
        app.cat_add_delete_forward();
        assert_eq!(app.cat_add_input.as_deref(), Some("abc"));
    }

    #[test]
    fn cat_add_move_left_right_clamp_at_bounds() {
        let mut app = test_app();
        app.cat_add_input = Some("abc".to_string());
        app.cat_add_cursor = 0;
        app.cat_add_move_left();
        assert_eq!(app.cat_add_cursor, 0, "clamped at start");
        app.cat_add_move_right();
        app.cat_add_move_right();
        app.cat_add_move_right();
        app.cat_add_move_right();
        assert_eq!(app.cat_add_cursor, 3, "clamped at end");
    }

    #[test]
    fn cat_add_move_home_end() {
        let mut app = test_app();
        app.cat_add_input = Some("abc".to_string());
        app.cat_add_cursor = 1;
        app.cat_add_move_home();
        assert_eq!(app.cat_add_cursor, 0);
        app.cat_add_move_end();
        assert_eq!(app.cat_add_cursor, 3);
    }

    #[test]
    fn cat_add_set_cursor_clamps_to_char_count() {
        let mut app = test_app();
        app.cat_add_input = Some("abc".to_string());
        app.cat_add_set_cursor(2);
        assert_eq!(app.cat_add_cursor, 2);
        app.cat_add_set_cursor(999);
        assert_eq!(app.cat_add_cursor, 3, "clamped to char count");
    }

    #[test]
    fn cat_add_clear_empties_input_and_resets_cursor_but_stays_open() {
        let mut app = test_app();
        app.cat_add_input = Some("https://example.com/cat.json community".to_string());
        app.cat_add_cursor = 10;
        app.cat_add_clear();
        assert_eq!(app.cat_add_input.as_deref(), Some(""));
        assert_eq!(app.cat_add_cursor, 0);
        assert!(
            app.cat_add_input.is_some(),
            "form stays open, unlike Esc which cancels it"
        );
    }

    #[test]
    fn cat_add_open_prefills_from_selected_discover_only_source_without_priority() {
        let mut app = test_app();
        let mut discover_only = test_source("b");
        discover_only.install_allowed = false;
        app.set_catalog_sources(
            CatalogTarget::Extension,
            vec![test_source("a"), discover_only],
        );
        app.cat_index = 1;
        app.cat_add_open();
        assert_eq!(
            app.cat_add_input.as_deref(),
            Some("https://example.com/b.json b")
        );
        assert_eq!(
            app.cat_add_cursor,
            "https://example.com/b.json b".chars().count()
        );
    }

    #[test]
    fn cat_add_open_prefills_priority_when_present() {
        let mut app = test_app();
        let mut source = test_source("community");
        source.install_allowed = false;
        source.priority = Some(5);
        app.set_catalog_sources(CatalogTarget::Extension, vec![source]);
        app.cat_add_open();
        assert_eq!(
            app.cat_add_input.as_deref(),
            Some("https://example.com/community.json community 5")
        );
    }

    #[test]
    fn cat_add_open_is_empty_when_selected_source_is_already_installed() {
        let mut app = test_app();
        let installed = test_source("a"); // test_source defaults install_allowed to true
        assert!(installed.install_allowed);
        app.set_catalog_sources(CatalogTarget::Extension, vec![installed]);
        app.cat_add_open();
        assert_eq!(
            app.cat_add_input.as_deref(),
            Some(""),
            "installed sources aren't prefilled — only discovery-only ones"
        );
        assert_eq!(app.cat_add_cursor, 0);
    }

    #[test]
    fn cat_add_open_is_empty_when_nothing_selected() {
        let mut app = test_app();
        assert!(app.current_catalog_list().is_empty());
        app.cat_add_open();
        assert_eq!(app.cat_add_input.as_deref(), Some(""));
        assert_eq!(app.cat_add_cursor, 0);
    }

    #[test]
    fn cat_add_insert_str_is_utf8_boundary_safe() {
        let mut app = test_app();
        app.cat_add_input = Some("héllo".to_string()); // 'é' is a multi-byte char
        app.cat_add_cursor = 2; // between 'é' and 'l' — a char boundary, not a byte one
        app.cat_add_insert_str("X");
        assert_eq!(app.cat_add_input.as_deref(), Some("héXllo"));
        assert_eq!(app.cat_add_cursor, 3);
    }

    #[test]
    fn set_catalog_sources_clamps_selection_after_shrink() {
        let mut app = test_app();
        app.set_catalog_sources(
            CatalogTarget::Extension,
            vec![test_source("a"), test_source("b"), test_source("c")],
        );
        app.cat_index = 2;
        // Simulate a successful removal (or refresh) that shrinks the list to 1 item.
        app.set_catalog_sources(CatalogTarget::Extension, vec![test_source("a")]);
        assert_eq!(app.cat_index, 0);

        // Shrinking to empty clamps to 0, not underflow.
        app.set_catalog_sources(CatalogTarget::Extension, Vec::new());
        assert_eq!(app.cat_index, 0);
    }

    #[test]
    fn set_catalog_sources_does_not_clamp_inactive_tab() {
        let mut app = test_app();
        app.set_catalog_sources(
            CatalogTarget::Workflow,
            vec![test_source("a"), test_source("b")],
        );
        app.cat_tab = CatalogTarget::Workflow;
        app.cat_index = 1;
        // Updating a different, inactive tab must not touch the active tab's index.
        app.set_catalog_sources(CatalogTarget::Extension, vec![test_source("x")]);
        assert_eq!(app.cat_index, 1);
    }

    #[test]
    fn open_popup_catalogs_preserves_last_viewed_tab() {
        let mut app = test_app();
        app.cat_tab = CatalogTarget::Workflow;
        app.open_popup(PopupKind::Catalogs);
        assert_eq!(
            app.cat_tab,
            CatalogTarget::Workflow,
            "status-bar/keypress entry is sticky"
        );
    }

    #[test]
    fn open_catalogs_reset_to_extensions_always_resets() {
        let mut app = test_app();
        app.cat_tab = CatalogTarget::Workflow;
        app.open_catalogs_reset_to_extensions();
        assert_eq!(
            app.cat_tab,
            CatalogTarget::Extension,
            "palette entry always resets"
        );
    }

    #[test]
    fn current_catalog_list_is_scoped_to_active_tab() {
        let mut app = test_app();
        app.set_catalog_sources(CatalogTarget::Extension, vec![test_source("ext-a")]);
        app.set_catalog_sources(
            CatalogTarget::Workflow,
            vec![test_source("wf-a"), test_source("wf-b")],
        );
        assert_eq!(app.current_catalog_list().len(), 1);
        app.cat_tab = CatalogTarget::Workflow;
        assert_eq!(app.current_catalog_list().len(), 2);
    }
}
