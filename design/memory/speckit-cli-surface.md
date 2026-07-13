---
version: "v0.12.11"
captured_at: "2026-07-12T11:48:29Z"
---

# Spec-Kit CLI Surface — v0.12.11

Captured by literally running `uvx --from git+https://github.com/github/spec-kit.git@v0.12.11 specify ... --help` (and recursing into subcommands) — this is the ground-truth CLI surface, not a changelog summary.

```
              ███████╗██████╗ ███████╗ ██████╗██╗███████╗██╗   ██╗              
              ██╔════╝██╔══██╗██╔════╝██╔════╝██║██╔════╝╚██╗ ██╔╝              
              ███████╗██████╔╝█████╗  ██║     ██║█████╗   ╚████╔╝               
              ╚════██║██╔═══╝ ██╔══╝  ██║     ██║██╔══╝    ╚██╔╝                
              ███████║██║     ███████╗╚██████╗██║██║        ██║                 
              ╚══════╝╚═╝     ╚══════╝ ╚═════╝╚═╝╚═╝        ╚═╝                 
                                                                                
               GitHub Spec Kit - Spec-Driven Development Toolkit                

                                                                                
 Usage: specify [OPTIONS] COMMAND [ARGS]...                                     
                                                                                
 Setup tool for Specify spec-driven development projects                        
                                                                                
╭─ Options ────────────────────────────────────────────────────────────────────╮
│ --version  -V        Show version and exit.                                  │
│ --help               Show this message and exit.                             │
╰──────────────────────────────────────────────────────────────────────────────╯
╭─ Commands ───────────────────────────────────────────────────────────────────╮
│ init         Initialize a new Specify project.                               │
│ check        Check that all required tools are installed.                    │
│ version      Display version and system information.                         │
│ self         Manage the specify CLI itself: check for newer releases,        │
│              preview upgrades with --dry-run, and upgrade in place.          │
│ extension    Manage spec-kit extensions                                      │
│ integration  Manage coding agent integrations                                │
│ preset       Manage spec-kit presets                                         │
│ bundle       Discover, install, and author Spec Kit bundles                  │
│ workflow     Manage and run automation workflows                             │
╰──────────────────────────────────────────────────────────────────────────────╯

=== specify init --help ===
                                                                                
 Usage: specify init [OPTIONS] [PROJECT_NAME]                                   
                                                                                
 Initialize a new Specify project.                                              
                                                                                
 Project files are scaffolded from assets bundled inside the specify-cli        
 package, so initialization does not need network access and templates          
 match the installed CLI version.                                               
                                                                                
 This command will:                                                             
 1. Check that required tools are installed                                     
 2. Let you choose your coding agent integration, or default to Copilot         
    in non-interactive sessions                                                 
 3. Install bundled Spec Kit templates, scripts, workflow, and shared           
    project infrastructure                                                      
 4. Set up coding agent integration commands and optional presets               
                                                                                
 Examples:                                                                      
     specify init my-project                                                    
     specify init my-project --integration claude                               
     specify init --ignore-agent-tools my-project                               
     specify init . --integration claude         # Initialize in current        
 directory                                                                      
     specify init .                     # Initialize in current directory       
 (interactive integration selection)                                            
     specify init --here --integration claude    # Alternative syntax for       
 current directory                                                              
     specify init --here --integration codex --integration-options="--skills"   
     specify init --here --integration codebuddy                                
     specify init --here --integration vibe      # Initialize with Mistral Vibe 
 support                                                                        
     specify init --here                                                        
     specify init --here --force  # Skip confirmation when current directory    
 not empty                                                                      
     specify init my-project --integration claude   # Claude installs skills by 
 default                                                                        
     specify init --here --integration gemini                                   
     specify init my-project --integration generic                              
 --integration-options="--commands-dir .myagent/commands/"  # Bring your own    
 agent; requires --commands-dir                                                 
     specify init my-project --integration claude --preset                      
 healthcare-compliance  # With preset                                           
                                                                                
╭─ Arguments ──────────────────────────────────────────────────────────────────╮
│   [project_name]      TEXT  Name for your new project directory (optional if │
│                             using --here, or use '.' for current directory)  │
╰──────────────────────────────────────────────────────────────────────────────╯
╭─ Options ────────────────────────────────────────────────────────────────────╮
│ --script                     TEXT  Script type to use: sh or ps              │
│ --ignore-agent-tools               Skip checks for coding agent tools like   │
│                                    Claude Code                               │
│ --here                             Initialize project in the current         │
│                                    directory instead of creating a new one   │
│ --force                            Force merge/overwrite when using --here   │
│                                    (skip confirmation)                       │
│ --preset                     TEXT  Install a preset during initialization    │
│                                    (by preset ID)                            │
│ --integration                TEXT  AI coding agent integration to use (e.g.  │
│                                    --integration copilot). See 'specify      │
│                                    check' for available integrations.        │
│ --integration-options        TEXT  Options for the integration (e.g.         │
│                                    --integration-options="--commands-dir     │
│                                    .myagent/cmds")                           │
│ --help                             Show this message and exit.               │
╰──────────────────────────────────────────────────────────────────────────────╯


=== specify check --help ===
                                                                                
 Usage: specify check [OPTIONS]                                                 
                                                                                
 Check that all required tools are installed.                                   
                                                                                
╭─ Options ────────────────────────────────────────────────────────────────────╮
│ --help          Show this message and exit.                                  │
╰──────────────────────────────────────────────────────────────────────────────╯


=== specify version --help ===
                                                                                
 Usage: specify version [OPTIONS]                                               
                                                                                
 Display version and system information.                                        
                                                                                
╭─ Options ────────────────────────────────────────────────────────────────────╮
│ --features          Show local CLI feature capabilities.                     │
│ --json              Emit feature capabilities as JSON. Requires --features.  │
│ --help              Show this message and exit.                              │
╰──────────────────────────────────────────────────────────────────────────────╯


=== specify self --help ===
                                                                                
 Usage: specify self [OPTIONS] COMMAND [ARGS]...                                
                                                                                
 Manage the specify CLI itself: check for newer releases, preview upgrades with 
 --dry-run, and upgrade in place.                                               
                                                                                
╭─ Options ────────────────────────────────────────────────────────────────────╮
│ --help          Show this message and exit.                                  │
╰──────────────────────────────────────────────────────────────────────────────╯
╭─ Commands ───────────────────────────────────────────────────────────────────╮
│ check    Check whether a newer specify-cli release is available. Read-only.  │
│ upgrade  Upgrade specify-cli to the latest release (or a pinned --tag).      │
╰──────────────────────────────────────────────────────────────────────────────╯


=== specify extension --help ===
                                                                                
 Usage: specify extension [OPTIONS] COMMAND [ARGS]...                           
                                                                                
 Manage spec-kit extensions                                                     
                                                                                
╭─ Options ────────────────────────────────────────────────────────────────────╮
│ --help          Show this message and exit.                                  │
╰──────────────────────────────────────────────────────────────────────────────╯
╭─ Commands ───────────────────────────────────────────────────────────────────╮
│ list          List installed extensions.                                     │
│ add           Install an extension.                                          │
│ remove        Uninstall an extension.                                        │
│ search        Search for available extensions in catalog.                    │
│ info          Show detailed information about an extension.                  │
│ update        Update extension(s) to latest version.                         │
│ enable        Enable a disabled extension.                                   │
│ disable       Disable an extension without removing it.                      │
│ set-priority  Set the resolution priority of an installed extension.         │
│ catalog       Manage extension catalogs                                      │
╰──────────────────────────────────────────────────────────────────────────────╯


=== specify integration --help ===
                                                                                
 Usage: specify integration [OPTIONS] COMMAND [ARGS]...                         
                                                                                
 Manage coding agent integrations                                               
                                                                                
╭─ Options ────────────────────────────────────────────────────────────────────╮
│ --help          Show this message and exit.                                  │
╰──────────────────────────────────────────────────────────────────────────────╯
╭─ Commands ───────────────────────────────────────────────────────────────────╮
│ install    Install an integration into an existing project.                  │
│ uninstall  Uninstall an integration, safely preserving modified files.       │
│ switch     Switch from the current integration to a different one.           │
│ upgrade    Upgrade an integration by reinstalling with diff-aware file       │
│            handling.                                                         │
│ list       List available integrations and installed status.                 │
│ status     Report the current project's integration status without changing  │
│            files.                                                            │
│ use        Set the default integration without uninstalling other            │
│            integrations.                                                     │
│ search     Search for integrations in the active catalog stack.              │
│ info       Show catalog details for a single integration.                    │
│ scaffold   Create a minimal built-in integration package and test skeleton.  │
│ catalog    Manage integration catalog sources                                │
╰──────────────────────────────────────────────────────────────────────────────╯


=== specify preset --help ===
                                                                                
 Usage: specify preset [OPTIONS] COMMAND [ARGS]...                              
                                                                                
 Manage spec-kit presets                                                        
                                                                                
╭─ Options ────────────────────────────────────────────────────────────────────╮
│ --help          Show this message and exit.                                  │
╰──────────────────────────────────────────────────────────────────────────────╯
╭─ Commands ───────────────────────────────────────────────────────────────────╮
│ list          List installed presets.                                        │
│ add           Install a preset.                                              │
│ remove        Remove an installed preset.                                    │
│ search        Search for presets in the catalog.                             │
│ resolve       Show which template will be resolved for a given name.         │
│ info          Show detailed information about a preset.                      │
│ set-priority  Set the resolution priority of an installed preset.            │
│ enable        Enable a disabled preset.                                      │
│ disable       Disable a preset without removing it.                          │
│ catalog       Manage preset catalogs                                         │
╰──────────────────────────────────────────────────────────────────────────────╯


=== specify bundle --help ===
                                                                                
 Usage: specify bundle [OPTIONS] COMMAND [ARGS]...                              
                                                                                
 Discover, install, and author Spec Kit bundles                                 
                                                                                
╭─ Options ────────────────────────────────────────────────────────────────────╮
│ --help          Show this message and exit.                                  │
╰──────────────────────────────────────────────────────────────────────────────╯
╭─ Commands ───────────────────────────────────────────────────────────────────╮
│ search    List matching bundles across the active catalog stack.             │
│ info      Show full metadata and the fully expanded component set (== what   │
│           install adds).                                                     │
│ list      List bundles currently installed in the project with versions.     │
│ install   Install a bundle's full component set through each primitive's     │
│           machinery.                                                         │
│ update    Re-resolve and refresh a bundle's components via each primitive's  │
│           update path.                                                       │
│ remove    Uninstall only the components this bundle contributed (no          │
│           collateral removals).                                              │
│ validate  Report whether the manifest is well-formed and references resolve. │
│ build     Produce a single versioned distributable artifact (.zip).          │
│ init      Ensure the project is initialized (idempotent), then optionally    │
│           install a bundle.                                                  │
│ catalog   Manage bundle catalog sources                                      │
╰──────────────────────────────────────────────────────────────────────────────╯


=== specify workflow --help ===
                                                                                
 Usage: specify workflow [OPTIONS] COMMAND [ARGS]...                            
                                                                                
 Manage and run automation workflows                                            
                                                                                
╭─ Options ────────────────────────────────────────────────────────────────────╮
│ --help          Show this message and exit.                                  │
╰──────────────────────────────────────────────────────────────────────────────╯
╭─ Commands ───────────────────────────────────────────────────────────────────╮
│ run      Run a workflow from an installed ID or local YAML path.             │
│ resume   Resume a paused or failed workflow run.                             │
│ status   Show workflow run status.                                           │
│ list     List installed workflows.                                           │
│ add      Install a workflow from catalog, URL, or local path.                │
│ remove   Uninstall a workflow.                                               │
│ search   Search workflow catalogs.                                           │
│ info     Show workflow details and step graph.                               │
│ catalog  Manage workflow catalogs                                            │
│ step     Manage workflow step types                                          │
╰──────────────────────────────────────────────────────────────────────────────╯


=== specify self check --help ===
                                                                                
 Usage: specify self check [OPTIONS]                                            
                                                                                
 Check whether a newer specify-cli release is available. Read-only.             
                                                                                
 This command only checks for updates; it does not modify your installation.    
 Use `specify self upgrade` to actually perform the upgrade once you've seen    
 the result here, or `specify self upgrade --dry-run` to preview the            
 installer command without running it.                                          
                                                                                
╭─ Options ────────────────────────────────────────────────────────────────────╮
│ --help          Show this message and exit.                                  │
╰──────────────────────────────────────────────────────────────────────────────╯


=== specify self upgrade --help ===
                                                                                
 Usage: specify self upgrade [OPTIONS]                                          
                                                                                
 Upgrade specify-cli to the latest release (or a pinned --tag).                 
                                                                                
 Bare invocation executes immediately with no confirmation prompt, matching     
 pip install -U / uv tool upgrade / npm update conventions. Use --dry-run       
 to preview without mutating anything. See `specify self check` for the         
 non-destructive read-only counterpart.                                         
                                                                                
 Detection classifies the runtime into uv-tool / pipx / uvx (ephemeral) /       
 source-checkout / unsupported. Only uv-tool and pipx are upgraded              
 automatically; the other three paths print path-specific guidance and          
 exit 0.                                                                        
                                                                                
 Exit codes:                                                                    
   0      success or no-op-success (already on latest, --dry-run, or            
          non-upgradable path with guidance shown)                              
   1      target-tag resolution failure or --tag regex validation failure       
   2      verification mismatch when the installer exited 0 but                 
          `specify --version` does not resolve to the target tag; if the        
          installer itself exits 2, that installer failure code is              
          propagated verbatim                                                   
   3      installer binary not found on PATH, or resolved installer path is     
          missing / non-executable                                              
   124    internal installer timeout when SPECIFY_UPGRADE_TIMEOUT_SECS is set,  
          or a real installer exit code 124 propagated verbatim; scripts        
          should treat 124 as ambiguous and inspect the failure message         
   other  installer exit code propagated verbatim                               
                                                                                
 Environment variables:                                                         
   SPECIFY_UPGRADE_TIMEOUT_SECS  Optional integer/float seconds. Caps how       
     long the installer subprocess may run. Unset (default) means no            
     timeout — interrupt with Ctrl+C if the installer hangs.                    
                                                                                
╭─ Options ────────────────────────────────────────────────────────────────────╮
│ --dry-run              Print the preview (method, current, target, installer │
│                        argv) and exit 0 without launching the installer      │
│                        subprocess.                                           │
│ --tag            TEXT  Pin the target version (vX.Y.Z). Without --tag, the   │
│                        latest stable release is resolved via GitHub          │
│                        Releases.                                             │
│ --help                 Show this message and exit.                           │
╰──────────────────────────────────────────────────────────────────────────────╯


=== specify extension catalog --help ===
                                                                                
 Usage: specify extension catalog [OPTIONS] COMMAND [ARGS]...                   
                                                                                
 Manage extension catalogs                                                      
                                                                                
╭─ Options ────────────────────────────────────────────────────────────────────╮
│ --help          Show this message and exit.                                  │
╰──────────────────────────────────────────────────────────────────────────────╯
╭─ Commands ───────────────────────────────────────────────────────────────────╮
│ list    List all active extension catalogs.                                  │
│ add     Add a catalog to .specify/extension-catalogs.yml.                    │
│ remove  Remove a catalog from .specify/extension-catalogs.yml.               │
╰──────────────────────────────────────────────────────────────────────────────╯


=== specify integration catalog --help ===
                                                                                
 Usage: specify integration catalog [OPTIONS] COMMAND [ARGS]...                 
                                                                                
 Manage integration catalog sources                                             
                                                                                
╭─ Options ────────────────────────────────────────────────────────────────────╮
│ --help          Show this message and exit.                                  │
╰──────────────────────────────────────────────────────────────────────────────╯
╭─ Commands ───────────────────────────────────────────────────────────────────╮
│ list    List configured integration catalog sources.                         │
│ add     Add an integration catalog source to the project config.             │
│ remove  Remove an integration catalog source by 0-based index.               │
╰──────────────────────────────────────────────────────────────────────────────╯


=== specify preset catalog --help ===
                                                                                
 Usage: specify preset catalog [OPTIONS] COMMAND [ARGS]...                      
                                                                                
 Manage preset catalogs                                                         
                                                                                
╭─ Options ────────────────────────────────────────────────────────────────────╮
│ --help          Show this message and exit.                                  │
╰──────────────────────────────────────────────────────────────────────────────╯
╭─ Commands ───────────────────────────────────────────────────────────────────╮
│ list    List all active preset catalogs.                                     │
│ add     Add a catalog to .specify/preset-catalogs.yml.                       │
│ remove  Remove a catalog from .specify/preset-catalogs.yml.                  │
╰──────────────────────────────────────────────────────────────────────────────╯


=== specify bundle catalog --help ===
                                                                                
 Usage: specify bundle catalog [OPTIONS] COMMAND [ARGS]...                      
                                                                                
 Manage bundle catalog sources                                                  
                                                                                
╭─ Options ────────────────────────────────────────────────────────────────────╮
│ --help          Show this message and exit.                                  │
╰──────────────────────────────────────────────────────────────────────────────╯
╭─ Commands ───────────────────────────────────────────────────────────────────╮
│ list    Print the active, priority-ordered catalog stack with scope and      │
│         policy.                                                              │
│ add     Register a project-scoped catalog source and persist it.             │
│ remove  Remove a project-scoped catalog source (built-in defaults can't be   │
│         deleted).                                                            │
╰──────────────────────────────────────────────────────────────────────────────╯


=== specify workflow catalog --help ===
                                                                                
 Usage: specify workflow catalog [OPTIONS] COMMAND [ARGS]...                    
                                                                                
 Manage workflow catalogs                                                       
                                                                                
╭─ Options ────────────────────────────────────────────────────────────────────╮
│ --help          Show this message and exit.                                  │
╰──────────────────────────────────────────────────────────────────────────────╯
╭─ Commands ───────────────────────────────────────────────────────────────────╮
│ list    List configured workflow catalog sources.                            │
│ add     Add a workflow catalog source.                                       │
│ remove  Remove a workflow catalog source by index.                           │
╰──────────────────────────────────────────────────────────────────────────────╯


=== specify workflow step --help ===
                                                                                
 Usage: specify workflow step [OPTIONS] COMMAND [ARGS]...                       
                                                                                
 Manage workflow step types                                                     
                                                                                
╭─ Options ────────────────────────────────────────────────────────────────────╮
│ --help          Show this message and exit.                                  │
╰──────────────────────────────────────────────────────────────────────────────╯
╭─ Commands ───────────────────────────────────────────────────────────────────╮
│ list     List installed step types (built-in and custom).                    │
│ add      Install a custom step type from the step catalog.                   │
│ remove   Uninstall a custom step type.                                       │
│ search   Search the step type catalog.                                       │
│ info     Show details for a step type.                                       │
│ catalog  Manage step catalogs                                                │
╰──────────────────────────────────────────────────────────────────────────────╯


```
