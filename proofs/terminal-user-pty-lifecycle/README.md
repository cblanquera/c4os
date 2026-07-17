# Terminal User PTY Lifecycle Proof

This proof checks the C4OS user-terminal lifecycle and its separation from
runtime tool activity. It is self-contained and does not depend on a separate
planning record.

## Question

Can the Terminal plugin own exactly one user PTY per chat while runtime
terminal tools remain separate tool-gateway activity?

## Verification

```sh
node --test proofs/terminal-user-pty-lifecycle/proof.test.mjs
```

## Result

Passed on 2026-07-02 as part of the third POC batch.

The harness proves:

- Opening a terminal for the same chat returns the existing user PTY.
- Assigned project chats start in the project cwd; unassigned chats start in
  the user home directory.
- Runtime `terminal.run` activity is visible in thread context and Chat Debug,
  not in the Terminal panel.
- Removing a chat terminates the user PTY and deletes chat-owned terminal
  state.

## Decision

Promote the one-user-PTY-per-chat lifecycle and runtime/tool separation as
feasible.
