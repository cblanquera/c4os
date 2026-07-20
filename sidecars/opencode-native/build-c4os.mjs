import { spawnSync } from "node:child_process";
import { Buffer } from "node:buffer";
import { createHash } from "node:crypto";
import {
  chmodSync,
  copyFileSync,
  existsSync,
  mkdtempSync,
  mkdirSync,
  readFileSync,
  realpathSync,
  renameSync,
  rmSync,
  statSync,
  writeFileSync,
} from "node:fs";
import { gunzipSync } from "node:zlib";
import { dirname, isAbsolute, join, resolve } from "node:path";
import { tmpdir } from "node:os";
import process from "node:process";
import { fileURLToPath, pathToFileURL } from "node:url";

const nativeRoot = realpathSync(dirname(fileURLToPath(import.meta.url)));
const manifest = JSON.parse(
  readFileSync(join(nativeRoot, "c4os-build.json"), "utf8"),
);
const forceRebuild = process.argv.slice(2).includes("--force");

function option(name, fallback) {
  const prefix = `--${name}=`;
  const argument = process.argv
    .slice(2)
    .find((value) => value.startsWith(prefix));
  const value = argument?.slice(prefix.length) ?? fallback;
  if (!value) throw new Error(`missing ${prefix}<absolute-path>`);
  if (!isAbsolute(value)) throw new Error(`${name} must be absolute`);
  return value;
}

function sha256Bytes(bytes) {
  return `sha256:${createHash("sha256").update(bytes).digest("hex")}`;
}

function sha256File(path) {
  return sha256Bytes(readFileSync(path));
}

const pathSafetyPath = join(nativeRoot, "build-path-safety.mjs");
if (
  sha256File(pathSafetyPath) !==
  "sha256:a92ab20759dc8fe8efa36bcaea9eed1c10630a856457ce3bc192d4b36c11aa8c"
) {
  throw new Error("unexpected OpenCode build-path safety helper digest");
}
const {
  assertCanonicalGitWorktree,
  assertNoGitRedirectionEnvironment,
  assertSafeBuildOwnedSourcePath,
  assertSafeInstallDestination,
  BUILD_OWNED_SOURCE_RELATIVE_PATH,
  runIsolatedGit,
} = await import(pathToFileURL(pathSafetyPath).href);

assertNoGitRedirectionEnvironment();

function run(command, args, options = {}) {
  const result = spawnSync(command, args, {
    cwd: options.cwd,
    env: options.env ?? process.env,
    encoding: options.encoding ?? "utf8",
    stdio: options.stdio ?? "pipe",
    maxBuffer: 64 * 1024 * 1024,
  });
  if (result.error) throw result.error;
  if (result.status !== 0) {
    throw new Error(
      `${command} failed (${result.status}): ${String(result.stderr).trim()}`,
    );
  }
  return result.stdout;
}

function runGit(args, options = {}) {
  const result = runIsolatedGit(args, options);
  return result.stdout;
}

function runDestructiveGit(canonicalSourceRoot, args, options = {}) {
  if (!["checkout", "clean", "reset"].includes(args[0])) {
    throw new Error("unexpected destructive Git operation");
  }
  assertSafeBuildOwnedSourcePath(repositoryRoot, canonicalSourceRoot, {
    requireSource: true,
  });
  assertCanonicalGitWorktree(canonicalSourceRoot);
  return runGit(args, { ...options, cwd: canonicalSourceRoot });
}

const repositoryRoot = realpathSync(resolve(nativeRoot, "../.."));
const buildOwnedSourceRoot = join(
  repositoryRoot,
  BUILD_OWNED_SOURCE_RELATIVE_PATH,
);
const sourceRoot = option("source-root", buildOwnedSourceRoot);
const usesBuildOwnedSourceRoot = resolve(sourceRoot) === buildOwnedSourceRoot;
if (forceRebuild && !usesBuildOwnedSourceRoot) {
  throw new Error("forced reconstruction requires the build-owned source root");
}
if (
  process.version !== `v${manifest.toolchain.nodeVersion}` ||
  sha256File(realpathSync(process.execPath)) !==
    manifest.toolchain.nodeDarwinArm64Sha256
) {
  throw new Error("unexpected Node executable or version");
}
const bun = realpathSync(
  option("bun", join(repositoryRoot, "node_modules/.bin/bun")),
);
const temporary = mkdtempSync(join(tmpdir(), "c4os-opencode-build-"));
let installTemporary;

try {
  const installed = resolve(nativeRoot, manifest.artifact.relativeInstallPath);
  assertSafeInstallDestination(nativeRoot, installed);
  if (
    !forceRebuild &&
    existsSync(installed) &&
    statSync(installed).size === manifest.artifact.sizeBytes &&
    sha256File(installed) === manifest.artifact.sha256 &&
    run(installed, ["--version"]).trim() === manifest.upstream.nativeVersion
  ) {
    process.stdout.write(
      `verified ${manifest.upstream.nativeVersion}+${manifest.buildFlavor} ${manifest.artifact.sha256}\n`,
    );
    process.exitCode = 0;
  } else {
    if (usesBuildOwnedSourceRoot) {
      assertSafeBuildOwnedSourcePath(repositoryRoot, sourceRoot, {
        requireSource: existsSync(sourceRoot),
      });
    }
    if (!existsSync(sourceRoot)) {
      if (!usesBuildOwnedSourceRoot) {
        throw new Error("automatic clone requires the build-owned source root");
      }
      assertSafeBuildOwnedSourcePath(repositoryRoot, sourceRoot);
      mkdirSync(dirname(sourceRoot), { recursive: true });
      assertSafeBuildOwnedSourcePath(repositoryRoot, sourceRoot);
      runGit(
        [
          "clone",
          "--depth",
          "1",
          "--branch",
          manifest.upstream.tag,
          manifest.upstream.repository,
          sourceRoot,
        ],
        { stdio: "inherit" },
      );
      assertSafeBuildOwnedSourcePath(repositoryRoot, sourceRoot, {
        requireSource: true,
      });
    }
    const canonicalSourceRoot = realpathSync(sourceRoot);
    if (usesBuildOwnedSourceRoot) {
      assertSafeBuildOwnedSourcePath(repositoryRoot, canonicalSourceRoot, {
        requireSource: true,
      });
    }
    assertCanonicalGitWorktree(canonicalSourceRoot);
    const upstreamCommit = runGit(["rev-parse", "HEAD"], {
      cwd: canonicalSourceRoot,
    }).trim();
    if (upstreamCommit !== manifest.upstream.commit) {
      throw new Error("unexpected OpenCode source commit");
    }
    if (run(bun, ["--version"]).trim() !== manifest.toolchain.bunVersion) {
      throw new Error("unexpected Bun version");
    }
    if (sha256File(bun) !== manifest.toolchain.bunDarwinArm64Sha256) {
      throw new Error("unexpected Bun executable digest");
    }

    const patch = join(nativeRoot, manifest.patch.path);
    if (sha256File(patch) !== manifest.patch.sha256) {
      throw new Error("unexpected C4OS OpenCode patch digest");
    }
    const bunPath = dirname(bun);
    const deterministicEnvironment = {
      PATH: `${bunPath}:${dirname(realpathSync(process.execPath))}:/usr/bin:/bin:/usr/sbin:/sbin`,
      TMPDIR: "/private/tmp",
      LANG: "C",
      LC_ALL: "C",
      TZ: "UTC",
      SOURCE_DATE_EPOCH: runGit(["show", "-s", "--format=%ct", "HEAD"], {
        cwd: canonicalSourceRoot,
      }).trim(),
    };

    if (forceRebuild) {
      assertSafeBuildOwnedSourcePath(repositoryRoot, canonicalSourceRoot, {
        requireSource: true,
      });
      runDestructiveGit(
        canonicalSourceRoot,
        ["reset", "--hard", manifest.upstream.commit],
        { stdio: "inherit" },
      );
      runDestructiveGit(canonicalSourceRoot, ["clean", "-ffdx"], {
        stdio: "inherit",
      });
    }

    const currentDiff = runGit(["diff", "--no-ext-diff", "--no-textconv"], {
      cwd: canonicalSourceRoot,
    });
    if (
      currentDiff.length !== 0 &&
      sha256Bytes(Buffer.from(currentDiff)) !== manifest.patch.sha256
    ) {
      throw new Error(
        "OpenCode source has changes outside the pinned C4OS patch",
      );
    }
    const untracked = runGit(
      ["status", "--porcelain=v1", "--untracked-files=all"],
      { cwd: canonicalSourceRoot },
    )
      .split("\n")
      .filter((line) => line.startsWith("?? "));
    if (untracked.length !== 0) {
      throw new Error("OpenCode source has unexpected untracked build inputs");
    }
    if (
      forceRebuild ||
      !existsSync(join(canonicalSourceRoot, "node_modules/.bun"))
    ) {
      run(
        bun,
        [
          "install",
          "--frozen-lockfile",
          `--cache-dir=${join(repositoryRoot, "target/c4os-bun-cache")}`,
        ],
        {
          cwd: canonicalSourceRoot,
          env: deterministicEnvironment,
          stdio: "inherit",
        },
      );
    }

    if (currentDiff.length === 0) {
      runGit(["apply", "--check", patch], { cwd: canonicalSourceRoot });
      runGit(["apply", patch], { cwd: canonicalSourceRoot });
    } else if (
      sha256Bytes(Buffer.from(currentDiff)) !== manifest.patch.sha256
    ) {
      throw new Error(
        "OpenCode source has changes outside the pinned C4OS patch",
      );
    }
    if (
      sha256Bytes(
        Buffer.from(
          runGit(["diff", "--no-ext-diff", "--no-textconv"], {
            cwd: canonicalSourceRoot,
          }),
        ),
      ) !== manifest.patch.sha256
    ) {
      throw new Error(
        "applied OpenCode patch does not match its pinned digest",
      );
    }

    const modelsGzip = join(nativeRoot, manifest.modelsSnapshot.path);
    if (sha256File(modelsGzip) !== manifest.modelsSnapshot.gzipSha256) {
      throw new Error("unexpected models.dev gzip digest");
    }
    const models = gunzipSync(readFileSync(modelsGzip));
    if (sha256Bytes(models) !== manifest.modelsSnapshot.jsonSha256) {
      throw new Error("unexpected models.dev JSON digest");
    }
    const modelsPath = join(temporary, "models-dev-api.json");
    writeFileSync(modelsPath, models, { mode: 0o600 });
    models.fill(0);

    run(bun, [manifest.build.script, ...manifest.build.arguments], {
      cwd: canonicalSourceRoot,
      env: {
        ...deterministicEnvironment,
        ...manifest.build.environment,
        MODELS_DEV_API_JSON: modelsPath,
      },
      stdio: "inherit",
    });

    const built = resolve(
      canonicalSourceRoot,
      manifest.artifact.relativeBuildPath,
    );
    if (
      sha256File(built) !== manifest.artifact.sha256 ||
      statSync(built).size !== manifest.artifact.sizeBytes ||
      run(built, ["--version"]).trim() !== manifest.upstream.nativeVersion
    ) {
      throw new Error("downstream OpenCode artifact does not match its pins");
    }
    assertSafeInstallDestination(nativeRoot, installed);
    mkdirSync(dirname(installed), { recursive: true });
    const installValidation = assertSafeInstallDestination(
      nativeRoot,
      installed,
    );
    if (!installValidation.canonicalParent) {
      throw new Error("OpenCode install parent was not materialized");
    }
    installTemporary = mkdtempSync(
      join(installValidation.canonicalParent, ".c4os-install-"),
    );
    const stagedInstall = join(installTemporary, "opencode");
    copyFileSync(built, stagedInstall);
    chmodSync(stagedInstall, 0o755);
    if (
      sha256File(stagedInstall) !== manifest.artifact.sha256 ||
      statSync(stagedInstall).size !== manifest.artifact.sizeBytes ||
      run(stagedInstall, ["--version"]).trim() !==
        manifest.upstream.nativeVersion
    ) {
      throw new Error(
        "staged downstream OpenCode artifact failed verification",
      );
    }
    assertSafeInstallDestination(nativeRoot, installed);
    renameSync(stagedInstall, installed);
    assertSafeInstallDestination(nativeRoot, installed);
    if (
      sha256File(installed) !== manifest.artifact.sha256 ||
      statSync(installed).size !== manifest.artifact.sizeBytes
    ) {
      throw new Error(
        "installed downstream OpenCode artifact failed verification",
      );
    }
    process.stdout.write(
      `installed ${manifest.upstream.nativeVersion}+${manifest.buildFlavor} ${manifest.artifact.sha256}\n`,
    );
  }
} finally {
  if (installTemporary) {
    rmSync(installTemporary, { recursive: true, force: true });
  }
  rmSync(temporary, { recursive: true, force: true });
}
