import { copyFileSync, existsSync, statSync } from "node:fs";
import { spawn, spawnSync } from "node:child_process";
import { resolve } from "node:path";

const root = resolve(new URL("..", import.meta.url).pathname);
const args = process.argv.slice(2);
const qaIndex = args.indexOf("--qa");
const qa = qaIndex !== -1;
if (qa) args.splice(qaIndex, 1);

const cargoArgs = ["build", "--manifest-path", "backend/Cargo.toml"];
const build = spawnSync("cargo", cargoArgs, {
  cwd: root,
  env: process.env,
  stdio: "inherit"
});

if (build.status !== 0) process.exit(build.status || 1);
if (build.error) {
  console.error(build.error.message);
  process.exit(1);
}

const debugBinary = resolve(root, "backend/target/debug/c4os-backend");
const appBinary = resolve(root, "backend/target/debug/C4OS.app/Contents/MacOS/C4OS");
const executable = syncDebugAppWrapper(debugBinary, appBinary);
const env = {
  ...process.env,
  ...(qa && !process.env.C4OS_SESSION_STORE
    ? { C4OS_SESSION_STORE: "/tmp/c4os-task-011-qa-sessions.json" }
    : {})
};

const child = spawn(executable, args, {
  cwd: root,
  env,
  stdio: "inherit"
});

child.on("exit", (code, signal) => {
  if (signal) {
    process.kill(process.pid, signal);
    return;
  }
  process.exit(code ?? 0);
});

child.on("error", (error) => {
  console.error(error.message);
  process.exit(1);
});

function syncDebugAppWrapper(source, target) {
  if (!existsSync(source)) return source;
  if (!existsSync(target)) return source;

  const sourceStat = statSync(source);
  const targetStat = statSync(target);
  if (sourceStat.mtimeMs > targetStat.mtimeMs || sourceStat.size !== targetStat.size) {
    copyFileSync(source, target);
  }
  return target;
}
