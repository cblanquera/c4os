import { spawnSync } from "node:child_process";
import { lstatSync, realpathSync } from "node:fs";
import { dirname, join, relative, resolve, sep } from "node:path";
import process from "node:process";

// Fixed location owned by the deterministic OpenCode reconstruction process.
export const BUILD_OWNED_SOURCE_RELATIVE_PATH =
  "target/c4os-opencode-source/v1.18.3";
export const FIXED_GIT_EXECUTABLE = "/usr/bin/git";

const ISOLATED_GIT_ENVIRONMENT = Object.freeze({
  GIT_CONFIG_GLOBAL: "/dev/null",
  GIT_CONFIG_NOSYSTEM: "1",
  GIT_CONFIG_SYSTEM: "/dev/null",
  GIT_TERMINAL_PROMPT: "0",
  HOME: "/var/empty",
  LANG: "C",
  LC_ALL: "C",
  PATH: "/usr/bin:/bin",
  TMPDIR: "/private/tmp",
  TZ: "UTC",
});

const FIXED_GIT_CONFIGURATION = Object.freeze([
  "-c",
  "credential.helper=",
  "-c",
  "core.fsmonitor=false",
  "-c",
  "core.hooksPath=/dev/null",
  "-c",
  "core.untrackedCache=false",
  "-c",
  "protocol.file.allow=never",
]);

const ALLOWED_LOCAL_GIT_CONFIGURATION = new Set([
  "core.bare",
  "core.filemode",
  "core.hookspath",
  "core.ignorecase",
  "core.logallrefupdates",
  "core.precomposeunicode",
  "core.repositoryformatversion",
  "remote.origin.fetch",
  "remote.origin.url",
]);

/** Reads path metadata without following a final symlink. */
function readPathStatus(path) {
  return lstatSync(path, { throwIfNoEntry: false });
}

/** Rejects ambient Git path/config variables instead of silently honoring them. */
export function assertNoGitRedirectionEnvironment(environment = process.env) {
  const inheritedGitVariables = Object.keys(environment).filter(
    (name) => name.startsWith("GIT_") && name !== "GIT_PAGER",
  );
  if (inheritedGitVariables.length !== 0) {
    throw new Error(
      `OpenCode reconstruction rejects unsafe ambient Git variables: ${inheritedGitVariables
        .sort()
        .join(", ")}`,
    );
  }
}

/** Runs fixed Git with no inherited environment or user/system configuration. */
export function runIsolatedGit(
  args,
  { cwd, encoding = "utf8", stdio = "pipe", acceptedStatuses = [0] } = {},
) {
  const result = spawnSync(
    FIXED_GIT_EXECUTABLE,
    [...FIXED_GIT_CONFIGURATION, ...args],
    {
      cwd,
      encoding,
      env: ISOLATED_GIT_ENVIRONMENT,
      maxBuffer: 64 * 1024 * 1024,
      stdio,
    },
  );
  if (result.error) throw result.error;
  if (!acceptedStatuses.includes(result.status)) {
    throw new Error(
      `${FIXED_GIT_EXECUTABLE} failed (${result.status}): ${String(
        result.stderr,
      ).trim()}`,
    );
  }
  return result;
}

/** Rejects symlinks in every existing component below the canonical repo root. */
function assertNoSymlinkedComponents(
  canonicalRepositoryRoot,
  expectedSourceRoot,
) {
  let currentPath = canonicalRepositoryRoot;
  const components = relative(
    canonicalRepositoryRoot,
    expectedSourceRoot,
  ).split(sep);

  for (const component of components) {
    currentPath = join(currentPath, component);
    const status = readPathStatus(currentPath);
    if (!status) return;
    if (status.isSymbolicLink()) {
      throw new Error(
        `build-owned OpenCode source has a symlinked path component: ${currentPath}`,
      );
    }
  }
}

/** Validates the fixed repository-owned OpenCode source location. */
export function assertSafeBuildOwnedSourcePath(
  repositoryRoot,
  sourceRoot,
  { requireSource = false } = {},
) {
  const canonicalRepositoryRoot = realpathSync(repositoryRoot);
  const expectedSourceRoot = join(
    canonicalRepositoryRoot,
    BUILD_OWNED_SOURCE_RELATIVE_PATH,
  );
  if (resolve(sourceRoot) !== expectedSourceRoot) {
    throw new Error(
      "build-owned OpenCode source must equal the repository-contained canonical path",
    );
  }

  assertNoSymlinkedComponents(canonicalRepositoryRoot, expectedSourceRoot);

  const expectedParent = dirname(expectedSourceRoot);
  const parentStatus = readPathStatus(expectedParent);
  if (parentStatus && realpathSync(expectedParent) !== expectedParent) {
    throw new Error(
      "build-owned OpenCode source parent escaped its repository-contained canonical path",
    );
  }

  const sourceStatus = readPathStatus(expectedSourceRoot);
  if (sourceStatus && realpathSync(expectedSourceRoot) !== expectedSourceRoot) {
    throw new Error(
      "build-owned OpenCode source escaped its repository-contained canonical path",
    );
  }
  if (requireSource && !sourceStatus) {
    throw new Error("build-owned OpenCode source does not exist");
  }

  return {
    canonicalRepositoryRoot,
    canonicalSourceRoot: sourceStatus ? expectedSourceRoot : undefined,
    expectedSourceRoot,
  };
}

/**
 * Verifies that fixed Git resolves exactly the guarded checkout and that the
 * checkout is not redirected through environment, core.worktree, or a linked
 * Git directory.
 */
export function assertCanonicalGitWorktree(canonicalSourceRoot) {
  const sourceRoot = realpathSync(canonicalSourceRoot);
  const gitDirectory = join(sourceRoot, ".git");
  const gitStatus = readPathStatus(gitDirectory);
  if (!gitStatus?.isDirectory() || gitStatus.isSymbolicLink()) {
    throw new Error(
      "OpenCode reconstruction requires a repository-local .git directory",
    );
  }

  const localConfigurationNames = runIsolatedGit(
    ["config", "--local", "--name-only", "--null", "--list"],
    { cwd: sourceRoot, encoding: "buffer" },
  )
    .stdout.toString("utf8")
    .split("\0")
    .filter(Boolean);
  const rejectedConfigurationNames = localConfigurationNames.filter(
    (name) => !ALLOWED_LOCAL_GIT_CONFIGURATION.has(name.toLowerCase()),
  );
  if (rejectedConfigurationNames.length !== 0) {
    throw new Error(
      `OpenCode reconstruction rejects non-allowlisted local Git configuration: ${rejectedConfigurationNames
        .sort()
        .join(", ")}`,
    );
  }

  const topLevel = runIsolatedGit(["rev-parse", "--show-toplevel"], {
    cwd: sourceRoot,
  }).stdout.trim();
  if (realpathSync(topLevel) !== sourceRoot) {
    throw new Error(
      "effective Git worktree does not equal the guarded OpenCode source",
    );
  }

  const insideWorktree = runIsolatedGit(
    ["rev-parse", "--is-inside-work-tree"],
    { cwd: sourceRoot },
  ).stdout.trim();
  if (insideWorktree !== "true") {
    throw new Error("guarded OpenCode source is not a Git worktree");
  }
  const bareRepository = runIsolatedGit(
    ["config", "--local", "--bool", "--get", "core.bare"],
    { cwd: sourceRoot },
  ).stdout.trim();
  if (bareRepository !== "false") {
    throw new Error("guarded OpenCode source must not be a bare repository");
  }

  for (const argument of ["--git-dir", "--git-common-dir"]) {
    const resolvedGitDirectory = resolve(
      sourceRoot,
      runIsolatedGit(["rev-parse", argument], {
        cwd: sourceRoot,
      }).stdout.trim(),
    );
    if (realpathSync(resolvedGitDirectory) !== realpathSync(gitDirectory)) {
      throw new Error(
        "effective Git directory does not equal the guarded repository-local .git directory",
      );
    }
  }

  const coreWorktree = runIsolatedGit(
    ["config", "--local", "--get", "core.worktree"],
    { cwd: sourceRoot, acceptedStatuses: [0, 1] },
  );
  if (coreWorktree.status === 0) {
    throw new Error(
      "OpenCode reconstruction rejects core.worktree redirection",
    );
  }

  return sourceRoot;
}

/** Validates a native-rooted install path without following a final symlink. */
export function assertSafeInstallDestination(nativeRoot, destination) {
  const canonicalNativeRoot = realpathSync(nativeRoot);
  const expectedDestination = resolve(destination);
  const destinationRelative = relative(
    canonicalNativeRoot,
    expectedDestination,
  );
  if (
    destinationRelative.length === 0 ||
    destinationRelative === ".." ||
    destinationRelative.startsWith(`..${sep}`)
  ) {
    throw new Error(
      "OpenCode install destination must remain below the canonical native root",
    );
  }

  const components = destinationRelative.split(sep);
  let currentPath = canonicalNativeRoot;
  for (const component of components.slice(0, -1)) {
    currentPath = join(currentPath, component);
    const status = readPathStatus(currentPath);
    if (!status) break;
    if (status.isSymbolicLink()) {
      throw new Error(
        `OpenCode install parent contains a symlinked component: ${currentPath}`,
      );
    }
    if (!status.isDirectory()) {
      throw new Error(
        `OpenCode install parent component is not a directory: ${currentPath}`,
      );
    }
  }

  const parent = dirname(expectedDestination);
  const parentStatus = readPathStatus(parent);
  let canonicalParent;
  if (parentStatus) {
    if (!parentStatus.isDirectory() || parentStatus.isSymbolicLink()) {
      throw new Error("OpenCode install parent must be a real directory");
    }
    canonicalParent = realpathSync(parent);
    const canonicalParentRelative = relative(
      canonicalNativeRoot,
      canonicalParent,
    );
    if (
      canonicalParentRelative === ".." ||
      canonicalParentRelative.startsWith(`..${sep}`)
    ) {
      throw new Error(
        "OpenCode install parent escaped the canonical native root",
      );
    }
  }

  const destinationStatus = readPathStatus(expectedDestination);
  if (destinationStatus?.isSymbolicLink()) {
    throw new Error("OpenCode install destination must not be a symlink");
  }
  if (destinationStatus && !destinationStatus.isFile()) {
    throw new Error("OpenCode install destination must be a regular file");
  }

  return {
    canonicalNativeRoot,
    canonicalParent,
    destinationStatus,
    expectedDestination,
  };
}
