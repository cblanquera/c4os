import assert from "node:assert/strict";
import {
  lstatSync,
  mkdtempSync,
  mkdirSync,
  realpathSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import test from "node:test";
import { join } from "node:path";
import { tmpdir } from "node:os";

import {
  assertCanonicalGitWorktree,
  assertNoGitRedirectionEnvironment,
  assertSafeBuildOwnedSourcePath,
  assertSafeInstallDestination,
  BUILD_OWNED_SOURCE_RELATIVE_PATH,
  FIXED_GIT_EXECUTABLE,
  runIsolatedGit,
} from "../build-path-safety.mjs";

/** Creates an isolated canonical repository path for one validation case. */
function makeRepository(t) {
  const temporaryRoot = mkdtempSync(join(tmpdir(), "c4os-build-path-test-"));
  const repositoryPath = join(temporaryRoot, "repository");
  mkdirSync(repositoryPath);
  const repositoryRoot = realpathSync(repositoryPath);
  t.after(() => rmSync(temporaryRoot, { recursive: true, force: true }));
  return { repositoryRoot, temporaryRoot };
}

for (const symlinkedComponent of [
  ".build",
  ".build/app",
  ".build/app/c4os-opencode-source",
  BUILD_OWNED_SOURCE_RELATIVE_PATH,
]) {
  test(`rejects symlinked ${symlinkedComponent} before a destructive action`, (t) => {
    const { repositoryRoot, temporaryRoot } = makeRepository(t);
    const expectedSourceRoot = join(
      repositoryRoot,
      BUILD_OWNED_SOURCE_RELATIVE_PATH,
    );
    const componentPath = join(repositoryRoot, symlinkedComponent);
    const externalPath = join(temporaryRoot, "external");
    mkdirSync(externalPath);
    mkdirSync(join(componentPath, ".."), { recursive: true });
    symlinkSync(externalPath, componentPath);

    let destructiveActionRan = false;
    assert.throws(() => {
      assertSafeBuildOwnedSourcePath(repositoryRoot, expectedSourceRoot);
      destructiveActionRan = true;
    }, /symlinked path component/);
    assert.equal(destructiveActionRan, false);
  });
}

test("requires the exact repository-contained build source path", (t) => {
  const { repositoryRoot, temporaryRoot } = makeRepository(t);
  const externalSourceRoot = join(temporaryRoot, "external-source");
  mkdirSync(externalSourceRoot);

  assert.throws(
    () =>
      assertSafeBuildOwnedSourcePath(repositoryRoot, externalSourceRoot, {
        requireSource: true,
      }),
    /must equal the repository-contained canonical path/,
  );
});

test("accepts the exact canonical build parent and source", (t) => {
  const { repositoryRoot } = makeRepository(t);
  const expectedSourceRoot = join(
    repositoryRoot,
    BUILD_OWNED_SOURCE_RELATIVE_PATH,
  );
  mkdirSync(expectedSourceRoot, { recursive: true });

  const validated = assertSafeBuildOwnedSourcePath(
    repositoryRoot,
    expectedSourceRoot,
    { requireSource: true },
  );

  assert.equal(validated.canonicalRepositoryRoot, repositoryRoot);
  assert.equal(validated.canonicalSourceRoot, expectedSourceRoot);
  assert.equal(validated.expectedSourceRoot, expectedSourceRoot);
});

test("rejects ambient Git environment redirection and configuration", () => {
  for (const name of [
    "GIT_DIR",
    "GIT_WORK_TREE",
    "GIT_COMMON_DIR",
    "GIT_CONFIG_COUNT",
    "GIT_CONFIG_GLOBAL",
  ]) {
    assert.throws(
      () => assertNoGitRedirectionEnvironment({ [name]: "/tmp/redirected" }),
      new RegExp(name),
    );
  }
  assert.doesNotThrow(() =>
    assertNoGitRedirectionEnvironment({
      GIT_PAGER: "cat",
      PATH: "/host/path",
    }),
  );
});

test("fixed isolated Git validates only a repository-local worktree", (t) => {
  const { repositoryRoot, temporaryRoot } = makeRepository(t);
  const sourceRoot = join(repositoryRoot, "source");
  mkdirSync(sourceRoot);
  runIsolatedGit(["init", "--quiet"], { cwd: sourceRoot });

  assert.equal(FIXED_GIT_EXECUTABLE, "/usr/bin/git");
  assert.equal(assertCanonicalGitWorktree(sourceRoot), sourceRoot);

  const redirectedWorktree = join(temporaryRoot, "redirected-worktree");
  mkdirSync(redirectedWorktree);
  runIsolatedGit(["config", "--local", "core.worktree", redirectedWorktree], {
    cwd: sourceRoot,
  });
  assert.throws(
    () => assertCanonicalGitWorktree(sourceRoot),
    /effective Git worktree|core\.worktree redirection|non-allowlisted local Git configuration/,
  );
});

test("rejects non-allowlisted local Git execution configuration", (t) => {
  const { repositoryRoot } = makeRepository(t);
  const sourceRoot = join(repositoryRoot, "source");
  mkdirSync(sourceRoot);
  runIsolatedGit(["init", "--quiet"], { cwd: sourceRoot });
  runIsolatedGit(["config", "--local", "diff.external", "/tmp/untrusted"], {
    cwd: sourceRoot,
  });

  assert.throws(
    () => assertCanonicalGitWorktree(sourceRoot),
    /non-allowlisted local Git configuration: diff\.external/,
  );
});

test("install destination rejects symlinked parent and final paths", (t) => {
  const { repositoryRoot, temporaryRoot } = makeRepository(t);
  const nativeRoot = join(repositoryRoot, "native");
  const externalRoot = join(temporaryRoot, "external-install");
  mkdirSync(nativeRoot);
  mkdirSync(externalRoot);

  const symlinkedParent = join(nativeRoot, "symlinked-parent");
  symlinkSync(externalRoot, symlinkedParent);
  assert.throws(
    () =>
      assertSafeInstallDestination(
        nativeRoot,
        join(symlinkedParent, "opencode"),
      ),
    /symlinked component/,
  );

  const realParent = join(nativeRoot, "bin");
  mkdirSync(realParent);
  const symlinkedDestination = join(realParent, "opencode");
  symlinkSync(join(externalRoot, "opencode"), symlinkedDestination);
  assert.throws(
    () => assertSafeInstallDestination(nativeRoot, symlinkedDestination),
    /must not be a symlink/,
  );
});

test("install destination accepts a canonical missing or regular final file", (t) => {
  const { repositoryRoot } = makeRepository(t);
  const nativeRoot = join(repositoryRoot, "native");
  const installParent = join(nativeRoot, "bin");
  const destination = join(installParent, "opencode");
  mkdirSync(installParent, { recursive: true });

  const missing = assertSafeInstallDestination(nativeRoot, destination);
  assert.equal(missing.canonicalParent, installParent);
  assert.equal(missing.destinationStatus, undefined);

  writeFileSync(destination, "native artifact");
  const regular = assertSafeInstallDestination(nativeRoot, destination);
  assert.equal(regular.destinationStatus.isFile(), true);
  assert.equal(lstatSync(installParent).isSymbolicLink(), false);
});
