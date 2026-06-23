import { dialog, ipcMain } from "electron";
import {
  UPDATE_FETCH_TARGETS_SEND,
  UPDATE_CANCEL,
  UPDATE_END,
  UPDATE_EXECUTE,
  UPDATE_LOG,
  UPDATE_STEP,
  UPDATE_FETCH_TARGETS_RECV,
  UPDATE_FETCH_CHANGELOG_SEND,
  UPDATE_FETCH_CHANGELOG_RECV,
  UPDATE_SAVE_TOKEN,
  UPDATE_LOAD_TOKEN_FILE,
  UPDATE_HAS_TOKEN,
  UPDATE_CLEAR_TOKEN,
} from "./update-channels";
import { spawn, ChildProcess } from "child_process";
import tkill from "@jub3i/tree-kill";
import { existsSync, readFileSync, rmSync } from "fs";
import { GithubSource } from "@/setup/GithubSourceDialog";
import {
  fetchChangelog,
  fetchTargets,
  removeStaleLockFiles,
} from "./git-fetch-utils";
import {
  clearToken,
  gitAuthArgs,
  hasToken,
  saveToken,
  usbDefaultPath,
} from "./token-store";

type UpdateExecuteListenerParams = {
  githubRepoOwner: string;
  githubRepoName: string;
  tag?: string;
  branch?: string;
  commit?: string;
};

// Store reference to current update process for cancellation
let currentUpdateProcess: ChildProcess | null = null;

export function addUpdateEventListeners() {
  ipcMain.handle(
    UPDATE_FETCH_TARGETS_SEND,
    async (event, source: GithubSource) => {
      try {
        const result = await fetchTargets(
          source.githubRepoOwner,
          source.githubRepoName,
        );
        event.sender.send(UPDATE_FETCH_TARGETS_RECV, result);
      } catch (error: any) {
        event.sender.send(
          UPDATE_FETCH_TARGETS_RECV,
          `get update targets failed: ${error}`,
        );
      }
    },
  );

  ipcMain.handle(
    UPDATE_FETCH_CHANGELOG_SEND,
    async (event, args: { source: GithubSource; ref: string }) => {
      try {
        const result = await fetchChangelog(
          args.source.githubRepoOwner,
          args.source.githubRepoName,
          args.ref,
        );

        event.sender.send(UPDATE_FETCH_CHANGELOG_RECV, result);
      } catch (error: any) {
        event.sender.send(
          UPDATE_FETCH_CHANGELOG_RECV,
          `get update changelog failed: ${error}`,
        );
      }
    },
  );

  ipcMain.handle(
    UPDATE_EXECUTE,
    async (event, params: UpdateExecuteListenerParams) => {
      update(event, params)
        .then(() => {
          currentUpdateProcess = null;
          event.sender.send(
            UPDATE_END,
            terminalSuccess("Update completed successfully!"),
          );
        })
        .catch((error) => {
          currentUpdateProcess = null;
          event.sender.send(
            UPDATE_END,
            terminalError(`Update failed: ${error.message}`),
          );
        });
    },
  );

  // ── Token management handlers ────────────────────────────────────────────
  ipcMain.handle(UPDATE_HAS_TOKEN, async () => {
    return hasToken();
  });

  ipcMain.handle(UPDATE_SAVE_TOKEN, async (_event, token: string) => {
    try {
      saveToken(token);
      return { success: true };
    } catch (error: any) {
      return { success: false, error: error?.message ?? String(error) };
    }
  });

  ipcMain.handle(UPDATE_CLEAR_TOKEN, async () => {
    clearToken();
    return { success: true };
  });

  ipcMain.handle(UPDATE_LOAD_TOKEN_FILE, async () => {
    try {
      const result = await dialog.showOpenDialog({
        properties: ["openFile"],
        defaultPath: usbDefaultPath(),
        filters: [
          { name: "Token files", extensions: ["txt"] },
          { name: "All files", extensions: ["*"] },
        ],
      });

      if (result.canceled || !result.filePaths[0]) {
        return { success: false, error: "Cancelled" };
      }

      const filePath = result.filePaths[0];
      const content = readFileSync(filePath, "utf8");

      // Parse: take first non-empty trimmed line
      const token = content
        .split("\n")
        .map((l) => l.trim())
        .find((l) => l.length > 0);

      if (!token || /\s/.test(token)) {
        return {
          success: false,
          error: "File does not contain a valid token",
        };
      }

      saveToken(token);
      return { success: true };
    } catch (error: any) {
      return { success: false, error: error?.message ?? String(error) };
    }
  });
  // ────────────────────────────────────────────────────────────────────────

  ipcMain.handle(UPDATE_CANCEL, async (event) => {
    if (currentUpdateProcess) {
      event.sender.send(
        UPDATE_LOG,
        terminalInfo("Cancelling update process..."),
      );

      // Kill the process and all its child processes using tree-kill
      try {
        const pid = currentUpdateProcess.pid!;

        // Use tree-kill to properly terminate the entire process tree
        // First try graceful termination with SIGTERM (default signal)
        await new Promise<void>((resolve, reject) => {
          tkill(pid, (err) => {
            if (err) {
              // If graceful termination fails, force kill with SIGKILL
              tkill(pid, "SIGKILL", (killErr) => {
                if (killErr) {
                  reject(killErr);
                } else {
                  resolve();
                }
              });
            } else {
              resolve();
            }
          });
        });

        currentUpdateProcess = null;
        event.sender.send(UPDATE_END, terminalInfo("Update process cancelled"));
        return { success: true };
      } catch (error: any) {
        event.sender.send(
          UPDATE_LOG,
          terminalError(`Error cancelling process: ${error.message}`),
        );
        return { success: false, error: error.message };
      }
    } else {
      event.sender.send(UPDATE_LOG, terminalInfo("No update process running"));
      return { success: false, error: "No update process running" };
    }
  });
}

async function update(
  event: Electron.IpcMainInvokeEvent,
  params: UpdateExecuteListenerParams,
): Promise<void> {
  return new Promise((resolve, reject) => {
    (async () => {
      try {
        const { githubRepoOwner, githubRepoName, tag, branch, commit } = params;

        // Implement your update logic here
        console.log("Update parameters:", {
          githubRepoOwner,
          githubRepoName,
          tag,
          branch,
          commit,
        });

        // Reset Rust build progress tracking
        rustBuildProgress = {
          totalDerivations: 0,
          builtDerivations: 0,
          maxPercent: 0,
        };

        const qitechControlEnv = process.env.QITECH_CONTROL_ENV;
        const homeDir =
          qitechControlEnv === "control-os" ? "/home/qitech" : process.env.HOME;
        if (!homeDir) {
          event.sender.send(
            UPDATE_LOG,
            terminalColor("red", terminalError("Home directory not found")),
          );
          return;
        }

        // 1. Prepare the repository. If a valid clone of the target repo already
        // exists we reuse it via an incremental fetch + checkout (no full re-clone
        // over the network); otherwise we fall back to a fresh clone.
        const repoDir = `${homeDir}/${githubRepoName}`;

        event.sender.send(UPDATE_STEP, {
          stepId: "clone-repo",
          status: "in-progress",
        });

        let prepareResult: { success: boolean; error?: string };

        if (await isValidCloneOf(repoDir, githubRepoOwner, githubRepoName)) {
          // REUSE PATH — incremental update of the existing clone.
          prepareResult = await updateRepository(
            { githubRepoOwner, githubRepoName, tag, branch, commit },
            event,
          );

          // Self-heal: if the incremental update fails (e.g. corrupt .git), fall
          // back to a clean clear + clone before giving up.
          if (!prepareResult.success) {
            event.sender.send(
              UPDATE_LOG,
              terminalInfo(
                "Incremental update failed, falling back to fresh clone",
              ),
            );
            const clearResult = await clearRepoDirectory(repoDir, event);
            if (!clearResult.success) {
              event.sender.send(UPDATE_LOG, clearResult.error);
              event.sender.send(UPDATE_STEP, {
                stepId: "clone-repo",
                status: "error",
              });
              return;
            }
            prepareResult = await cloneRepository(
              { githubRepoOwner, githubRepoName, tag, branch, commit },
              event,
            );
          }
        } else {
          // FRESH CLONE PATH — first run, corrupt, or wrong remote.
          const clearResult = await clearRepoDirectory(repoDir, event);
          if (!clearResult.success) {
            event.sender.send(UPDATE_LOG, clearResult.error);
            event.sender.send(UPDATE_STEP, {
              stepId: "clone-repo",
              status: "error",
            });
            return;
          }
          prepareResult = await cloneRepository(
            { githubRepoOwner, githubRepoName, tag, branch, commit },
            event,
          );
        }

        if (!prepareResult.success) {
          event.sender.send(UPDATE_STEP, {
            stepId: "clone-repo",
            status: "error",
          });
          event.sender.send(UPDATE_LOG, prepareResult.error);
          return;
        }
        event.sender.send(UPDATE_STEP, {
          stepId: "clone-repo",
          status: "completed",
        });

        // 3. make the nixos-install.sh script executable (not tracked in progress UI)
        const chmodResult = await runCommand(
          "chmod",
          ["+x", "nixos-install.sh"],
          repoDir,
          event,
        );
        if (!chmodResult.success) {
          event.sender.send(UPDATE_LOG, chmodResult.error);
          return;
        }

        // 4. run the nixos-install.sh script
        // This script will handle rust-build, electron-build, and system-install
        // Start with rust-build (cargo builds)
        event.sender.send(UPDATE_STEP, {
          stepId: "rust-build",
          status: "in-progress",
        });

        const installResult = await runCommandWithStepTracking(
          "./nixos-install.sh",
          [],
          repoDir,
          event,
        );

        if (!installResult.success) {
          // Mark current and remaining steps as error
          event.sender.send(UPDATE_STEP, {
            stepId: "rust-build",
            status: "error",
          });
          event.sender.send(UPDATE_STEP, {
            stepId: "system-install",
            status: "error",
          });
          event.sender.send(UPDATE_LOG, installResult.error);
          return;
        }

        // Success - steps are already marked as completed by runCommandWithStepTracking
        resolve();
      } catch (error: any) {
        reject(error);
      }
    })();
  });
}

type CloneRepositoryParams = {
  githubRepoOwner: string;
  githubRepoName: string;
  tag?: string;
  branch?: string;
  commit?: string;
};

async function clearRepoDirectory(
  repoDir: string,
  event: Electron.IpcMainInvokeEvent,
): Promise<{ success: boolean; error?: string }> {
  try {
    // Check if the repo directory exists
    if (existsSync(repoDir)) {
      // If it exists, delete the repo directory
      rmSync(repoDir, { recursive: true, force: true });
      event.sender.send(
        UPDATE_LOG,
        terminalSuccess(`Deleted existing repository at ${repoDir}`),
      );
    } else {
      event.sender.send(
        UPDATE_LOG,
        terminalInfo(
          `No existing repository found at ${repoDir}, nothing to delete`,
        ),
      );
    }
    return { success: true };
  } catch (error: any) {
    event.sender.send(UPDATE_LOG, terminalError(`Error: ${error.toString()}`));
    return { success: false, error: error.toString() };
  }
}

// Run a git command and capture its stdout (used for read-only inspection like
// `git remote get-url origin`). Unlike runCommand this does not stream to the UI.
function captureGit(
  args: string[],
  workingDir: string,
): Promise<{ success: boolean; stdout: string }> {
  return new Promise((resolve) => {
    try {
      const child = spawn("git", args, {
        cwd: workingDir,
        stdio: ["ignore", "pipe", "pipe"],
        env: { ...process.env, GIT_TERMINAL_PROMPT: "0" },
      });
      let stdout = "";
      child.stdout.on("data", (data) => {
        stdout += data.toString();
      });
      child.on("error", () => resolve({ success: false, stdout: "" }));
      child.on("close", (code) =>
        resolve({ success: code === 0, stdout: stdout.trim() }),
      );
    } catch {
      resolve({ success: false, stdout: "" });
    }
  });
}

// Normalize a GitHub URL to `owner/name` (lowercased, credentials and trailing
// `.git` stripped) so we can compare an existing clone's remote against the target.
function normalizeGithubRepo(url: string): string | null {
  const match = url
    .trim()
    .toLowerCase()
    .match(/github\.com[/:]([^/]+)\/([^/]+?)(?:\.git)?\/?$/);
  if (!match) return null;
  return `${match[1]}/${match[2]}`;
}

// Returns true if repoDir is an existing git clone whose origin points at the
// target owner/name. Used to decide between the reuse and fresh-clone paths.
async function isValidCloneOf(
  repoDir: string,
  owner: string,
  name: string,
): Promise<boolean> {
  if (!existsSync(`${repoDir}/.git`)) return false;
  const res = await captureGit(["remote", "get-url", "origin"], repoDir);
  if (!res.success) return false;
  const actual = normalizeGithubRepo(res.stdout);
  return actual === `${owner}/${name}`.toLowerCase();
}

// Reuse an existing clone: fetch the requested ref incrementally, then force the
// working tree to exactly match it (full clean checkout). Avoids re-downloading
// the entire repository on every update.
async function updateRepository(
  params: CloneRepositoryParams,
  event: Electron.IpcMainInvokeEvent,
): Promise<{ success: boolean; error?: string }> {
  const { githubRepoName, tag, branch, commit } = params;

  const qitechControlEnv = process.env.QITECH_CONTROL_ENV;
  const homeDir = qitechControlEnv ? "/home/qitech" : process.env.HOME;
  if (!homeDir) {
    return { success: false, error: terminalError("Home directory not found") };
  }
  const repoDir = `${homeDir}/${githubRepoName}`;

  if (!tag && !branch && !commit) {
    return {
      success: false,
      error: terminalError("No specific version specified!"),
    };
  }

  try {
    event.sender.send(
      UPDATE_LOG,
      terminalInfo(`Reusing existing repository at ${repoDir}`),
    );

    // Clear any stale lock files left by an interrupted git operation.
    removeStaleLockFiles(repoDir);

    // Fetch the requested ref incrementally. Tags are always fetched so tag
    // updates resolve; an explicit ref fetch makes sure single-branch clones
    // still obtain the target branch/commit.
    const fetchArgs = [
      ...gitAuthArgs(),
      "fetch",
      "--force",
      "--tags",
      "origin",
    ];
    if (branch) {
      fetchArgs.push(`${branch}:refs/remotes/origin/${branch}`);
    } else if (commit) {
      fetchArgs.push(commit);
    }
    const fetchRes = await runCommand("git", fetchArgs, repoDir, event);
    if (!fetchRes.success) {
      return {
        success: false,
        error: terminalError("Failed to fetch updates"),
      };
    }

    // Check out the requested ref. `-f` discards any conflicting local state.
    let checkoutArgs: string[];
    if (tag) {
      event.sender.send(UPDATE_LOG, terminalInfo(`Checking out tag: ${tag}`));
      checkoutArgs = [
        "-c",
        "advice.detachedHead=false",
        "checkout",
        "-f",
        `tags/${tag}`,
      ];
    } else if (branch) {
      event.sender.send(
        UPDATE_LOG,
        terminalInfo(`Checking out branch: ${branch}`),
      );
      checkoutArgs = ["checkout", "-f", "-B", branch, `origin/${branch}`];
    } else {
      event.sender.send(
        UPDATE_LOG,
        terminalInfo(`Checking out commit: ${commit}`),
      );
      checkoutArgs = [
        "-c",
        "advice.detachedHead=false",
        "checkout",
        "-f",
        commit as string,
      ];
    }
    const checkoutRes = await runCommand("git", checkoutArgs, repoDir, event);
    if (!checkoutRes.success) {
      return {
        success: false,
        error: terminalError("Failed to checkout target version"),
      };
    }

    // For a branch, fast-forward the working tree to the fetched remote tip.
    if (branch) {
      const resetBranch = await runCommand(
        "git",
        ["reset", "--hard", `origin/${branch}`],
        repoDir,
        event,
      );
      if (!resetBranch.success) {
        return {
          success: false,
          error: terminalError(`Failed to reset to origin/${branch}`),
        };
      }
    }

    // Full clean checkout: wipe all untracked/ignored files so the tree matches
    // a fresh clone.
    const cleanRes = await runCommand("git", ["clean", "-fdx"], repoDir, event);
    if (!cleanRes.success) {
      return {
        success: false,
        error: terminalError("Failed to clean working tree"),
      };
    }

    event.sender.send(
      UPDATE_LOG,
      terminalSuccess("Repository updated successfully"),
    );
    return { success: true, error: undefined };
  } catch (error: any) {
    // runCommand rejects with { success, error } on a non-zero exit; surface that.
    const message =
      typeof error?.error === "string"
        ? error.error
        : (error?.message ?? error?.toString?.() ?? "Unknown error");
    return { success: false, error: terminalError(message) };
  }
}

async function cloneRepository(
  params: CloneRepositoryParams,
  event: Electron.IpcMainInvokeEvent,
): Promise<{ success: boolean; error?: string }> {
  const { githubRepoOwner, githubRepoName, tag, branch, commit } = params;

  const qitechControlEnv = process.env.QITECH_CONTROL_ENV;
  const homeDir = qitechControlEnv ? "/home/qitech" : process.env.HOME;

  if (!homeDir) {
    return { success: false, error: terminalError("Home directory not found") };
  }

  // Construct repository URL
  const repoUrl = `https://github.com/${githubRepoOwner}/${githubRepoName}.git`;

  // Determine clone arguments based on whether tag, branch, or commit is specified.
  // Auth args are prepended so private repos work without storing credentials
  // in the on-disk git config (unlike a token-in-URL approach).
  const cloneArgs = [...gitAuthArgs(), "clone", "--progress", repoUrl];

  if (tag) {
    // Clone a specific tag
    cloneArgs.push("--branch", tag, "--single-branch");
    event.sender.send(UPDATE_LOG, terminalInfo(`Cloning tag: ${tag}`));
  } else if (branch) {
    // Clone a specific branch
    cloneArgs.push("--branch", branch, "--single-branch");
    event.sender.send(UPDATE_LOG, terminalInfo(`Cloning branch: ${branch}`));
  } else if (commit) {
    // For commit, we need to clone first, then checkout the specific commit
    event.sender.send(
      UPDATE_LOG,
      terminalInfo(`Cloning repository, will checkout commit: ${commit}`),
    );
  } else {
    return {
      success: false,
      error: terminalError("No specific version specified!"),
    };
  }

  const cmd1 = await runCommand("git", cloneArgs, homeDir, event);

  if (!cmd1.success) {
    return {
      success: false,
      error: terminalError("Failed to clone repository"),
    };
  }

  // If commit is specified, checkout the specific commit
  if (commit && cmd1.success) {
    const repoDir = `${homeDir}/${githubRepoName}`;
    const cmd2 = await runCommand("git", ["checkout", commit], repoDir, event);

    if (!cmd2.success) {
      return {
        success: false,
        error: terminalError(`Failed to checkout commit: ${commit}`),
      };
    }

    event.sender.send(
      UPDATE_LOG,
      terminalSuccess(`Successfully checked out commit: ${commit}`),
    );
  }
  event.sender.send(
    UPDATE_LOG,
    terminalSuccess("Repository cloned successfully"),
  );
  return { success: true, error: undefined };
}

async function runCommand(
  cmd: string,
  args: string[],
  workingDir: string,
  event: Electron.IpcMainInvokeEvent,
): Promise<{ success: boolean; error?: string }> {
  try {
    // Redact any `-c http.extraheader=...` value so the PAT never appears in
    // the on-screen log stream.
    const redactedArgs = args.map((arg, i) => {
      if (
        i > 0 &&
        args[i - 1] === "-c" &&
        arg.startsWith("http.extraheader=")
      ) {
        return "http.extraheader=***";
      }
      return arg;
    });
    const completeCommand = `${cmd} ${redactedArgs.join(" ")}`;
    const workingDirText = terminalGray(workingDir);
    event.sender.send(
      UPDATE_LOG,
      `🚀 ${workingDirText} ${terminalColor("blue", completeCommand)}`,
    );

    const childProcess = spawn(cmd, args, {
      cwd: workingDir,
    });

    // Store reference to current process for cancellation
    currentUpdateProcess = childProcess;

    // Stream stdout logs back to renderer
    childProcess.stdout.on("data", (data) => {
      const log = data.toString();
      console.log(log);
      event.sender.send(UPDATE_LOG, log);
    });

    // Stream stderr logs back to renderer
    childProcess.stderr.on("data", (data) => {
      const log = data.toString();
      console.error(log);
      event.sender.send(UPDATE_LOG, log);

      // Git outputs progress to stderr
      if (cmd === "git" && args.includes("--progress")) {
        parseGitProgress(log, event);
      }
    });

    // Handle process completion
    return new Promise((resolve, reject) => {
      childProcess.on("close", (code, signal) => {
        // Clear process reference when completed
        if (currentUpdateProcess === childProcess) {
          currentUpdateProcess = null;
        }

        if (signal === "SIGTERM" || signal === "SIGKILL") {
          event.sender.send(UPDATE_LOG, terminalInfo("Command was cancelled"));
          reject({
            success: false,
            error: "Command was cancelled",
          });
        } else if (code === 0) {
          event.sender.send(
            UPDATE_LOG,
            terminalSuccess("Command completed successfully"),
          );
          resolve({ success: true, error: undefined });
        } else {
          event.sender.send(
            UPDATE_LOG,
            terminalError(`Command failed with code ${code}`),
          );
          reject({
            success: false,
            error: terminalError(code?.toString() ?? "NO_CODE"),
          });
        }
      });

      childProcess.on("error", (err) => {
        // Clear process reference on error
        if (currentUpdateProcess === childProcess) {
          currentUpdateProcess = null;
        }

        event.sender.send(
          UPDATE_LOG,
          terminalError(`Command error: ${err.message}`),
        );
        reject({ success: false, error: err.message });
      });
    });
  } catch (error: any) {
    event.sender.send(UPDATE_LOG, terminalError(`Error: ${error.toString()}`));
    return { success: false, error: error.toString() };
  }
}

// Parse Git clone progress
function parseGitProgress(
  log: string,
  event: Electron.IpcMainInvokeEvent,
): void {
  // Git progress format: "Receiving objects: 45% (234/520)"
  // or "Resolving deltas: 100% (150/150)"
  const receivingMatch = log.match(/Receiving objects:\s*(\d+)%/);
  const resolvingMatch = log.match(/Resolving deltas:\s*(\d+)%/);

  if (receivingMatch) {
    const percent = parseInt(receivingMatch[1], 10);
    event.sender.send(UPDATE_STEP, {
      stepId: "clone-repo",
      status: "in-progress",
      progress: Math.floor(percent * 0.8), // Receiving is 80% of clone
    });
  } else if (resolvingMatch) {
    const percent = parseInt(resolvingMatch[1], 10);
    event.sender.send(UPDATE_STEP, {
      stepId: "clone-repo",
      status: "in-progress",
      progress: Math.floor(80 + percent * 0.2), // Resolving is last 20%
    });
  }
}

// Track Rust build progress
let rustBuildProgress = {
  totalDerivations: 0,
  builtDerivations: 0,
  maxPercent: 0, // Track max to prevent backward movement
};

// Parse Rust build output for progress
function parseRustBuildOutput(
  log: string,
  event: Electron.IpcMainInvokeEvent,
): void {
  // Track derivations to build
  const derivationsMatch = log.match(/these (\d+) derivations? will be built/i);
  if (derivationsMatch) {
    rustBuildProgress.totalDerivations = parseInt(derivationsMatch[1], 10);
    rustBuildProgress.builtDerivations = 0;
    rustBuildProgress.maxPercent = 0;
    event.sender.send(UPDATE_STEP, {
      stepId: "rust-build",
      status: "in-progress",
      progress: 0,
    });
    return;
  }

  // Track building packages
  if (
    log.includes("building '/nix/store/") ||
    log.includes("building /nix/store/")
  ) {
    rustBuildProgress.builtDerivations++;

    // Check if this is the server-deps package (one of the last builds)
    const isServerDeps = log.includes("-server-deps");

    let percent = 15;
    if (isServerDeps) {
      // server-deps indicates we're at 85%
      percent = 85;
    } else if (rustBuildProgress.totalDerivations > 0) {
      const derivationProgress =
        rustBuildProgress.builtDerivations / rustBuildProgress.totalDerivations;
      percent = 15 + Math.floor(derivationProgress * 70); // Map to 15-85%
    }

    // Only move forward
    percent = Math.max(percent, rustBuildProgress.maxPercent);
    rustBuildProgress.maxPercent = percent;

    event.sender.send(UPDATE_STEP, {
      stepId: "rust-build",
      status: "in-progress",
      progress: percent,
    });
  }

  // Track installing phase - go up to 90%
  if (log.includes("installing") || log.includes("Installing")) {
    const percent = Math.max(90, rustBuildProgress.maxPercent);
    rustBuildProgress.maxPercent = percent;
    event.sender.send(UPDATE_STEP, {
      stepId: "rust-build",
      status: "in-progress",
      progress: percent,
    });
  }
}

// Enhanced version of runCommand that tracks build steps based on log output
async function runCommandWithStepTracking(
  cmd: string,
  args: string[],
  workingDir: string,
  event: Electron.IpcMainInvokeEvent,
): Promise<{ success: boolean; error?: string }> {
  try {
    const completeCommand = `${cmd} ${args.join(" ")}`;
    const workingDirText = terminalGray(workingDir);
    event.sender.send(
      UPDATE_LOG,
      `🚀 ${workingDirText} ${terminalColor("blue", completeCommand)}`,
    );

    const childProcess = spawn(cmd, args, {
      cwd: workingDir,
    });

    // Store reference to current process for cancellation
    currentUpdateProcess = childProcess;

    // Track which steps have been marked as in-progress
    // Note: rust-build is already marked as in-progress before calling this function
    let systemInstallStarted = false;

    // Function to process log output and update steps
    const processLogForSteps = (log: string) => {
      const logLower = log.toLowerCase();

      // Parse Rust build progress
      parseRustBuildOutput(log, event);

      // Check for system install indicators
      // System install happens after Rust build completes (at ~90%)
      // Look for bootloader/activation messages that indicate final system installation
      if (
        !systemInstallStarted &&
        rustBuildProgress.maxPercent >= 90 &&
        (logLower.includes("updating grub") ||
          logLower.includes("installing bootloader") ||
          logLower.includes("updating bootloader") ||
          logLower.includes("activating the configuration") ||
          logLower.includes("building the system configuration") ||
          logLower.includes("these 0 derivations"))
      ) {
        console.log("🟢 Detected system install start:", log.substring(0, 100));
        // Mark rust as complete, start system install
        event.sender.send(UPDATE_STEP, {
          stepId: "rust-build",
          status: "completed",
        });
        event.sender.send(UPDATE_STEP, {
          stepId: "system-install",
          status: "in-progress",
        });
        systemInstallStarted = true;
      }
    };

    // Stream stdout logs back to renderer
    childProcess.stdout.on("data", (data) => {
      const log = data.toString();
      console.log(log);
      event.sender.send(UPDATE_LOG, log);
      processLogForSteps(log);
    });

    // Stream stderr logs back to renderer
    childProcess.stderr.on("data", (data) => {
      const log = data.toString();
      console.error(log);
      event.sender.send(UPDATE_LOG, log);
      processLogForSteps(log);
    });

    // Handle process completion
    return new Promise((resolve, reject) => {
      childProcess.on("close", (code, signal) => {
        // Clear process reference when completed
        if (currentUpdateProcess === childProcess) {
          currentUpdateProcess = null;
        }

        if (signal === "SIGTERM" || signal === "SIGKILL") {
          event.sender.send(UPDATE_LOG, terminalInfo("Command was cancelled"));
          reject({
            success: false,
            error: "Command was cancelled",
          });
        } else if (code === 0) {
          // Mark all remaining steps as completed on success
          if (!systemInstallStarted) {
            event.sender.send(UPDATE_STEP, {
              stepId: "rust-build",
              status: "completed",
            });
            event.sender.send(UPDATE_STEP, {
              stepId: "system-install",
              status: "completed",
            });
          } else {
            event.sender.send(UPDATE_STEP, {
              stepId: "system-install",
              status: "completed",
            });
          }

          event.sender.send(
            UPDATE_LOG,
            terminalSuccess("Command completed successfully"),
          );
          resolve({ success: true, error: undefined });
        } else {
          event.sender.send(
            UPDATE_LOG,
            terminalError(`Command failed with code ${code}`),
          );
          reject({
            success: false,
            error: terminalError(code?.toString() ?? "NO_CODE"),
          });
        }
      });

      childProcess.on("error", (err) => {
        // Clear process reference on error
        if (currentUpdateProcess === childProcess) {
          currentUpdateProcess = null;
        }

        event.sender.send(
          UPDATE_LOG,
          terminalError(`Command error: ${err.message}`),
        );
        reject({
          success: false,
          error: terminalError(err.message),
        });
      });
    });
  } catch (error: any) {
    event.sender.send(UPDATE_LOG, terminalError(`Error: ${error.toString()}`));
    return { success: false, error: error.toString() };
  }
}

function terminalColor(
  color: "blue" | "green" | "red" | "cyan" | "gray",
  text: string,
): string {
  const colors = {
    blue: "\x1b[34m",
    green: "\x1b[32m",
    red: "\x1b[31m",
    cyan: "\x1b[36m",
    gray: "\x1b[90m",
  };
  return `${colors[color]}${text}\x1b[0m`;
}

function terminalError(text: string): string {
  return terminalColor("red", "💥 " + text);
}

function terminalSuccess(text: string): string {
  return terminalColor("green", "✅ " + text);
}

function terminalInfo(text: string): string {
  return terminalColor("cyan", "📝 " + text);
}

function terminalGray(text: string): string {
  return terminalColor("gray", text);
}
