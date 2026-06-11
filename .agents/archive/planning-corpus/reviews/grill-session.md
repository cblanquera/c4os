# Grill Session Questions And Accepted Answers

This file itemizes the 54 normalized grill questions / boundary decisions from the implementation-readiness grill. The phrasing is cleaned up for readability, but each answer reflects an accepted boundary from the session.

## MVP Shape

1. **Question:** Is the MVP coding-first or general-purpose from day one?
   **Accepted answer:** Coding-first. The MVP is a single-user local desktop app for one selected local Git project at a time, not a general-purpose workspace.

2. **Question:** Should MVP use Tauri, Electron, or Chromium?
   **Accepted answer:** Use Tauri for MVP. Chromium is deferred unless Tauri WebView cannot support the basic app UI and text-like previews.

3. **Question:** Can the app register multiple projects?
   **Accepted answer:** Yes. Multiple local Git projects may be registered, but only one selected project is active at a time.

4. **Question:** How many sessions can actively run?
   **Accepted answer:** One active/running session at a time. MVP supports latest/resume behavior only, not full session management.

5. **Question:** Should non-Git folders be supported?
   **Accepted answer:** No. MVP projects are Git-backed folders only.

## File Access

6. **Question:** How should project-root file reads and writes work?
   **Accepted answer:** Project-root reads are allowed and logged without per-read approval. File writes are approval-gated. Outside-root reads and writes are blocked.

7. **Question:** How should symlinks and traversal paths be handled?
   **Accepted answer:** Use resolved-target containment: canonicalize paths, resolve traversal segments and symlinks, then enforce the selected project root.

8. **Question:** Can secret-deny files be read or written with approval?
   **Accepted answer:** No. Secret-deny files are hard-blocked for agent reads and writes with no approval override.

9. **Question:** Should the file browser be an editor?
   **Accepted answer:** No. The MVP file browser is read-only. It may show secret-deny files as present, but must not preview contents.

## Shell Execution

10. **Question:** What shell execution baseline is acceptable?
    **Accepted answer:** Shell commands run as the current OS user in the project root or approved subpath. MVP must not claim strong sandboxing. Approved commands may use normal network access.

11. **Question:** Should shell commands inherit the full interactive environment?
    **Accepted answer:** No. Shell commands use a backend-filtered environment, and obvious credential/secret variables are stripped.

12. **Question:** How are destructive shell commands approved?
    **Accepted answer:** Destructive commands require fresh one-time approval every time and are never covered by session allow.

13. **Question:** What is the recovery story after a bad approved local action?
    **Accepted answer:** Manual recovery only. MVP provides approvals, logs, changed-file lists, diffs, and stop controls, but no rollback, snapshots, restore points, or undo stack.

14. **Question:** How should shell output be persisted?
    **Accepted answer:** Persist bounded redacted/truncated summaries only, not unlimited raw stdout/stderr.

15. **Question:** Can shell session allow exist?
    **Accepted answer:** Yes, but narrowly: only matching non-destructive shell commands inside the selected project root or approved subpath for the current session.

16. **Question:** What should denied or blocked actions return?
    **Accepted answer:** Structured denial results with categories such as `user_denied`, `outside_project_root`, `secret_denied`, `destructive_requires_one_time`, or `mvp_scope_blocked`; no silent failure or fake success.

## Git

17. **Question:** What Git behavior is in MVP?
    **Accepted answer:** Git inspection is allowed and logged without approval. Git state changes are approval-gated. Product workflows for commits, branches, PRs, merges, rebases, tags, pushes, and worktrees are excluded.

## Runtime And Instructions

18. **Question:** How should root `AGENTS.md` behave?
    **Accepted answer:** Root `AGENTS.md` is display-only app guidance. It does not automatically enter model context or affect permissions. Runtime-native instruction loading must be observable and disclosed or disabled.

19. **Question:** Who owns canonical records?
    **Accepted answer:** The app owns canonical session, message, tool, approval, artifact, project, model, and settings records. OpenCode IDs/logs/persistence are adapter references only.

20. **Question:** What must OpenCode prove before direct MVP use?
    **Accepted answer:** Structured events, session control, reliable pre-execution interception for file writes/shell/Git state changes, provider/model routing visibility, and instruction-loading observability. UI-only control, terminal scraping, post-execution audit, and best-effort interception are not enough.

## Provider And Model Context

21. **Question:** What model provider path is in MVP?
    **Accepted answer:** OpenRouter only. Direct providers, local providers, fallback routing, and provider-specific app settings are post-MVP.

22. **Question:** What if the runtime uses a different model than the app-selected OpenRouter model?
    **Accepted answer:** The runtime effective model must match the app-owned selected OpenRouter model. Undetectable override blocks direct OpenCode MVP use.

23. **Question:** How should model metadata and cost/accounting work?
    **Accepted answer:** Metadata is best effort and stale/unknown states are labeled. No per-call token counts, cost estimates, spend history, budget meters, credit balance, billing links, or budget enforcement in MVP.

24. **Question:** Where is the OpenRouter API key stored?
    **Accepted answer:** Only in the OS keychain or platform credential store. If unavailable or failing, provider setup is blocked.

25. **Question:** Can OpenRouter credentials change while a session is running?
    **Accepted answer:** No. Key update/revoke is allowed only when no session is running. A running session keeps its starting credential reference.

26. **Question:** What can be sent into OpenRouter-bound model context?
    **Accepted answer:** Active transcript, selected model/routing metadata, approved or policy-allowed tool results/summaries, explicitly read in-project file contents, persisted safe shell summaries, `output-omitted` markers, and safe shell summary reason labels.

27. **Question:** How should users be informed about model context?
    **Accepted answer:** Provider setup gives standing disclosure that prompts and bounded context leave through OpenRouter. Model-call activity records bounded context-source summaries. No per-call approval, raw prompt export, token inspector, or editable context composer.

28. **Question:** What happens on OpenRouter or network failure?
    **Accepted answer:** Fail closed. Already-submitted user messages remain appended, failed assistant/run state is persisted, and retry/follow-up is explicit. No hidden retry, silent resend, fallback provider, offline fallback, or synthetic continuation.

29. **Question:** What does Stop do?
    **Accepted answer:** Stop cancels runtime/model streaming and app-supervised child processes while preserving transcript, tool history, approvals, artifacts, and partial assistant output marked stopped/interrupted.

30. **Question:** Should runs continue when the app is minimized or closed?
    **Accepted answer:** Minimized can continue while the app process is running. Closing/quitting stops active work. No tray daemon, background service, notifications, scheduled runs, or wake/resume automation in MVP.

31. **Question:** What happens after crash or force-quit?
    **Accepted answer:** Next launch marks the run interrupted/crashed and preserves last persisted transcript/tool records. It does not reconnect, reattach, replay, continue generation, or resend unsent prompts.

## Sessions, Search, And Artifacts

32. **Question:** Is transcript history mutable?
    **Accepted answer:** No. Transcript is append-only. No message edit/delete, pruning, redaction UI, branch-from-message, branch-from-failure, or conversation rewrite.

33. **Question:** Is search in MVP?
    **Accepted answer:** No global search, cross-session search, full-text transcript search, tool-log search, artifact search, or project-wide file content search UI.

34. **Question:** What artifacts are in MVP?
    **Accepted answer:** Text-like outputs only: plain text, Markdown, logs, diffs, and generated source/config files. No active HTML, image/PDF/document/spreadsheet previews, duplicate/export workflows, artifact execution, or Chromium-backed previews.

35. **Question:** Is browser automation or web content ingestion in MVP?
    **Accepted answer:** No. Browser automation, DOM extraction, screenshots, web content ingestion, and browser-based previews are excluded.

36. **Question:** Is OpenRouter cost/accounting in MVP?
    **Accepted answer:** No. Account diagnostics, credit balance, invoices, spend warnings, token/cost accounting, budget meters, and budget enforcement are excluded.

37. **Question:** Can the session model change after session creation?
    **Accepted answer:** No. One selected OpenRouter model is fixed for the whole session. Default model changes apply only to future sessions.

38. **Question:** Are custom agents/personas in MVP?
    **Accepted answer:** No. MVP uses one default coding agent/persona. Runtime agent references are adapter metadata only.

39. **Question:** Are prompt editors or skill creator workflows in MVP?
    **Accepted answer:** No. Editable system prompts, project prompt editors, instruction composers, prompt templates, hidden app instruction layers, and skill creator workflows are post-MVP.

40. **Question:** Can root `AGENTS.md` be explicitly read or edited?
    **Accepted answer:** Yes. Explicit reads follow normal logged project-root read rules and can enter model context only because they were explicitly read. Edits follow normal approval-gated write rules and do not automatically reload into model context, permissions, or precedence.

## Approval Details

41. **Question:** What should file-write approval prompts show?
    **Accepted answer:** Target path, action type, and bounded diff/summary when available. If a safe diff cannot be produced, disclose that clearly and still require explicit approval.

42. **Question:** Can multiple file writes be approved as a batch?
    **Accepted answer:** Yes, as one explicit atomic batch when every target/action/per-file preview state is visible. If any item is blocked, the whole batch is blocked.

43. **Question:** What is the file-write batch cap?
    **Accepted answer:** A fixed MVP product threshold, initially 10 files plus bounded total preview size. Oversized batches are blocked and must be split.

44. **Question:** Are pending approvals durable?
    **Accepted answer:** No. Pending approval prompts are non-durable runtime state and are discarded on close, crash, force-quit, or restart.

45. **Question:** What do answered approval records store?
    **Accepted answer:** Durable local ledger records with structured metadata, decision, timestamp, resulting action status, and bounded redacted prompt summary or diff reference. No full prompt replay blobs, raw command output, raw secret values, edit/delete, or durable pending state.

46. **Question:** Can approval records be exported or copied?
    **Accepted answer:** Approval records are local-ledger-only. No export, copy-all, JSON download, support bundle, share workflow, or dedicated approval-record copy button. Normal OS text selection/copy of visible ledger text is allowed.

## Shell Output Follow-Up Questions

47. **Question:** Should the user be able to copy visible shell output summaries?
    **Accepted answer:** Yes. Normal OS text selection/copy of visible redacted/truncated summaries is allowed, but no dedicated copy-raw-output or export button exists.

48. **Question:** What if a safe shell output summary cannot be produced?
    **Accepted answer:** Persist command metadata plus an explicit `output-omitted` marker. Do not store raw stdout/stderr as fallback.

49. **Question:** Should live terminal buffers be retained after a command/run completes?
    **Accepted answer:** No. Live buffers are ephemeral and are not retained after navigation away, reload, app close, or session restore.

50. **Question:** Can an open live terminal drawer remain after command completion?
    **Accepted answer:** Yes, temporarily in the same app session only when labeled live/ephemeral and not presented as persisted history.

51. **Question:** Should shell output summaries be sent into model context?
    **Accepted answer:** Yes, but only as the persisted bounded redacted/truncated summary or `output-omitted` marker from the tool record. Never send live raw terminal buffers or omitted raw output.

52. **Question:** Can users expand shell summaries to see redaction/truncation reasons?
    **Accepted answer:** Yes, but only safe labels such as `truncated_by_size`, `redacted_secret_pattern`, and `output_omitted_safe_summary_failed`. No redacted substrings, sensitive raw byte counts, offsets, hashes, or reconstruction hints.

53. **Question:** Can shell summary reason labels enter model context?
    **Accepted answer:** Yes. They are part of the persisted safe tool record, but only the labels. No raw counts, substrings, offsets, hashes, or reconstruction metadata.

54. **Question:** Should exact shell redaction/truncation limits be defined now?
    **Accepted answer:** No. Exact limits remain assigned to the local execution spike. The frozen contract is: bounded summary, safe reason labels, fail-closed `output-omitted`, no raw fallback, and no reconstructable metadata.
