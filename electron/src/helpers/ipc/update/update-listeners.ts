import { ipcMain } from "electron";
import {
  UPDATE_CANCEL,
  UPDATE_END,
  UPDATE_EXECUTE,
  UPDATE_LOG,
  UPDATE_STEP,
} from "./update-channels";
import { spawn, ChildProcess } from "child_process";
import tkill from "@jub3i/tree-kill";
import { existsSync, rmSync } from "fs";

type UpdateExecuteListenerParams = {
  githubRepoOwner: string;
  githubRepoName: string;
  githubToken?: string;
  tag?: string;
  branch?: string;
  commit?: string;
};

// Store reference to current update process for cancellation
let currentUpdateProcess: ChildProcess | null = null;

export function addUpdateEventListeners() {
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
        const {
          githubRepoOwner,
          githubRepoName,
          githubToken,
          tag,
          branch,
          commit,
        } = params;

        // Implement your update logic here
        console.log("Update parameters:", {
          githubRepoOwner,
          githubRepoName,
          githubToken,
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

        // 1. first make sure the clone path is empty by deleting it if it containsa .git folder
        const repoDir = `${homeDir}/${githubRepoName}`;

        // Clear repo (not tracked in progress UI)
        const clearResult = await clearRepoDirectory(
          `${homeDir}/${githubRepoName}`,
          event,
        );
        if (!clearResult.success) {
          event.sender.send(UPDATE_LOG, clearResult.error);
          return;
        }

        // 2. clone the repository
        event.sender.send(UPDATE_STEP, {
          stepId: "clone-repo",
          status: "in-progress",
        });
        const cloneResult = await cloneRepository(
          {
            githubRepoOwner,
            githubRepoName,
            githubToken,
            tag,
            branch,
            commit,
          },
          event,
        );
        if (!cloneResult.success) {
          event.sender.send(UPDATE_STEP, {
            stepId: "clone-repo",
            status: "error",
          });
          event.sender.send(UPDATE_LOG, cloneResult.error);
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
            stepId: "electron-build",
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
  githubToken?: string;
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

async function cloneRepository(
  params: CloneRepositoryParams,
  event: Electron.IpcMainInvokeEvent,
): Promise<{ success: boolean; error?: string }> {
  const { githubRepoOwner, githubRepoName, githubToken, tag, branch, commit } =
    params;

  const qitechControlEnv = process.env.QITECH_CONTROL_ENV;
  const homeDir = qitechControlEnv ? "/home/qitech" : process.env.HOME;

  if (!homeDir) {
    return { success: false, error: terminalError("Home directory not found") };
  }

  // Construct repository URL
  const repoUrl = githubToken
    ? `https://${githubToken}@github.com/${githubRepoOwner}/${githubRepoName}.git`
    : `https://github.com/${githubRepoOwner}/${githubRepoName}.git`;

  // Determine clone arguments based on whether tag, branch, or commit is specified
  const cloneArgs = ["clone", repoUrl];

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

    let percent = 15;
    if (rustBuildProgress.totalDerivations > 0) {
      const derivationProgress =
        rustBuildProgress.builtDerivations /
        rustBuildProgress.totalDerivations;
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

  // Track installing phase
  if (log.includes("installing") || log.includes("Installing")) {
    const percent = Math.max(88, rustBuildProgress.maxPercent);
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
    let electronBuildStarted = false;
    let systemInstallStarted = false;

    // Function to process log output and update steps
    const processLogForSteps = (log: string) => {
      const logLower = log.toLowerCase();

      // Parse Rust build progress
      parseRustBuildOutput(log, event);

      // Check for Electron build indicators
      if (
        !electronBuildStarted &&
        (logLower.includes("building electron") ||
          (logLower.includes("npm") &&
            (logLower.includes("build") || logLower.includes("install"))) ||
          (logLower.includes("vite") && logLower.includes("build")))
      ) {
        // Mark rust as complete, start electron
        event.sender.send(UPDATE_STEP, {
          stepId: "rust-build",
          status: "completed",
        });
        event.sender.send(UPDATE_STEP, {
          stepId: "electron-build",
          status: "in-progress",
        });
        electronBuildStarted = true;
      }

      // Check for system install indicators
      if (
        electronBuildStarted &&
        !systemInstallStarted &&
        (logLower.includes("installing") ||
          logLower.includes("nixos-rebuild") ||
          logLower.includes("system install") ||
          logLower.includes("activating"))
      ) {
        // Mark electron as complete, start system install
        event.sender.send(UPDATE_STEP, {
          stepId: "electron-build",
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
          if (!electronBuildStarted) {
            event.sender.send(UPDATE_STEP, {
              stepId: "rust-build",
              status: "completed",
            });
            event.sender.send(UPDATE_STEP, {
              stepId: "electron-build",
              status: "completed",
            });
            event.sender.send(UPDATE_STEP, {
              stepId: "system-install",
              status: "completed",
            });
          } else if (!systemInstallStarted) {
            event.sender.send(UPDATE_STEP, {
              stepId: "electron-build",
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
