# Brief

## User Goal

Reassess whether C4OS should restart on Electron with a Node server layer, use both the Pi and OpenCode SDKs directly, and reduce dependence on OS-specific facilities such as theme handling and keychain storage. Compare that direction with the current implementation, identify macOS/Windows/Linux flexibility problems, and preserve the research in a reviewable spec.

## Outcome

Produce a reviewable recommendation that:

1. inventories current Tauri/Rust/macOS and Unix coupling;
2. compares it with a sandboxed Electron renderer, thin Electron main process, Node application service, and isolated runtime workers;
3. separates cross-platform leverage from security and native behavior that remains target-specific;
4. defines what to preserve, replace, and prove in a controlled restart; and
5. leaves accepted Context and Frozen implementation records unchanged until the user accepts a new direction.

## In Scope

- Current production source at audited commit `336dd5e8f53fed85f02adf692afb0e9b53cc59b0` and read-only observations of the 2026-07-28 working tree.
- Complete repository research lineage in Specs 00001 through 00003 and current Context.
- Current primary documentation/source for Electron, OpenCode SDK/server/desktop, Pi SDK, Node SQLite, and Node PTY support.
- Shell, renderer isolation, application-service authority, runtimes, Browser, credentials, theme, Terminal, processes, filesystem, persistence, plugins/MCP, packaging, updates, and native evidence.
- A Proof-first restart and migration boundary.

## Non-Goals

- Editing production code, deleting current work, or starting the rewrite.
- Treating the user's architectural preference as an already accepted Context change.
- Claiming that Electron eliminates target-specific security, filesystem, PTY, process-tree, accessibility, signing, or packaging work.
- Selecting a SQLite driver, HTTP-vs-MessagePort control transport, installer, updater, or native containment helper without Proof evidence.
- Treating CI, Docker, a VM, emulation, or one OS as native evidence for another target.

## Deliverable Standard

This research spec may Freeze only after the architecture direction is accepted, critical Gaps have a recorded disposition, the Proof matrix is completed or explicitly deferred, and reusable accepted outcomes are promoted into Context. Freeze still would not authorize implementation; a separate implementation spec is required.
