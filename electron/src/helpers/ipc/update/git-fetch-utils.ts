import { spawn } from "node:child_process";
import { existsSync, readdirSync, rmSync } from "node:fs";
import path from "node:path";
import { gitAuthArgs } from "./token-store";

export type GitRefInfo = {
  hash: string;
  name: string;
  date: string;
};

export type RepoImportResult = {
  tags: GitRefInfo[];
  commits: GitRefInfo[];
  branches: GitRefInfo[];
};

export function runGitCommand(
  args: string[],
  cwd?: string,
): Promise<{ stdout: string; stderr: string }> {
  return new Promise((resolve, reject) => {
    // Prepend auth args so private repos authenticate over HTTPS without
    // storing credentials in the mirror clone's on-disk git config.
    const child = spawn("git", [...gitAuthArgs(), ...args], {
      cwd,
      stdio: ["ignore", "pipe", "pipe"],
      env: {
        ...process.env,
        GIT_TERMINAL_PROMPT: "0",
      },
    });

    let stdout = "";
    let stderr = "";

    child.stdout.on("data", (data: { toString: () => string }) => {
      stdout += data.toString();
    });

    child.stderr.on("data", (data: { toString: () => string }) => {
      stderr += data.toString();
    });

    child.on("error", (err: any) => {
      reject(err);
    });

    child.on("close", (code: number, signal: string) => {
      if (signal === "SIGTERM" || signal === "SIGKILL") {
        reject(new Error("Git command cancelled"));
        return;
      }

      if (code === 0) {
        resolve({ stdout, stderr });
      } else {
        reject(
          new Error(
            `Git command failed (${args.join(" ")}):\n${stderr || stdout}`,
          ),
        );
        return;
      }
    });
  });
}

function getDir(owner: string, name: string) {
  const tmpDir =
    process.env.TMPDIR || process.env.TMP || process.env.TEMP || "/tmp";

  return `${tmpDir}/${owner}/${name}`;
}

async function importIfNotExists(owner: string, name: string) {
  const dir = path.resolve(getDir(owner, name));
  const url = `https://github.com/${owner}/${name}.git`;

  if (!existsSync(dir)) {
    await runGitCommand(["clone", "--mirror", "--filter=blob:none", url, dir]);
  }
}

function removeStaleLockFiles(repoPath: string): void {
  function walk(dir: string): void {
    try {
      for (const entry of readdirSync(dir, { withFileTypes: true })) {
        const full = path.join(dir, entry.name);
        if (entry.isDirectory()) {
          walk(full);
        } else if (entry.name.endsWith(".lock")) {
          try {
            rmSync(full, { force: true });
          } catch (err) {
            console.error("Failed to remove stale lock file:", err);
          }
        }
      }
    } catch (err) {
      console.error("Failed to walk .git directory for lock cleanup:", err);
    }
  }
  walk(path.join(repoPath, ".git"));
}

async function fetchWithRetry(repoPath: string): Promise<void> {
  const maxRetries = 3;
  for (let attempt = 0; attempt < maxRetries; attempt++) {
    try {
      await runGitCommand(["fetch", "--prune", "--force", "origin"], repoPath);
      return;
    } catch (error: any) {
      const msg = error?.message || String(error);
      if (attempt < maxRetries - 1 && /cannot lock ref/i.test(msg)) {
        removeStaleLockFiles(repoPath);
        try {
          await runGitCommand(["pack-refs", "--all"], repoPath);
        } catch (err) {
          console.error("Failed to run git pack-refs:", err);
        }
        await new Promise((r) => setTimeout(r, 1000 * (attempt + 1)));
        continue;
      }
      throw error;
    }
  }
}

export async function fetchTargets(
  owner: string,
  name: string,
): Promise<RepoImportResult> {
  const repoPath = path.resolve(getDir(owner, name));

  try {
    await importIfNotExists(owner, name);

    // fetch updates — clean stale locks first, then fetch with retry on transient lock errors
    removeStaleLockFiles(repoPath);
    await fetchWithRetry(repoPath);

    // retrieve last 1000 commits from master branch
    const commitsRes = await runGitCommand(
      [
        "log",
        "master",
        "--max-count=1000",
        '--pretty=format:"%H|%ad|%f"',
        "--date=iso",
      ],
      repoPath,
    );

    const commits: GitRefInfo[] = commitsRes.stdout
      .split("\n")
      .filter(Boolean)
      .map((line) => {
        const [hash, date, name] = line.split("|");
        return { hash, name, date };
      })
      .sort((a, b) => Date.parse(b.date) - Date.parse(a.date));

    // retrieve all branches
    const branchesRes = await runGitCommand(
      [
        "for-each-ref",
        "refs/heads",
        "--format=%(objectname)|%(refname:short)|%(committerdate:iso8601)",
      ],
      repoPath,
    );

    const branches: GitRefInfo[] = branchesRes.stdout
      .split("\n")
      .filter(Boolean)
      .map((line) => {
        const [hash, name, date] = line.split("|");
        return { hash, name, date };
      })
      .sort((a, b) => Date.parse(b.date) - Date.parse(a.date));

    // retrieve all tags
    const tagsRes = await runGitCommand(
      [
        "for-each-ref",
        "refs/tags",
        '--format="%(objectname)|%(refname:short)|%(creatordate:iso8601)"',
      ],
      repoPath,
    );

    const tags: GitRefInfo[] = tagsRes.stdout
      .split("\n")
      .filter(Boolean)
      .map((line) => {
        const [hash, name, date] = line.split("|");
        return { hash, name, date };
      })
      .sort((a, b) =>
        b.name.localeCompare(a.name, undefined, {
          numeric: true,
          sensitivity: "base",
        }),
      );

    return {
      commits,
      branches,
      tags,
    };
  } catch (error: any) {
    throw new Error(error?.message || String(error));
  }
}

export async function fetchChangelog(
  owner: string,
  name: string,
  ref: string,
): Promise<string> {
  try {
    await importIfNotExists(owner, name);
    const repoPath = path.resolve(getDir(owner, name));

    const result = await runGitCommand(
      ["show", `${ref}:CHANGELOG.md`],
      repoPath,
    );

    return result.stdout;
  } catch (error: any) {
    return `${error}`;
  }
}
