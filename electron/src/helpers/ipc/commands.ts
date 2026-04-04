import { ChildProcess, spawn } from "child_process";

export type CommandResult = {
  statusCode: number;
  stdout: string;
  stderr: string;
};

export type Command = {
  child: ChildProcess;
  result: Promise<CommandResult>;
};

export type RunOptions = {
  onStdout?: (line: string) => void;
  onStderr?: (line: string) => void;
  workingDir?: string;
};

export function run(
  command: string,
  { onStdout, onStderr, workingDir }: RunOptions = {},
): Command {
  const child = spawn("sh", ["-c", command], { cwd: workingDir });

  const result: Promise<CommandResult> = new Promise((resolve, reject) => {
    const stdout = new LineBuffer();
    const stderr = new LineBuffer();

    stdout.onLine = onStdout;
    stderr.onLine = onStderr;

    child.stdout?.on("data", (data) => stdout.onData(data));
    child.stderr?.on("data", (data) => stderr.onData(data));

    const onExit = (code: number) => {
      stdout.onClose();
      stderr.onClose();

      if (code === 0) {
        resolve({
          statusCode: code,
          stdout: stdout.total,
          stderr: stderr.total,
        });
      } else {
        reject(new Error(stderr.total || `Process exited with code ${code}`));
      }
    };

    child.on("exit", onExit);
    child.on("close", onExit);

    child.on("error", reject);
  });

  return { child, result };
}

class LineBuffer {
  total = "";
  line?: string = undefined;
  onLine?: (line: string) => void;

  onData(data: any) {
    const datas: string = data.toString();
    this.total += datas;

    if (this.onLine) {
      const split = datas.split(/\n|\r/);

      if (split.length > 2) {
        this.line = (this.line ?? "") + split.shift();
        this.onLine(this.line);
      }

      while (split.length > 2) {
        this.onLine(split.shift()!);
      }

      this.line = split[0] || undefined;
    }
  }

  onClose() {
    if (this.line !== undefined && this.onLine) {
      this.onLine(this.line);
    }
  }
}

export async function killWithDescendants(
  pid: number,
  signal: NodeJS.Signals = "SIGTERM",
): Promise<void> {
  const stipped = signal.replace("SIG", "");
  await run(`kill -s ${stipped} $(ps -s ${pid} -o pid=)`).result;
}
