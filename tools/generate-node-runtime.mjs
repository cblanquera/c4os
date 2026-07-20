import { Buffer } from "node:buffer";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import {
  chmod,
  copyFile,
  mkdir,
  open,
  realpath,
  rename,
  stat,
} from "node:fs/promises";
import { dirname, isAbsolute, resolve } from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";

const repositoryRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const outputDirectory = resolve(repositoryRoot, "target/c4os-runtime-assets");
const outputPath = resolve(outputDirectory, "node-runtime.json");
const temporaryPath = `${outputPath}.tmp`;
const bundledExecutablePath = resolve(outputDirectory, "node");
const temporaryExecutablePath = `${bundledExecutablePath}.tmp`;
const PINNED_NODE_VERSION = "26.2.0";
const PINNED_NODE_SHA256 =
  "sha256:b276251704734604aad4ab2dc4a07892565baea39400f6422abeb1fe39637440";
const configuredExecutable =
  process.env.C4OS_BUNDLED_NODE_EXECUTABLE ??
  (process.env.NVM_BIN ? resolve(process.env.NVM_BIN, "node") : undefined);
if (!configuredExecutable || !isAbsolute(configuredExecutable)) {
  throw new Error(
    "Set C4OS_BUNDLED_NODE_EXECUTABLE or NVM_BIN to the pinned standalone Node runtime.",
  );
}
const executable = await realpath(configuredExecutable);
const metadata = await stat(executable);

if (!isAbsolute(executable) || !metadata.isFile() || metadata.size === 0) {
  throw new Error(
    "The local-development Node runtime is not a regular executable.",
  );
}

const handle = await open(executable, "r");
const digest = createHash("sha256");

try {
  for await (const chunk of handle.readableWebStream()) {
    digest.update(Buffer.from(chunk));
  }
} finally {
  await handle.close();
}

const executableSha256 = `sha256:${digest.digest("hex")}`;
const executableVersion = execFileSync(executable, ["--version"], {
  encoding: "utf8",
  env: {},
})
  .trim()
  .replace(/^v/u, "");
const linkedLibraries = execFileSync("/usr/bin/otool", ["-L", executable], {
  encoding: "utf8",
  env: {},
})
  .split("\n")
  .slice(1)
  .map((line) => line.trim().split(" ")[0])
  .filter(Boolean);
if (
  executableVersion !== PINNED_NODE_VERSION ||
  executableSha256 !== PINNED_NODE_SHA256 ||
  linkedLibraries.some(
    (library) =>
      !library.startsWith("/System/") && !library.startsWith("/usr/lib/"),
  )
) {
  throw new Error(
    "The bundled Node candidate is not the pinned standalone macOS arm64 runtime.",
  );
}

const record = {
  schemaVersion: 1,
  executableRelativePath: "sidecars/node",
  version: PINNED_NODE_VERSION,
  minimumSupportedVersion: "22.19.0",
  sha256: PINNED_NODE_SHA256,
};

await mkdir(outputDirectory, { recursive: true, mode: 0o700 });
await copyFile(executable, temporaryExecutablePath);
await chmod(temporaryExecutablePath, 0o755);
await rename(temporaryExecutablePath, bundledExecutablePath);
const output = await open(temporaryPath, "w", 0o600);

try {
  await output.writeFile(`${JSON.stringify(record, null, 2)}\n`, "utf8");
  await output.sync();
} finally {
  await output.close();
}

await rename(temporaryPath, outputPath);
