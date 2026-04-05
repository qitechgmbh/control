import { ipcMain } from "electron";
import {
  UPDATE_CANCEL,
  UPDATE_START,
  UPDATE_END,
  UPDATE_EXECUTE,
  UPDATE_LOG,
  UPDATE_STEP,
} from "./update-channels";
import { existsSync, rmSync } from "fs";
import { Command, run, killWithDescendants, RunOptions } from "../commands";

let homeDir: string | undefined;
let repoDir: string;

let currentCommand: Command | undefined = undefined;
let updating = false;
let canceled = false;

let reportProgress: (report: UpdateProgressReport) => void;
let log: (line: string) => void;

export function addUpdateEventListeners() {
  ipcMain.handle(UPDATE_EXECUTE, async (event, info: UpdateInfo) => {
    await cancel();

    updating = true;
    canceled = false;

    rustBuildProgress = {
      totalDerivations: 0,
      builtDerivations: 0,
      maxPercent: 0,
    };

    systemInstallStarted = false;

    currentCommand = undefined;

    reportProgress = (report) => event.sender.send(UPDATE_STEP, report);
    log = (line) => event.sender.send(UPDATE_LOG, line);

    event.sender.send(UPDATE_START);

    try {
      await update(info);
      log(terminalSuccess("Update completed successfully!"));
    } catch (error: any) {
      log(terminalError(`Update failed: ${error}`));
      throw error;
    } finally {
      event.sender.send(UPDATE_END);
      updating = false;

      // restart the backend if the user canceled or something went wrong
      await runWithTerminal("sudo systemctl start qitech-control-server");
    }
  });

  ipcMain.handle(UPDATE_CANCEL, cancel);
}

async function cancel() {
  if (!updating) {
    return;
  }

  canceled = true;

  log(terminalInfo("Cancelling update..."));

  const pid = currentCommand?.child.pid;
  if (pid) {
    log(terminalInfo(`Killing current update process with pid ${pid}...`));

    try {
      await killWithDescendants(pid);
    } catch {
      // If graceful termination fails, force kill with SIGKILL
      await killWithDescendants(pid, "SIGKILL");
    }
  }
}

async function update(info: UpdateInfo): Promise<void> {
  const qitechControlEnv = process.env.QITECH_CONTROL_ENV;
  homeDir =
    qitechControlEnv === "control-os" ? "/home/qitech" : process.env.HOME;

  if (!homeDir) {
    throw new Error("Home directory not found");
  }

  repoDir = `${homeDir}/${info.githubRepoName}`;

  // 1. first make sure the clone path is empty by deleting it if it contains a .git folder
  await clearRepoDirectory();

  checkCanceled();

  // 2. clone the repository
  try {
    await cloneRepository(info);
  } catch (error) {
    reportProgress({
      stepId: "clone-repo",
      status: "error",
    });
    throw error;
  }

  checkCanceled();

  // 3. make the nixos-install.sh script executable (not tracked in progress UI)
  await runWithTerminal("chmod +x nixos-install.sh", { workingDir: repoDir });

  checkCanceled();

  // 4. stop backend for maximal build capacity (not tracked in progress UI)
  await runWithTerminal("sudo systemctl stop qitech-control-server");

  checkCanceled();

  // 5. run the nixos-install.sh script
  // This script will handle rust-build, electron-build, and system-install
  // Start with rust-build (cargo builds)
  reportProgress({
    stepId: "rust-build",
    status: "in-progress",
  });

  try {
    // We use taskset here to also use the isolated cpu cores.
    await runWithTerminal("taskset --cpu-list 0-3 ./nixos-install.sh", {
      workingDir: repoDir,
      onStdout: parseInstallProgress,
      onStderr: parseInstallProgress,
    });
  } catch (error) {
    reportProgress({
      stepId: "rust-build",
      status: "error",
    });
    reportProgress({
      stepId: "system-install",
      status: "error",
    });
    throw error;
  }
}

// TODO: Instead just fetch and checkout what we need
async function clearRepoDirectory(): Promise<void> {
  // Check if the repo directory exists
  if (existsSync(repoDir)) {
    // If it exists, delete the repo directory
    rmSync(repoDir, { recursive: true, force: true });
    log(terminalSuccess(`Deleted existing repository at ${repoDir}`));
  } else {
    log(
      terminalInfo(
        `No existing repository found at ${repoDir}, nothing to delete`,
      ),
    );
  }
}

async function cloneRepository({
  githubRepoOwner,
  githubRepoName,
  githubToken,
  tag,
  branch,
  commit,
}: UpdateInfo): Promise<void> {
  reportProgress({
    stepId: "clone-repo",
    status: "in-progress",
  });

  // Construct repository URL
  const repoUrl = githubToken
    ? `https://${githubToken}@github.com/${githubRepoOwner}/${githubRepoName}.git`
    : `https://github.com/${githubRepoOwner}/${githubRepoName}.git`;

  // Determine clone arguments based on whether tag, branch, or commit is specified
  let cloneCmd = `git clone --progress ${repoUrl}`;

  if (tag) {
    // Clone a specific tag
    log(terminalInfo(`Cloning tag: ${tag}`));
    cloneCmd += ` --branch ${tag} --single-branch`;
  } else if (branch) {
    // Clone a specific branch
    log(terminalInfo(`Cloning branch: ${branch}`));
    cloneCmd += ` --branch ${branch} --single-branch`;
  } else if (commit) {
    // For commit, we need to clone first, then checkout the specific commit
    log(terminalInfo(`Cloning repository, will checkout commit: ${commit}`));
  } else {
    throw new Error("No specific version specified!");
  }

  // Git outputs progress to stderr
  await runWithTerminal(cloneCmd, {
    workingDir: homeDir,
    onStderr: (line) => {
      const status = parseGitProgress(line);
      reportProgress(status);
    },
  });

  // If commit is specified, checkout the specific commit
  if (commit) {
    const repoDir = `${homeDir}/${githubRepoName}`;
    await runWithTerminal(`git checkout ${commit}`, { workingDir: repoDir });
    log(terminalSuccess(`Successfully checked out commit: ${commit}`));
  }

  log(terminalSuccess("Repository cloned successfully"));
  reportProgress({
    stepId: "clone-repo",
    status: "completed",
  });
}

let systemInstallStarted = false;

function parseInstallProgress(log: string) {
  const logLower = log.toLowerCase();

  if (!systemInstallStarted) {
    // Parse Rust build progress
    const status = parseRustBuildOutput(logLower);
    reportProgress(status);

    // Check for system install indicators
    // System install happens after Rust build completes (at ~90%)
    // Look for bootloader/activation messages that indicate final system installation
    systemInstallStarted =
      rustBuildProgress.maxPercent >= 90 &&
      (logLower.includes("updating grub") ||
        logLower.includes("installing bootloader") ||
        logLower.includes("updating bootloader") ||
        logLower.includes("activating the configuration") ||
        logLower.includes("building the system configuration") ||
        logLower.includes("these 0 derivations"));

    if (systemInstallStarted) {
      reportProgress({
        stepId: "rust-build",
        status: "completed",
      });

      reportProgress({
        stepId: "system-install",
        status: "in-progress",
      });
    }
  }
}

async function runWithTerminal(
  cmd: string,
  { workingDir, onStdout, onStderr }: RunOptions = {},
): Promise<void> {
  log(`🚀 ${terminalGray(workingDir ?? "./")} ${terminalColor("blue", cmd)}`);

  currentCommand = run(cmd, {
    workingDir,
    onStdout: (line) => {
      console.log(line);
      log(line);

      if (onStdout) {
        onStdout(line);
      }
    },

    onStderr: (line) => {
      console.error(line);
      log(line);

      if (onStderr) {
        onStderr(line);
      }
    },
  });

  try {
    await currentCommand.result;
    log(terminalSuccess("Command completed successfully"));
  } finally {
    currentCommand = undefined;
  }
}

// Parse Git clone progress
function parseGitProgress(log: string): UpdateProgressReport {
  // Git progress formats
  // "remote: Counting objects: 45% (234/520)"
  // "remote: Compressing objects: 45% (234/520)"
  // "Receiving objects: 45% (234/520)"
  // "Resolving deltas: 100% (150/150)"

  const countingMatch = log.match(/remote: Counting objects:\s*(\d+)%/);
  if (countingMatch) {
    const percent = parseInt(countingMatch[1], 10);
    return {
      stepId: "clone-repo",
      status: "in-progress",
      progress: Math.floor(percent * 0.1), // Counting is 10% of clone
    };
  }

  const compressingMatch = log.match(/remote: Compressing objects:\s*(\d+)%/);
  if (compressingMatch) {
    const percent = parseInt(compressingMatch[1], 10);
    return {
      stepId: "clone-repo",
      status: "in-progress",
      progress: Math.floor(10 + percent * 0.1), // Counting is 10% of clone
    };
  }

  const receivingMatch = log.match(/Receiving objects:\s*(\d+)%/);
  if (receivingMatch) {
    const percent = parseInt(receivingMatch[1], 10);
    return {
      stepId: "clone-repo",
      status: "in-progress",
      progress: Math.floor(20 + percent * 0.6), // Receiving is 60% of clone
    };
  }

  const resolvingMatch = log.match(/Resolving deltas:\s*(\d+)%/);
  if (resolvingMatch) {
    const percent = parseInt(resolvingMatch[1], 10);
    return {
      stepId: "clone-repo",
      status: "in-progress",
      progress: Math.floor(80 + percent * 0.2), // Resolving is last 20%
    };
  }

  // Percent omitted, since we don't know it
  return {
    stepId: "clone-repo",
    status: "in-progress",
  };
}

// Track Rust build progress
let rustBuildProgress = {
  totalDerivations: 0,
  builtDerivations: 0,
  maxPercent: 0, // Track max to prevent backward movement
};

// Parse Rust build output for progress
// TODO: its totally possible that the new nixos has slightly differnt output as of now
function parseRustBuildOutput(log: string): UpdateProgressReport {
  // Track derivations to build
  const derivationsMatch = log.match(/these (\d+) derivations? will be built/i);
  if (derivationsMatch) {
    rustBuildProgress.totalDerivations = parseInt(derivationsMatch[1], 10);
    rustBuildProgress.builtDerivations = 0;
    rustBuildProgress.maxPercent = 0;

    return {
      stepId: "rust-build",
      status: "in-progress",
      progress: 0,
    };
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

    return {
      stepId: "rust-build",
      status: "in-progress",
      progress: percent,
    };
  }

  // Track installing phase - go up to 90%
  if (log.includes("installing") || log.includes("Installing")) {
    const percent = Math.max(90, rustBuildProgress.maxPercent);
    rustBuildProgress.maxPercent = percent;
    return {
      stepId: "rust-build",
      status: "in-progress",
      progress: percent,
    };
  }

  // Percent omitted, since we don't know it
  return {
    stepId: "rust-build",
    status: "in-progress",
  };
}

function checkCanceled() {
  if (canceled) {
    throw new Error("Update was canceled by user");
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
