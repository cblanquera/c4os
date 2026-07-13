# Terminal Plugin Proof Planning

Status: proposed

Proof implementation artifacts, if approved during proof execution, must live under the listed `proofs/<proof-name>/` paths. Do not create proof code during planning replay.

| Proof Path | Proof Question |
| --- | --- |
| proofs/terminal-user-pty-lifecycle/ | Prove one user PTY per chat, cwd selection, delete lifecycle, and separation from runtime terminal tools. |

## Execution Results

Verification command:

```sh
node --test proofs/terminal-user-pty-lifecycle/proof.test.mjs
```

| Proof Path | Result | Decision |
| --- | --- | --- |
| proofs/terminal-user-pty-lifecycle/ | Passed. The same chat reuses one user PTY, assigned projects start in project cwd, unassigned chats start in user home, runtime terminal tools remain thread/Chat Debug activity, and chat removal terminates the PTY plus deletes chat-owned terminal state. | Promote the one-user-PTY-per-chat lifecycle and runtime/tool separation as feasible. |
