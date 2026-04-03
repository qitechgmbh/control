import { spawn } from "child_process";

export type CommandResult = {
  statusCode: number;
  stdout: string;
  stderr: string;
};

export function run(command: string): Promise<CommandResult> {
  return new Promise((resolve, reject) => {
    const process = spawn("sh", ["-c", command]);

    let stdout = "";
    let stderr = "";

    process.stdout?.on("data", (data) => {
      stdout += data.toString();
    });

    process.stderr?.on("data", (data) => {
      stderr += data.toString();
    });

    const onExit = (code: number) => {
      if (code === 0) {
        resolve({
          statusCode: code,
          stdout,
          stderr,
        });
      } else {
        reject(new Error(stderr || `Process exited with code ${code}`));
      }
    };

    process.on("exit", onExit);
    process.on("close", onExit);

    process.on("error", reject);
  });
}
