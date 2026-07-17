mod app;
mod config;
mod event;
mod theme;
mod ui;

use std::io;
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use crossterm::execute;
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use tokio::sync::mpsc;

use spectatui_core::speckit::cli::{CliAction, CliTarget, SpecifyCliClient};
use spectatui_core::speckit::registry;
use spectatui_core::speckit::watch::{self, FsEvent};
use spectatui_core::speckit::Project;
use spectatui_core::tmux::TmuxClient;

use app::{
    palette_commands, App, ClickAction, DashboardLayout, ExtTab, PaletteAction, Pane, PopupKind,
    Screen, SettingsRow,
};
use event::{AppEvent, EventStream};

#[derive(Parser)]
#[command(name = "spectatui", about = "TUI dashboard for GitHub Spec-Kit")]
struct Cli {
    /// Path to the Spec-Kit project root
    #[arg(long, short, default_value = ".")]
    project: PathBuf,

    /// Theme: dark or light
    #[arg(long)]
    theme: Option<String>,

    /// Accent: indigo, teal, or amber
    #[arg(long)]
    accent: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let root = cli
        .project
        .canonicalize()
        .unwrap_or_else(|_| cli.project.clone());

    let project = Project::discover(&root).context("failed to discover project")?;

    let mut app_config = config::load_config(Some(&root));
    if let Some(t) = &cli.theme {
        app_config.theme = t.clone();
    }
    if let Some(a) = &cli.accent {
        app_config.accent = a.clone();
    }

    let mut app = App::new(project, app_config);

    // Check tmux availability
    app.tmux_available = TmuxClient::has_tmux().await;

    // Set up terminal
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    if app.config.mouse_support {
        execute!(stdout, crossterm::event::EnableMouseCapture)?;
    }
    execute!(stdout, crossterm::event::EnableBracketedPaste)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_loop(&mut terminal, &mut app, &root).await;

    // Cleanup
    if app.config.mouse_support {
        execute!(
            terminal.backend_mut(),
            crossterm::event::DisableMouseCapture
        )?;
    }
    execute!(
        terminal.backend_mut(),
        crossterm::event::DisableBracketedPaste
    )?;
    terminal::disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

async fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    root: &std::path::Path,
) -> Result<()> {
    let (mut events, event_tx) = EventStream::new(std::time::Duration::from_millis(100));

    // Start file watcher — bridge FsEvent to AppEvent
    let (fs_send, mut fs_recv) = mpsc::unbounded_channel();
    let event_tx_clone = event_tx.clone();
    tokio::spawn(async move {
        while let Some(ev) = fs_recv.recv().await {
            let _ = event_tx_clone.send(AppEvent::FsChanged(ev));
        }
    });
    let _watcher = watch::start_watcher(root, fs_send).ok();

    // CLI client
    let cli_client = SpecifyCliClient::new(root.to_path_buf());

    // Selected-feature channel: the tmux poller reads the latest selection so it
    // can capture the right pane without owning app state.
    let (selection_tx, selection_rx) =
        tokio::sync::watch::channel(app.selected_feature().map(|f| f.id.clone()));

    // tmux poller — emits TmuxChanged off the Tick path.
    if app.tmux_available {
        let tmux_tx = event_tx.clone();
        let selection_rx = selection_rx.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_millis(750));
            loop {
                interval.tick().await;
                let sessions = TmuxClient::list_sessions().await.unwrap_or_default();
                let selected = selection_rx.borrow().clone();
                let session = match selected {
                    Some(id) => {
                        if let Some(mut s) = TmuxClient::find_session(&id).await {
                            if let Ok(lines) = TmuxClient::capture_pane(&s.name, 50).await {
                                s.last_snapshot = lines;
                            }
                            Some(s)
                        } else {
                            None
                        }
                    }
                    None => None,
                };
                if tmux_tx
                    .send(AppEvent::TmuxChanged { sessions, session })
                    .is_err()
                {
                    return;
                }
            }
        });
    }

    // Async catalog indexing
    spawn_catalog_index(event_tx.clone(), root.to_path_buf());

    // Catalog *source* listing — one fetch per resource kind, independent of the
    // "available items" indexing above (see AppEvent::CatalogSourcesLoaded).
    for target in [
        registry::CatalogTarget::Extension,
        registry::CatalogTarget::Preset,
        registry::CatalogTarget::Integration,
        registry::CatalogTarget::Workflow,
    ] {
        spawn_catalog_sources_fetch(event_tx.clone(), root.to_path_buf(), target);
    }

    loop {
        terminal.draw(|frame| ui::draw(frame, app))?;

        if app.should_quit {
            return Ok(());
        }

        // Poll CLI job progress. A successful job reports what refresh_project()
        // didn't already cover: which catalog-source list to re-fetch (CatalogAdd/
        // CatalogRemove), whether the async "available items" index is stale, and/or
        // a queued followup action (the re-add half of a catalog source edit).
        let refresh = app.poll_cli_job();
        if let Some(target) = refresh.catalog_sources_target {
            spawn_catalog_sources_fetch(event_tx.clone(), root.to_path_buf(), target);
        }
        if refresh.needs_reindex {
            spawn_catalog_index(event_tx.clone(), root.to_path_buf());
        }
        if let Some(action) = refresh.followup_action {
            // Preserve the just-finished step's command + output ahead of the
            // followup's own, so a catalog-source edit's two real CLI calls both
            // stay visible in the output log instead of the first being silently
            // overwritten by the second. The popup already prints the *new* job's
            // own `command_line` as a header, so only the prior step's needs
            // inlining into the output body here.
            let prior = app
                .cli_job
                .as_ref()
                .map(|j| format!("$ {}\n{}", j.command_line, j.output));
            spawn_and_show_cli_job(app, &cli_client, &action);
            if let (Some(prior), Some(job)) = (prior, &mut app.cli_job) {
                job.output = format!("{prior}\n\n{}", job.output);
            }
        }

        // Publish the current selection for the tmux poller.
        let _ = selection_tx.send(app.selected_feature().map(|f| f.id.clone()));

        if let Some(event) = events.next().await {
            match event {
                AppEvent::Key(key) => {
                    let mouse_before = app.config.mouse_support;
                    handle_key(app, key, &cli_client);
                    if let Some(target) = app.catalog_refresh_request.take() {
                        spawn_catalog_sources_fetch(event_tx.clone(), root.to_path_buf(), target);
                    }
                    // Apply a Mouse-support setting change immediately (no restart needed).
                    if app.config.mouse_support != mouse_before {
                        if app.config.mouse_support {
                            let _ = execute!(
                                terminal.backend_mut(),
                                crossterm::event::EnableMouseCapture
                            );
                        } else {
                            let _ = execute!(
                                terminal.backend_mut(),
                                crossterm::event::DisableMouseCapture
                            );
                        }
                    }
                    if app.attach_request {
                        app.attach_request = false;
                        attach_session(terminal, app).await?;
                    }
                    if app.launch_request {
                        app.launch_request = false;
                        launch_session(terminal, app).await?;
                    }
                }
                AppEvent::Mouse(mouse) => {
                    handle_mouse(app, mouse, &cli_client);
                }
                AppEvent::FsChanged(fs_event) => match fs_event {
                    FsEvent::SpecsChanged | FsEvent::SpecifyChanged => {
                        app.refresh_project();
                    }
                },
                AppEvent::Resize(_, _) => {
                    // ratatui handles resize automatically
                }
                AppEvent::CatalogIndexed {
                    cli_available,
                    integrations,
                    extensions,
                    presets,
                    workflows,
                } => {
                    app.cli_available = cli_available;
                    app.merge_catalog_results(integrations, extensions, presets, workflows);
                }
                AppEvent::CatalogSourcesLoaded { target, sources } => {
                    app.set_catalog_sources(target, sources);
                }
                AppEvent::Paste(text) => {
                    // Scoped to the catalog add-form only; paste elsewhere is a no-op.
                    if app.cat_add_input.is_some() {
                        app.cat_add_insert_str(&text);
                    }
                }
                AppEvent::TmuxChanged { sessions, session } => {
                    app.apply_tmux(sessions, session);
                }
                AppEvent::Tick => {
                    app.indexing_tick = app.indexing_tick.wrapping_add(1);
                }
            }
        }
    }
}

/// Spawn the "available items" catalog index (installed+available integrations/
/// extensions/presets/workflows), sending the result as `AppEvent::CatalogIndexed`.
/// Used for the initial load at startup and re-run after any CLI job completes that
/// can change installed/available state (see `App::poll_cli_job`'s `needs_reindex`),
/// since `Project::discover()` can't populate this data itself (workflows and
/// "available" items require async CLI/network calls) and otherwise `refresh_project()`
/// would just keep re-applying the stale snapshot from the last index.
fn spawn_catalog_index(tx: mpsc::UnboundedSender<AppEvent>, root: PathBuf) {
    tokio::spawn(async move {
        let cli_available = registry::specify_cli_available(&root).await;
        let (integrations, extensions, presets, workflows) = tokio::join!(
            registry::fetch_available_integrations(&root),
            registry::fetch_available_extensions(&root),
            registry::fetch_available_presets(&root),
            registry::fetch_available_workflows(&root),
        );
        let _ = tx.send(AppEvent::CatalogIndexed {
            cli_available,
            integrations,
            extensions,
            presets,
            workflows,
        });
    });
}

/// Spawn a `list_catalog_sources` fetch for `target`, sending the result as
/// `AppEvent::CatalogSourcesLoaded` — used for the initial load, manual refresh
/// (`r` key), and the post-success refresh after a `CatalogAdd`/`CatalogRemove`.
fn spawn_catalog_sources_fetch(
    tx: mpsc::UnboundedSender<AppEvent>,
    root: PathBuf,
    target: registry::CatalogTarget,
) {
    tokio::spawn(async move {
        let sources = registry::list_catalog_sources(&root, target).await;
        let _ = tx.send(AppEvent::CatalogSourcesLoaded { target, sources });
    });
}

/// Suspend the TUI, run a foreground tmux attach to `target`, then restore
/// the alternate screen / raw mode when the tmux process exits (on detach).
async fn attach_to(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    target: &str,
    mouse_support: bool,
) -> Result<()> {
    // Leave the TUI.
    if mouse_support {
        execute!(
            terminal.backend_mut(),
            crossterm::event::DisableMouseCapture
        )?;
    }
    terminal::disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    let attach_result = TmuxClient::attach(target).await;

    // Restore the TUI.
    terminal::enable_raw_mode()?;
    execute!(terminal.backend_mut(), EnterAlternateScreen)?;
    if mouse_support {
        execute!(terminal.backend_mut(), crossterm::event::EnableMouseCapture)?;
    }
    terminal.clear()?;

    attach_result
}

/// Attach to the currently selected feature's live tmux session, if any.
async fn attach_session(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> Result<()> {
    let Some(target) = app.attach_target() else {
        return Ok(());
    };
    if !app.tmux_available {
        return Ok(());
    }
    attach_to(terminal, &target, app.config.mouse_support).await
}

/// Create a tmux session running the default coding agent for the selected
/// feature, then attach to it. No-ops if tmux is unavailable, no feature is
/// selected, or no default coding agent is configured.
async fn launch_session(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> Result<()> {
    if !app.tmux_available {
        return Ok(());
    }
    let Some(feature_id) = app.selected_feature().map(|f| f.id.clone()) else {
        return Ok(());
    };
    let Some(agent_cmd) = app.default_agent_key() else {
        return Ok(());
    };
    let session_name = app.session_name_for(&feature_id);
    let cwd = app.project.root.clone();
    if TmuxClient::launch_session(&session_name, &cwd, &agent_cmd)
        .await
        .is_err()
    {
        return Ok(());
    }
    attach_to(terminal, &session_name, app.config.mouse_support).await
}

fn handle_key(app: &mut App, key: KeyEvent, cli_client: &SpecifyCliClient) {
    // Ctrl+C always quits — except inside the Catalog Manager's add-source
    // input, where it clears the field instead (more useful there than an
    // accidental full-app quit while editing).
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
        if app.active_popup == Some(PopupKind::Catalogs) && app.cat_add_input.is_some() {
            app.cat_add_clear();
        } else {
            app.should_quit = true;
        }
        return;
    }

    // Command palette input handling
    if let Some(palette) = &mut app.palette {
        match key.code {
            KeyCode::Esc => {
                app.palette = None;
            }
            KeyCode::Enter => {
                let commands = palette_commands();
                let filtered: Vec<_> = commands
                    .into_iter()
                    .filter(|c| {
                        palette.input.is_empty()
                            || c.label
                                .to_lowercase()
                                .contains(&palette.input.to_lowercase())
                    })
                    .collect();

                if let Some(cmd) = filtered.into_iter().nth(palette.selected) {
                    app.palette = None;
                    execute_palette_action(app, cmd.action);
                }
            }
            KeyCode::Up => {
                if palette.selected > 0 {
                    palette.selected -= 1;
                }
            }
            KeyCode::Down => {
                let count = palette_commands()
                    .iter()
                    .filter(|c| {
                        palette.input.is_empty()
                            || c.label
                                .to_lowercase()
                                .contains(&palette.input.to_lowercase())
                    })
                    .count();
                if palette.selected + 1 < count {
                    palette.selected += 1;
                }
            }
            KeyCode::Char(c) => {
                palette.input.push(c);
                palette.selected = 0;
            }
            KeyCode::Backspace => {
                palette.input.pop();
                palette.selected = 0;
            }
            _ => {}
        }
        return;
    }

    // Popup key handling
    if let Some(popup) = app.active_popup {
        match popup {
            PopupKind::QuitConfirm => match key.code {
                KeyCode::Char('q') | KeyCode::Enter => app.should_quit = true,
                KeyCode::Esc => app.close_popup(),
                _ => {}
            },
            PopupKind::CliConfirm => match key.code {
                KeyCode::Enter => {
                    if let Some(action) = app.pending_action.take() {
                        spawn_and_show_cli_job(app, cli_client, &action);
                    }
                }
                KeyCode::Char('f') => {
                    app.force_flag = !app.force_flag;
                    // Only Remove carries a --force flag in the command; sync it so the
                    // previewed command line and the spawned command both reflect the toggle.
                    if let Some(CliAction::Remove { force, .. }) = &mut app.pending_action {
                        *force = app.force_flag;
                    }
                }
                KeyCode::Esc => {
                    app.pending_action = None;
                    app.close_popup();
                }
                _ => {}
            },
            PopupKind::CliOutput => match key.code {
                KeyCode::Up | KeyCode::Char('k') => app.cli_scroll_up(),
                KeyCode::Down | KeyCode::Char('j') => app.cli_scroll_down(),
                KeyCode::Esc => {
                    app.close_popup();
                }
                _ => {}
            },
            PopupKind::Integrations => match handle_filter_key(app, key) {
                FilterKey::ConsumedReset => app.integration_index = 0,
                FilterKey::Consumed => {}
                FilterKey::NotConsumed => {
                    let sel = app
                        .filtered_integrations()
                        .get(app.integration_index)
                        .map(|it| (it.key.clone(), it.installed, it.is_default));
                    let confirm = |app: &mut App, action: CliAction| {
                        request_cli_action(app, action, cli_client)
                    };
                    match key.code {
                        KeyCode::Esc => app.close_popup(),
                        KeyCode::Up | KeyCode::Char('k') => app.integration_select_prev(),
                        KeyCode::Down | KeyCode::Char('j') => app.integration_select_next(),
                        KeyCode::Char('i') => {
                            if let Some((k, installed, _)) = &sel {
                                if !installed {
                                    confirm(app, CliAction::IntegrationInstall { key: k.clone() });
                                }
                            }
                        }
                        KeyCode::Char('s') => {
                            if let Some((k, installed, is_default)) = &sel {
                                if !installed || !is_default {
                                    confirm(app, CliAction::IntegrationSwitch { key: k.clone() });
                                }
                            }
                        }
                        KeyCode::Char('d') => {
                            if let Some((k, installed, is_default)) = &sel {
                                if *installed && !is_default {
                                    confirm(
                                        app,
                                        CliAction::IntegrationUseDefault { key: k.clone() },
                                    );
                                }
                            }
                        }
                        KeyCode::Char('x') => {
                            if let Some((k, installed, _)) = &sel {
                                if *installed {
                                    confirm(
                                        app,
                                        CliAction::IntegrationUninstall { key: k.clone() },
                                    );
                                }
                            }
                        }
                        KeyCode::Char('g') => {
                            if let Some((k, installed, _)) = &sel {
                                if *installed {
                                    confirm(
                                        app,
                                        CliAction::IntegrationUpgrade {
                                            key: Some(k.clone()),
                                        },
                                    );
                                }
                            }
                        }
                        KeyCode::Char('v') => {
                            if let Some((k, installed, _)) = &sel {
                                if *installed {
                                    spawn_and_show_cli_job(
                                        app,
                                        cli_client,
                                        &CliAction::IntegrationStatus { key: k.clone() },
                                    );
                                }
                            }
                        }
                        KeyCode::Char('n') => {
                            if let Some((k, _, _)) = &sel {
                                spawn_and_show_cli_job(
                                    app,
                                    cli_client,
                                    &CliAction::IntegrationGetInfo { key: k.clone() },
                                );
                            }
                        }
                        _ => {}
                    }
                }
            },
            PopupKind::Workflows => match handle_filter_key(app, key) {
                FilterKey::ConsumedReset => app.wf_index = 0,
                FilterKey::Consumed => {}
                FilterKey::NotConsumed => {
                    let sel = app
                        .filtered_workflows()
                        .get(app.wf_index)
                        .map(|wf| (wf.id.clone(), wf.installed, wf.last_run.clone()));
                    let confirm = |app: &mut App, action: CliAction| {
                        request_cli_action(app, action, cli_client)
                    };
                    match key.code {
                        KeyCode::Esc => app.close_popup(),
                        KeyCode::Up | KeyCode::Char('k') => app.wf_select_prev(),
                        KeyCode::Down | KeyCode::Char('j') => app.wf_select_next(),
                        KeyCode::Char('a') => {
                            if let Some((id, installed, _)) = &sel {
                                if !installed {
                                    confirm(app, CliAction::WorkflowAdd { source: id.clone() });
                                }
                            }
                        }
                        KeyCode::Char('x') => {
                            if let Some((id, installed, _)) = &sel {
                                if *installed {
                                    confirm(app, CliAction::WorkflowRemove { id: id.clone() });
                                }
                            }
                        }
                        KeyCode::Char('r') => {
                            if let Some((id, installed, _)) = &sel {
                                if *installed {
                                    confirm(app, CliAction::WorkflowRun { source: id.clone() });
                                }
                            }
                        }
                        KeyCode::Char('R') => {
                            if let Some((_, _, Some(run_id))) = &sel {
                                confirm(
                                    app,
                                    CliAction::WorkflowResume {
                                        run_id: run_id.clone(),
                                    },
                                );
                            }
                        }
                        KeyCode::Char('s') => {
                            if sel.is_some() {
                                confirm(app, CliAction::WorkflowStatus { run_id: None });
                            }
                        }
                        KeyCode::Char('i') => {
                            if let Some((id, _, _)) = &sel {
                                spawn_and_show_cli_job(
                                    app,
                                    cli_client,
                                    &CliAction::WorkflowGetInfo { id: id.clone() },
                                );
                            }
                        }
                        _ => {}
                    }
                }
            },
            PopupKind::Features => match key.code {
                KeyCode::Esc => app.close_popup(),
                KeyCode::Up | KeyCode::Char('k') => app.select_prev_feature(),
                KeyCode::Down | KeyCode::Char('j') => app.select_next_feature(),
                KeyCode::Enter => {
                    app.close_popup();
                    app.screen = Screen::Dashboard;
                }
                _ => {}
            },
            PopupKind::Extensions | PopupKind::Presets => {
                handle_ext_preset_popup_key(app, key, cli_client);
            }
            // Remove/tab-switch/refresh handling is layered on in later tasks (US2/US3).
            PopupKind::Catalogs => {
                if app.cat_add_input.is_some() {
                    match key.code {
                        KeyCode::Esc => {
                            app.cat_add_input = None;
                            app.cat_add_cursor = 0;
                            app.cat_edit_target = None;
                        }
                        KeyCode::Backspace => app.cat_add_backspace(),
                        KeyCode::Delete => app.cat_add_delete_forward(),
                        KeyCode::Left => app.cat_add_move_left(),
                        KeyCode::Right => app.cat_add_move_right(),
                        KeyCode::Home => app.cat_add_move_home(),
                        KeyCode::End => app.cat_add_move_end(),
                        // Only meaningful in edit mode (see `cat_edit_toggle_install_allowed`);
                        // free to repurpose here since Tab otherwise does nothing while the
                        // add-form has focus (it switches catalog kind tabs the rest of the time).
                        KeyCode::Tab => app.cat_edit_toggle_install_allowed(),
                        KeyCode::Enter => {
                            if let Some(input) = app.cat_add_input.take() {
                                app.cat_add_cursor = 0;
                                let edit_target = app.cat_edit_target.take();
                                if let Some((url, name, priority)) =
                                    app::parse_catalog_add_input(&input)
                                {
                                    let add_action = CliAction::CatalogAdd {
                                        target: app.cat_tab,
                                        url,
                                        name,
                                        priority,
                                        install_allowed: edit_target
                                            .is_some()
                                            .then_some(app.cat_edit_install_allowed),
                                    };
                                    if let Some(old_name) = edit_target {
                                        // No single CLI edit/update verb exists (only
                                        // list/add/remove) — remove the original, then
                                        // chain the re-add once that succeeds.
                                        app.pending_followup_action = Some(add_action);
                                        request_cli_action(
                                            app,
                                            CliAction::CatalogRemove {
                                                target: app.cat_tab,
                                                name: old_name,
                                            },
                                            cli_client,
                                        );
                                    } else {
                                        request_cli_action(app, add_action, cli_client);
                                    }
                                }
                            }
                        }
                        KeyCode::Char(c) => {
                            let mut buf = [0u8; 4];
                            app.cat_add_insert_str(c.encode_utf8(&mut buf));
                        }
                        _ => {}
                    }
                } else {
                    match key.code {
                        KeyCode::Esc => app.close_popup(),
                        KeyCode::Up | KeyCode::Char('k') => app.cat_select_prev(),
                        KeyCode::Down | KeyCode::Char('j') => app.cat_select_next(),
                        KeyCode::Tab => {
                            app.cat_tab = app.cat_tab.next();
                            app.cat_index = 0;
                        }
                        KeyCode::BackTab => {
                            app.cat_tab = app.cat_tab.prev();
                            app.cat_index = 0;
                        }
                        KeyCode::Char('a') => {
                            app.cat_add_open();
                        }
                        KeyCode::Char('e') => {
                            if app.cat_edit_available() {
                                app.cat_edit_open();
                            }
                        }
                        KeyCode::Char('x') => {
                            if let Some(name) = app
                                .current_catalog_list()
                                .get(app.cat_index)
                                .map(|s| s.name.clone())
                            {
                                request_cli_action(
                                    app,
                                    CliAction::CatalogRemove {
                                        target: app.cat_tab,
                                        name,
                                    },
                                    cli_client,
                                );
                            }
                        }
                        KeyCode::Char('r') => {
                            app.catalog_refresh_request = Some(app.cat_tab);
                        }
                        _ => {}
                    }
                }
            }
            _ => {
                if key.code == KeyCode::Esc {
                    app.close_popup()
                }
            }
        }
        return;
    }

    // Layout editor key handling
    if app.layout_editor_active {
        match key.code {
            KeyCode::Esc => {
                app.layout_editor_active = false;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if app.layout_editor_index > 0 {
                    app.layout_editor_index -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let max = app.custom_layout.panes.len().saturating_sub(1);
                if app.layout_editor_index < max {
                    app.layout_editor_index += 1;
                }
            }
            KeyCode::Char(' ') => {
                let sorted: Vec<usize> = {
                    let mut pairs: Vec<(usize, u8)> = app
                        .custom_layout
                        .panes
                        .iter()
                        .enumerate()
                        .map(|(i, p)| (i, p.order))
                        .collect();
                    pairs.sort_by_key(|(_, o)| *o);
                    pairs.into_iter().map(|(i, _)| i).collect()
                };
                if let Some(&real_idx) = sorted.get(app.layout_editor_index) {
                    let kind = app.custom_layout.panes[real_idx].kind;
                    app.custom_layout.toggle_visibility(kind);
                }
            }
            KeyCode::Char('<') => {
                let sorted: Vec<usize> = {
                    let mut pairs: Vec<(usize, u8)> = app
                        .custom_layout
                        .panes
                        .iter()
                        .enumerate()
                        .map(|(i, p)| (i, p.order))
                        .collect();
                    pairs.sort_by_key(|(_, o)| *o);
                    pairs.into_iter().map(|(i, _)| i).collect()
                };
                if let Some(&real_idx) = sorted.get(app.layout_editor_index) {
                    app.custom_layout.swap_order(real_idx, -1);
                }
            }
            KeyCode::Char('>') => {
                let sorted: Vec<usize> = {
                    let mut pairs: Vec<(usize, u8)> = app
                        .custom_layout
                        .panes
                        .iter()
                        .enumerate()
                        .map(|(i, p)| (i, p.order))
                        .collect();
                    pairs.sort_by_key(|(_, o)| *o);
                    pairs.into_iter().map(|(i, _)| i).collect()
                };
                if let Some(&real_idx) = sorted.get(app.layout_editor_index) {
                    app.custom_layout.swap_order(real_idx, 1);
                }
            }
            KeyCode::Char('+') | KeyCode::Char('=') => {
                let sorted: Vec<usize> = {
                    let mut pairs: Vec<(usize, u8)> = app
                        .custom_layout
                        .panes
                        .iter()
                        .enumerate()
                        .map(|(i, p)| (i, p.order))
                        .collect();
                    pairs.sort_by_key(|(_, o)| *o);
                    pairs.into_iter().map(|(i, _)| i).collect()
                };
                if let Some(&real_idx) = sorted.get(app.layout_editor_index) {
                    let kind = app.custom_layout.panes[real_idx].kind;
                    app.custom_layout.resize_pane(kind, 1);
                }
            }
            KeyCode::Char('-') => {
                let sorted: Vec<usize> = {
                    let mut pairs: Vec<(usize, u8)> = app
                        .custom_layout
                        .panes
                        .iter()
                        .enumerate()
                        .map(|(i, p)| (i, p.order))
                        .collect();
                    pairs.sort_by_key(|(_, o)| *o);
                    pairs.into_iter().map(|(i, _)| i).collect()
                };
                if let Some(&real_idx) = sorted.get(app.layout_editor_index) {
                    let kind = app.custom_layout.panes[real_idx].kind;
                    app.custom_layout.resize_pane(kind, -1);
                }
            }
            KeyCode::Enter => {
                app.config.custom_layout = Some(app.custom_layout.clone());
                let _ = config::save_config(&app.config);
                app.layout_editor_active = false;
                app.layout = DashboardLayout::Custom;
            }
            _ => {}
        }
        return;
    }

    // Global keys (available on all screens)
    match key.code {
        KeyCode::Char('q') => {
            app.open_popup(PopupKind::QuitConfirm);
            return;
        }
        KeyCode::Char('t') => {
            app.toggle_theme();
            return;
        }
        KeyCode::Char('T') => {
            app.cycle_accent();
            return;
        }
        KeyCode::Char(':') => {
            app.open_palette();
            return;
        }
        KeyCode::Char('?') => {
            app.open_popup(PopupKind::Help);
            return;
        }
        KeyCode::Char('i') => {
            app.open_popup(PopupKind::Integrations);
            return;
        }
        KeyCode::Char('f') => {
            app.open_popup(PopupKind::Features);
            return;
        }
        KeyCode::Char('w') => {
            app.open_popup(PopupKind::Workflows);
            return;
        }
        KeyCode::Char('c') => {
            app.open_popup(PopupKind::Catalogs);
            return;
        }
        _ => {}
    }

    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('k') {
        app.open_palette();
        return;
    }

    // Screen-specific keys
    match app.screen {
        Screen::Dashboard => handle_dashboard_key(app, key, cli_client),
        Screen::SpecBrowser => handle_spec_browser_key(app, key),
        Screen::Constitution => handle_constitution_key(app, key),
        Screen::Settings => handle_settings_key(app, key, cli_client),
        Screen::SessionAttach => match key.code {
            KeyCode::Esc => app.go_back(),
            KeyCode::Backspace => {
                app.attach_input.pop();
            }
            KeyCode::Char(c) => {
                app.attach_input.push(c);
            }
            KeyCode::Enter => {
                if app.attach_input.is_empty() {
                    // Empty input: go full-screen and attach to the live pane.
                    app.attach_request = true;
                } else if let Some(target) = app.attach_target() {
                    // Send the typed follow-up to the agent pane.
                    let text = std::mem::take(&mut app.attach_input);
                    tokio::spawn(async move {
                        let _ = TmuxClient::send_keys(&target, &text).await;
                    });
                }
            }
            _ => {}
        },
    }
}

fn handle_dashboard_key(app: &mut App, key: KeyEvent, _cli_client: &SpecifyCliClient) {
    match key.code {
        KeyCode::Tab => app.cycle_tab_forward(),
        KeyCode::BackTab => app.cycle_tab_backward(),
        KeyCode::Up | KeyCode::Char('k') => app.select_prev_feature(),
        KeyCode::Down | KeyCode::Char('j') => app.select_next_feature(),
        KeyCode::Enter if app.focused_pane == Pane::AgentOutput && app.tmux_session.is_none() => {
            app.launch_request = true;
        }
        KeyCode::Enter => app.enter_spec_browser(),
        KeyCode::Char('a') => {
            app.screen = Screen::SessionAttach;
        }
        KeyCode::Char('C') => app.enter_constitution(),
        KeyCode::Char('e') => {
            app.ext_tab = ExtTab::Extensions;
            app.open_popup(PopupKind::Extensions);
        }
        KeyCode::Char('s') => app.enter_settings(),
        KeyCode::Char('p') => {
            app.ext_tab = ExtTab::Presets;
            app.open_popup(PopupKind::Presets);
        }
        KeyCode::Char('1') => app.layout = DashboardLayout::Overview,
        KeyCode::Char('2') => app.layout = DashboardLayout::Coding,
        KeyCode::Char('3') => app.layout = DashboardLayout::Audit,
        KeyCode::Char('4') => app.layout = DashboardLayout::Custom,
        _ => {}
    }
}

fn handle_spec_browser_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Tab => app.cycle_tab_forward(),
        KeyCode::BackTab => app.cycle_tab_backward(),
        KeyCode::Up | KeyCode::Char('k') => app.scroll_up(),
        KeyCode::Down | KeyCode::Char('j') => app.scroll_down(),
        KeyCode::Left => {
            app.select_prev_feature();
            app.spec_scroll = 0;
        }
        KeyCode::Right => {
            app.select_next_feature();
            app.spec_scroll = 0;
        }
        KeyCode::Esc => app.go_back(),
        _ => {}
    }
}

fn handle_constitution_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => app.scroll_up(),
        KeyCode::Down | KeyCode::Char('j') => app.scroll_down(),
        KeyCode::Esc => app.go_back(),
        _ => {}
    }
}

/// Run a CLI action, gated by the `Confirm before --force` setting: when enabled
/// (default) open the confirm dialog first; otherwise spawn the job immediately.
fn request_cli_action(app: &mut App, action: CliAction, cli_client: &SpecifyCliClient) {
    if app.config.confirm_before_force {
        app.pending_action = Some(action);
        app.force_flag = false;
        app.active_popup = Some(PopupKind::CliConfirm);
    } else {
        spawn_and_show_cli_job(app, cli_client, &action);
    }
}

/// Spawn a CLI action and show its output — unless another action is already running
/// (spec FR-019a), in which case the new action is blocked and the in-flight job's
/// output is brought back into view instead of being silently overwritten.
fn spawn_and_show_cli_job(app: &mut App, cli_client: &SpecifyCliClient, action: &CliAction) {
    if !app.can_start_cli_action() {
        app.active_popup = Some(PopupKind::CliOutput);
        return;
    }
    let (job, rx) = cli_client.spawn_job(action);
    app.show_cli_job(job, rx);
}

fn current_ext_id(app: &App) -> Option<String> {
    match app.ext_tab {
        ExtTab::Extensions => app
            .filtered_extensions()
            .get(app.ext_index)
            .map(|e| e.id.clone()),
        ExtTab::Presets => app
            .filtered_presets()
            .get(app.preset_index)
            .map(|p| p.id.clone()),
    }
}

enum FilterKey {
    /// Key consumed; the list query changed so the selection should reset to 0.
    ConsumedReset,
    /// Key consumed by the filter; no further handling.
    Consumed,
    /// Not a filter key; fall through to normal handling.
    NotConsumed,
}

/// Handle the inline list-filter keys (`/` to open, typing to narrow). Mutates
/// `filter_query`/`filter_active`; the caller resets its selection index on
/// `ConsumedReset`.
fn handle_filter_key(app: &mut App, key: KeyEvent) -> FilterKey {
    if app.filter_active {
        match key.code {
            KeyCode::Esc => {
                app.filter_active = false;
                app.filter_query.clear();
                FilterKey::ConsumedReset
            }
            KeyCode::Enter => {
                app.filter_active = false;
                FilterKey::Consumed
            }
            KeyCode::Backspace => {
                app.filter_query.pop();
                FilterKey::ConsumedReset
            }
            // Arrows still move the selection while filtering.
            KeyCode::Up | KeyCode::Down => FilterKey::NotConsumed,
            KeyCode::Char(c) => {
                app.filter_query.push(c);
                FilterKey::ConsumedReset
            }
            _ => FilterKey::Consumed,
        }
    } else if key.code == KeyCode::Char('/') {
        app.filter_active = true;
        FilterKey::ConsumedReset
    } else {
        FilterKey::NotConsumed
    }
}

fn handle_ext_preset_popup_key(app: &mut App, key: KeyEvent, cli_client: &SpecifyCliClient) {
    match handle_filter_key(app, key) {
        FilterKey::ConsumedReset => {
            app.ext_index = 0;
            app.preset_index = 0;
            return;
        }
        FilterKey::Consumed => return,
        FilterKey::NotConsumed => {}
    }
    match key.code {
        KeyCode::Esc => app.close_popup(),
        KeyCode::Tab | KeyCode::BackTab => {
            let next = match app.ext_tab {
                ExtTab::Extensions => PopupKind::Presets,
                ExtTab::Presets => PopupKind::Extensions,
            };
            app.open_popup(next);
        }
        KeyCode::Up | KeyCode::Char('k') => app.ext_select_prev(),
        KeyCode::Down | KeyCode::Char('j') => app.ext_select_next(),
        KeyCode::Char('a') => {
            let target = match app.ext_tab {
                ExtTab::Extensions => CliTarget::Extension,
                ExtTab::Presets => CliTarget::Preset,
            };
            if let Some(id) = current_ext_id(app) {
                let action = CliAction::Add {
                    target,
                    id,
                    priority: None,
                    dev_path: None,
                    from_url: None,
                };
                request_cli_action(app, action, cli_client);
            }
        }
        KeyCode::Char('x') => {
            let target = match app.ext_tab {
                ExtTab::Extensions => CliTarget::Extension,
                ExtTab::Presets => CliTarget::Preset,
            };
            if let Some(id) = current_ext_id(app) {
                let action = CliAction::Remove {
                    target,
                    id,
                    keep_config: false,
                    force: false,
                };
                request_cli_action(app, action, cli_client);
            }
        }
        KeyCode::Char('e') => {
            let target = match app.ext_tab {
                ExtTab::Extensions => CliTarget::Extension,
                ExtTab::Presets => CliTarget::Preset,
            };
            if let Some(id) = current_ext_id(app) {
                let action = CliAction::Enable { target, id };
                request_cli_action(app, action, cli_client);
            }
        }
        KeyCode::Char('d') => {
            let target = match app.ext_tab {
                ExtTab::Extensions => CliTarget::Extension,
                ExtTab::Presets => CliTarget::Preset,
            };
            if let Some(id) = current_ext_id(app) {
                let action = CliAction::Disable { target, id };
                request_cli_action(app, action, cli_client);
            }
        }
        KeyCode::Char('u') if app.ext_tab == ExtTab::Extensions => {
            if let Some(id) = current_ext_id(app) {
                let action = CliAction::Update {
                    target: CliTarget::Extension,
                    id: Some(id),
                };
                request_cli_action(app, action, cli_client);
            }
        }
        KeyCode::Char('p') => {
            if let Some(id) = current_ext_id(app) {
                let target = match app.ext_tab {
                    ExtTab::Extensions => CliTarget::Extension,
                    ExtTab::Presets => CliTarget::Preset,
                };
                let action = CliAction::SetPriority {
                    target,
                    id,
                    priority: 75,
                };
                request_cli_action(app, action, cli_client);
            }
        }
        KeyCode::Char('r') if app.ext_tab == ExtTab::Presets => {
            if let Some(id) = current_ext_id(app) {
                let action = CliAction::Resolve { name: id };
                spawn_and_show_cli_job(app, cli_client, &action);
            }
        }
        _ => {}
    }
}

fn handle_settings_key(app: &mut App, key: KeyEvent, _cli_client: &SpecifyCliClient) {
    if app.settings_editing.is_some() {
        match key.code {
            KeyCode::Enter | KeyCode::Esc => app.settings_end_edit(),
            KeyCode::Backspace => app.settings_edit_backspace(),
            KeyCode::Char(c) => app.settings_edit_push(c),
            _ => {}
        }
        return;
    }
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => app.settings_prev(),
        KeyCode::Down | KeyCode::Char('j') => app.settings_next(),
        KeyCode::Left => app.settings_adjust(-1),
        KeyCode::Right | KeyCode::Enter | KeyCode::Char(' ') => app.settings_primary_action(),
        KeyCode::Esc => app.go_back(),
        _ => {}
    }
}

fn handle_mouse(app: &mut App, mouse: MouseEvent, cli_client: &SpecifyCliClient) {
    match mouse.kind {
        // Route the wheel through the keyboard nav so it scrolls whatever Up/Down
        // currently affects (active popup, focused pane, or document view).
        MouseEventKind::ScrollDown => {
            handle_key(
                app,
                KeyEvent::new(KeyCode::Down, KeyModifiers::NONE),
                cli_client,
            );
        }
        MouseEventKind::ScrollUp => {
            handle_key(
                app,
                KeyEvent::new(KeyCode::Up, KeyModifiers::NONE),
                cli_client,
            );
        }
        MouseEventKind::Down(MouseButton::Left) => {
            if let Some(action) = app.hit_test(mouse.column, mouse.row) {
                execute_click_action(app, action);
            }
        }
        _ => {}
    }
}

fn execute_click_action(app: &mut App, action: ClickAction) {
    match action {
        ClickAction::OpenPopup(kind) => app.open_popup(kind),
        ClickAction::SetScreen(screen) => app.screen = screen,
        ClickAction::OpenSettings => app.enter_settings(),
        ClickAction::SetLayout(layout) => {
            app.layout = layout;
            app.screen = Screen::Dashboard;
        }
        ClickAction::FocusPane(pane) => app.focused_pane = pane,
        ClickAction::SelectFeature(i) => {
            if i < app.project.features.len() {
                app.feature_index = i;
                app.spec_scroll = 0;
            }
        }
        ClickAction::SelectExt(i) => {
            if i < app.project.extensions.len() {
                app.ext_index = i;
            }
        }
        ClickAction::SelectPreset(i) => {
            if i < app.project.presets.len() {
                app.preset_index = i;
            }
        }
        ClickAction::SelectIntegration(i) => {
            if i < app.project.integrations.len() {
                app.integration_index = i;
            }
        }
        ClickAction::SelectWorkflow(i) => {
            if i < app.project.workflows.len() {
                app.wf_index = i;
            }
        }
        ClickAction::SelectCatalogSource(i) => {
            if i < app.current_catalog_list().len() {
                app.cat_index = i;
            }
        }
        ClickAction::SetCatalogTab(target) => {
            app.cat_tab = target;
            app.cat_index = 0;
        }
        ClickAction::SetCatalogAddCursor(pos) => {
            if app.cat_add_input.is_some() {
                app.cat_add_set_cursor(pos);
            }
        }
        ClickAction::SetExtTab(tab) => {
            app.ext_tab = tab;
        }
        ClickAction::SetSpecTab(tab) => {
            app.spec_tab = tab;
            app.spec_scroll = 0;
        }
        ClickAction::SettingsSelect(i) => {
            if i < SettingsRow::ALL.len() {
                app.settings_index = i;
                app.settings_editing = None;
            }
        }
        ClickAction::SettingsChip(row, opt_idx) => {
            if let Some(i) = SettingsRow::ALL.iter().position(|r| *r == row) {
                app.settings_index = i;
            }
            if let Some(value) = row.options().get(opt_idx) {
                app.settings_set_value(row, value);
            }
        }
        ClickAction::SettingsEdit(i) => {
            if i < SettingsRow::ALL.len() && SettingsRow::ALL[i].is_text() {
                app.settings_index = i;
                app.settings_editing = Some(i);
            }
        }
        ClickAction::LayoutEditorSelect(i) => {
            if i < app.custom_layout.panes.len() {
                app.layout_editor_index = i;
            }
        }
        ClickAction::PaletteRun(i) => {
            let commands = palette_commands();
            let input = app
                .palette
                .as_ref()
                .map(|p| p.input.clone())
                .unwrap_or_default();
            let filtered: Vec<_> = commands
                .into_iter()
                .filter(|c| {
                    input.is_empty() || c.label.to_lowercase().contains(&input.to_lowercase())
                })
                .collect();
            if let Some(cmd) = filtered.into_iter().nth(i) {
                app.palette = None;
                execute_palette_action(app, cmd.action);
            }
        }
        ClickAction::JumpToFeature(i) => {
            if i < app.project.features.len() {
                app.feature_index = i;
                app.spec_scroll = 0;
            }
            app.close_popup();
            app.screen = Screen::Dashboard;
        }
    }
}

fn execute_palette_action(app: &mut App, action: PaletteAction) {
    match action {
        PaletteAction::SetLayout(layout) => {
            app.layout = layout;
            app.screen = Screen::Dashboard;
        }
        PaletteAction::SetScreen(screen) => match screen {
            // Route guarded screens through their helpers so they no-op when there is
            // no selected feature / no constitution instead of opening an empty screen.
            Screen::SpecBrowser => app.enter_spec_browser(),
            Screen::Constitution => app.enter_constitution(),
            other => app.screen = other,
        },
        PaletteAction::ToggleTheme => app.toggle_theme(),
        PaletteAction::CycleAccent => app.cycle_accent(),
        PaletteAction::OpenPopup(PopupKind::Catalogs) => app.open_catalogs_reset_to_extensions(),
        PaletteAction::OpenPopup(kind) => app.open_popup(kind),
        PaletteAction::Quit => app.should_quit = true,
    }
}
