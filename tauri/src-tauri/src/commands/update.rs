use crate::state::{AppState, RustBuildProgress, UpdateProcess};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::os::unix::process::CommandExt as _;
use std::sync::{Arc, LazyLock};
use tauri::{AppHandle, Emitter, State};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::Mutex;

static RE_DERIVATIONS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"these (\d+) derivations? will be built").unwrap());
static RE_RECEIVING: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"Receiving objects:\s*(\d+)%").unwrap());
static RE_RESOLVING: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"Resolving deltas:\s*(\d+)%").unwrap());

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UpdateExecuteParams {
    github_repo_owner: String,
    github_repo_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    github_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tag: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    branch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    commit: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UpdateStepParams {
    step_id: String,
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    progress: Option<u8>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateResult {
    success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

#[tauri::command]
pub async fn update_execute(
    app: AppHandle,
    state: State<'_, AppState>,
    params: UpdateExecuteParams,
) -> Result<(), String> {
    // Prevent concurrent updates
    {
        let process_lock = state.update_process.lock().await;
        if process_lock.is_some() {
            return Err("An update is already in progress".to_string());
        }
    }

    let state_clone = state.inner().clone();

    tokio::spawn(async move {
        match run_update(&app, &state_clone, &params).await {
            Ok(()) => {
                app.emit(
                    "update-end",
                    UpdateResult {
                        success: true,
                        error: None,
                    },
                )
                .ok();
            }
            Err(e) => {
                app.emit(
                    "update-end",
                    UpdateResult {
                        success: false,
                        error: Some(e.to_string()),
                    },
                )
                .ok();
            }
        }
    });

    Ok(())
}

#[tauri::command]
pub async fn update_cancel(state: State<'_, AppState>) -> Result<UpdateResult, String> {
    let mut process_lock = state.update_process.lock().await;

    if let Some(mut update_process) = process_lock.take() {
        // Kill the entire process group to terminate all child processes
        if let Some(pid) = update_process.child.id() {
            unsafe {
                libc::killpg(pid as libc::pid_t, libc::SIGTERM);
            }
            // Give processes a moment to terminate gracefully, then force kill
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            unsafe {
                libc::killpg(pid as libc::pid_t, libc::SIGKILL);
            }
        }
        // Also kill the direct child as a fallback
        match update_process.child.kill().await {
            Ok(()) => Ok(UpdateResult {
                success: true,
                error: None,
            }),
            Err(e) => Ok(UpdateResult {
                success: false,
                error: Some(e.to_string()),
            }),
        }
    } else {
        Ok(UpdateResult {
            success: false,
            error: Some("No update process running".to_string()),
        })
    }
}

async fn run_update(
    app: &AppHandle,
    state: &AppState,
    params: &UpdateExecuteParams,
) -> Result<(), anyhow::Error> {
    let home_dir = if std::env::var("QITECH_CONTROL_ENV").unwrap_or_default() == "control-os" {
        "/home/qitech".to_string()
    } else {
        std::env::var("HOME")?
    };

    let repo_dir = format!("{}/{}", home_dir, params.github_repo_name);

    // Step 1: Clear repo directory
    clear_repo_directory(app, &repo_dir).await?;

    // Step 2: Clone repository
    emit_step(app, "clone-repo", "in-progress", None)?;
    clone_repository(app, params, &home_dir).await?;
    emit_step(app, "clone-repo", "completed", None)?;

    // Step 3: Make script executable
    Command::new("chmod")
        .args(["+x", "nixos-install.sh"])
        .current_dir(&repo_dir)
        .output()
        .await?;

    // Step 4: Run nixos-install.sh with progress tracking
    emit_step(app, "rust-build", "in-progress", None)?;
    run_install_script(app, state, &repo_dir).await?;

    Ok(())
}

async fn clear_repo_directory(app: &AppHandle, repo_dir: &str) -> Result<(), anyhow::Error> {
    if std::path::Path::new(repo_dir).exists() {
        tokio::fs::remove_dir_all(repo_dir).await?;
        emit_log(
            app,
            &terminal_success(&format!("Deleted existing repository at {repo_dir}")),
        )?;
    } else {
        emit_log(
            app,
            &terminal_info(&format!(
                "No existing repository found at {repo_dir}, nothing to delete"
            )),
        )?;
    }
    Ok(())
}

async fn clone_repository(
    app: &AppHandle,
    params: &UpdateExecuteParams,
    home_dir: &str,
) -> Result<(), anyhow::Error> {
    let repo_url = if let Some(token) = &params.github_token {
        format!(
            "https://{}@github.com/{}/{}.git",
            token, params.github_repo_owner, params.github_repo_name
        )
    } else {
        format!(
            "https://github.com/{}/{}.git",
            params.github_repo_owner, params.github_repo_name
        )
    };

    let mut args = vec!["clone".to_string(), "--progress".to_string(), repo_url];

    if let Some(tag) = &params.tag {
        args.extend_from_slice(&[
            "--branch".to_string(),
            tag.clone(),
            "--single-branch".to_string(),
        ]);
        emit_log(app, &terminal_info(&format!("Cloning tag: {tag}")))?;
    } else if let Some(branch) = &params.branch {
        args.extend_from_slice(&[
            "--branch".to_string(),
            branch.clone(),
            "--single-branch".to_string(),
        ]);
        emit_log(app, &terminal_info(&format!("Cloning branch: {branch}")))?;
    } else if let Some(commit) = &params.commit {
        emit_log(
            app,
            &terminal_info(&format!(
                "Cloning repository, will checkout commit: {commit}"
            )),
        )?;
    } else {
        return Err(anyhow::anyhow!("No specific version specified!"));
    }

    let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    let token_to_redact = params.github_token.clone();
    run_command_with_output(app, "git", &args_ref, home_dir, move |line, app| {
        // Redact the GitHub token from any logged output
        let line = if let Some(ref token) = token_to_redact {
            line.replace(token, "***")
        } else {
            line.to_string()
        };
        parse_git_progress(&line, app)
    })
    .await?;

    // Checkout specific commit if specified
    if let Some(commit) = &params.commit {
        let repo_dir = format!("{home_dir}/{}", params.github_repo_name);
        run_command_with_output(app, "git", &["checkout", commit], &repo_dir, |_, _| Ok(()))
            .await?;
        emit_log(
            app,
            &terminal_success(&format!("Successfully checked out commit: {commit}")),
        )?;
    }

    emit_log(app, &terminal_success("Repository cloned successfully"))?;
    Ok(())
}

async fn run_install_script(
    app: &AppHandle,
    state: &AppState,
    repo_dir: &str,
) -> Result<(), anyhow::Error> {
    let complete_command = "./nixos-install.sh";
    let working_dir_text = terminal_gray(repo_dir);
    emit_log(
        app,
        &format!(
            "🚀 {working_dir_text} {}",
            terminal_color("blue", complete_command)
        ),
    )?;

    let mut cmd = Command::new("./nixos-install.sh");
    cmd.current_dir(repo_dir)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    // Create a new process group so we can kill the entire tree on cancel
    unsafe {
        cmd.pre_exec(|| {
            libc::setpgid(0, 0);
            Ok(())
        });
    }
    let mut child = cmd.spawn()?;

    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

    // Store process in state for cancellation
    {
        let mut process_lock = state.update_process.lock().await;
        *process_lock = Some(UpdateProcess {
            child,
            rust_build_progress: RustBuildProgress::default(),
        });
    }

    let update_process = state.update_process.clone();
    let mut system_install_started = false;

    // Stream stdout
    let app_stdout = app.clone();
    let update_process_stdout = update_process.clone();
    let stdout_task = tokio::spawn(async move {
        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();
        while let Ok(Some(line)) = lines.next_line().await {
            emit_log(&app_stdout, &line).ok();
            parse_rust_build_output(&app_stdout, &update_process_stdout, &line).await;
        }
    });

    // Stream stderr
    let app_stderr = app.clone();
    let update_process_stderr = update_process.clone();
    let stderr_task = tokio::spawn(async move {
        let reader = BufReader::new(stderr);
        let mut lines = reader.lines();
        while let Ok(Some(line)) = lines.next_line().await {
            emit_log(&app_stderr, &line).ok();
            parse_rust_build_output(&app_stderr, &update_process_stderr, &line).await;
        }
    });

    // Wait for output streaming to complete
    stdout_task.await?;
    stderr_task.await?;

    // Wait for process to complete
    let mut process_lock = update_process.lock().await;
    if let Some(mut update_process) = process_lock.take() {
        let status = update_process.child.wait().await?;

        // Check if system install was detected based on progress
        if update_process.rust_build_progress.max_percent >= 90 {
            system_install_started = true;
        }

        if !status.success() {
            emit_step(app, "rust-build", "error", None)?;
            emit_step(app, "system-install", "error", None)?;
            return Err(anyhow::anyhow!(
                "Install script failed with code {:?}",
                status.code()
            ));
        }

        // Mark remaining steps as completed on success
        if !system_install_started {
            emit_step(app, "rust-build", "completed", None)?;
        }
        emit_step(app, "system-install", "completed", None)?;
    }

    emit_log(app, &terminal_success("Command completed successfully"))?;
    Ok(())
}

async fn parse_rust_build_output(
    app: &AppHandle,
    update_process: &Arc<Mutex<Option<UpdateProcess>>>,
    line: &str,
) {
    let line_lower = line.to_lowercase();

    // Track derivations
    if let Some(captures) = RE_DERIVATIONS.captures(line) {
        if let Some(total_match) = captures.get(1) {
            if let Ok(total) = total_match.as_str().parse::<usize>() {
                let mut process_lock = update_process.lock().await;
                if let Some(up) = process_lock.as_mut() {
                    up.rust_build_progress.total_derivations = total;
                    up.rust_build_progress.built_derivations = 0;
                    up.rust_build_progress.max_percent = 0;
                }
                emit_step(app, "rust-build", "in-progress", Some(0)).ok();
                return;
            }
        }
    }

    // Track building packages
    if line_lower.contains("building '/nix/store/") || line_lower.contains("building /nix/store/") {
        let mut process_lock = update_process.lock().await;
        if let Some(up) = process_lock.as_mut() {
            up.rust_build_progress.built_derivations += 1;

            let is_server_deps = line.contains("-server-deps");

            let mut percent = 15usize;
            if is_server_deps {
                percent = 85;
            } else if up.rust_build_progress.total_derivations > 0 {
                let ratio = up.rust_build_progress.built_derivations as f32
                    / up.rust_build_progress.total_derivations as f32;
                percent = 15 + (ratio * 70.0) as usize;
            }

            percent = percent.max(up.rust_build_progress.max_percent);
            up.rust_build_progress.max_percent = percent;

            emit_step(app, "rust-build", "in-progress", Some(percent as u8)).ok();
        }
    }

    // Track installing phase
    if line_lower.contains("installing") {
        let mut process_lock = update_process.lock().await;
        if let Some(up) = process_lock.as_mut() {
            let percent = 90usize.max(up.rust_build_progress.max_percent);
            up.rust_build_progress.max_percent = percent;
            emit_step(app, "rust-build", "in-progress", Some(percent as u8)).ok();
        }
    }

    // Detect system install phase
    if line_lower.contains("updating grub")
        || line_lower.contains("installing bootloader")
        || line_lower.contains("updating bootloader")
        || line_lower.contains("activating the configuration")
        || line_lower.contains("building the system configuration")
        || line_lower.contains("these 0 derivations")
    {
        let process_lock = update_process.lock().await;
        if let Some(up) = process_lock.as_ref() {
            if up.rust_build_progress.max_percent >= 90 {
                emit_step(app, "rust-build", "completed", None).ok();
                emit_step(app, "system-install", "in-progress", None).ok();
            }
        }
    }
}

fn parse_git_progress(line: &str, app: &AppHandle) -> Result<(), anyhow::Error> {
    if let Some(captures) = RE_RECEIVING.captures(line) {
        if let Some(percent_match) = captures.get(1) {
            if let Ok(percent) = percent_match.as_str().parse::<u32>() {
                emit_step(
                    app,
                    "clone-repo",
                    "in-progress",
                    Some((percent as f32 * 0.8) as u8),
                )?;
                return Ok(());
            }
        }
    }

    if let Some(captures) = RE_RESOLVING.captures(line) {
        if let Some(percent_match) = captures.get(1) {
            if let Ok(percent) = percent_match.as_str().parse::<u32>() {
                emit_step(
                    app,
                    "clone-repo",
                    "in-progress",
                    Some(80 + (percent as f32 * 0.2) as u8),
                )?;
            }
        }
    }

    Ok(())
}

async fn run_command_with_output<F>(
    app: &AppHandle,
    cmd: &str,
    args: &[&str],
    cwd: &str,
    parse_fn: F,
) -> Result<(), anyhow::Error>
where
    F: Fn(&str, &AppHandle) -> Result<(), anyhow::Error> + Send + Sync + 'static,
{
    let complete_command = format!("{cmd} {}", args.join(" "));
    // Redact any tokens from the logged command (tokens appear in git clone URLs)
    let safe_command = redact_url_credentials(&complete_command);
    let working_dir_text = terminal_gray(cwd);
    emit_log(
        app,
        &format!(
            "🚀 {working_dir_text} {}",
            terminal_color("blue", &safe_command)
        ),
    )?;

    let mut child = Command::new(cmd)
        .args(args)
        .current_dir(cwd)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()?;

    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

    let parse_fn = Arc::new(parse_fn);

    // Stream stdout
    let app_stdout = app.clone();
    let stdout_task = tokio::spawn(async move {
        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();
        while let Ok(Some(line)) = lines.next_line().await {
            emit_log(&app_stdout, &line).ok();
        }
    });

    // Stream stderr (git outputs progress to stderr)
    let app_stderr = app.clone();
    let parse_fn_clone = parse_fn.clone();
    let stderr_task = tokio::spawn(async move {
        let reader = BufReader::new(stderr);
        let mut lines = reader.lines();
        while let Ok(Some(line)) = lines.next_line().await {
            emit_log(&app_stderr, &line).ok();
            parse_fn_clone(&line, &app_stderr).ok();
        }
    });

    let status = child.wait().await?;
    stdout_task.await?;
    stderr_task.await?;

    if !status.success() {
        let code = status.code().unwrap_or(-1);
        if code == -1 {
            // Process was killed (signal)
            emit_log(app, &terminal_info("Command was cancelled"))?;
            return Err(anyhow::anyhow!("Command was cancelled"));
        }
        emit_log(
            app,
            &terminal_error(&format!("Command failed with code {code}")),
        )?;
        return Err(anyhow::anyhow!("Command failed with code {code}"));
    }

    emit_log(app, &terminal_success("Command completed successfully"))?;
    Ok(())
}

// Terminal color helpers (matching Electron's ANSI escape sequences)
fn terminal_color(color: &str, text: &str) -> String {
    let code = match color {
        "blue" => "\x1b[34m",
        "green" => "\x1b[32m",
        "red" => "\x1b[31m",
        "cyan" => "\x1b[36m",
        "gray" => "\x1b[90m",
        _ => "",
    };
    format!("{code}{text}\x1b[0m")
}

fn terminal_error(text: &str) -> String {
    terminal_color("red", &format!("💥 {text}"))
}

fn terminal_success(text: &str) -> String {
    terminal_color("green", &format!("✅ {text}"))
}

fn terminal_info(text: &str) -> String {
    terminal_color("cyan", &format!("📝 {text}"))
}

fn terminal_gray(text: &str) -> String {
    terminal_color("gray", text)
}

fn redact_url_credentials(text: &str) -> String {
    // Redact tokens from URLs like https://TOKEN@github.com/...
    static RE_URL_TOKEN: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"https://[^@]+@github\.com").unwrap());
    RE_URL_TOKEN
        .replace_all(text, "https://***@github.com")
        .to_string()
}

fn emit_log(app: &AppHandle, log: &str) -> Result<(), anyhow::Error> {
    app.emit("update-log", log)?;
    Ok(())
}

fn emit_step(
    app: &AppHandle,
    step_id: &str,
    status: &str,
    progress: Option<u8>,
) -> Result<(), anyhow::Error> {
    app.emit(
        "update-step",
        UpdateStepParams {
            step_id: step_id.to_string(),
            status: status.to_string(),
            progress,
        },
    )?;
    Ok(())
}
