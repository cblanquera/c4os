# Implementation Clarification Questions

This is the lossless resumable ledger for the Spec 00003 grill pass. Evidence-resolved answers come from accepted Context, References, and the Frozen research journeys. Recommendations remain unaccepted until the user chooses them; no user-owned questions are currently queued.

## User Questions Logged 2026-07-18

### Q-001 — First-launch onboarding
- **Status:** accepted-decision.
- **Source:** user, context, wireframe.
- **Question:** “What is the user onboarding process once the user opens the app for the first time?”
- **Evidence answer:** With no provider, C4OS opens a standalone provider form without Settings navigation. The user selects OpenRouter, Hugging Face, OpenAI, or OpenAI Compatible; supplies the visible required profile/auth/endpoint fields; tests the connection; and continues through secure credential storage to Workspace Start. Errors preserve non-secret form state and never echo secrets. See D-005.
- **User answer:** “testing must succeed. basically we need a model to start chat.”
- **Normalized decision:** Continue requires a successful current connection test and at least one usable model. Q-012 defines the accepted initial model/runtime/environment defaults.
### Q-002 — Workspace behavior
- **Status:** accepted-decision with implementation research complete.
- **Source:** user, context, wireframe.
- **Question:** “how do workspaces work?”
- **Evidence answer:** Start offers Open Folder, Open Workspace, Clone Repository, and recent Workspaces. C4OS resolves an explicit path, records a Workspace identity and trusted-root grant, opens Chat without invoking AI, and restores Projects and nested sessions. The active shell can add, relocate, rename, reorder, or remove Project references. The accepted docs do not define the canonical Workspace-versus-Project relationship or saved-Workspace format.
- **User answer:** “You actually failed to explain what a workspace is. A workspace works similar to a VSCode workspace. It's a zip file that stores a list of projects, workspace level, project level configs, and per chat configs/cache/archive in order to load in the main screen.”
- **Normalized decision:** A Workspace is the portable zip container; Projects are referenced folders/trusted roots, and Chat/config/cache/archive overlays belong to the loaded Workspace. Q-011 retains save/repack and recovery mechanics.
### Q-003 — Composer modes and AI use
- **Status:** accepted-decision.
- **Source:** user, context, wireframe.
- **Question:** “What is the purpose for chat modes? for files, browser and terminal modes, will AI be used when sending?”
- **Evidence answer:** Chat `Send` and Reply `Send` invoke AI. Files `Open`, Browser `Open`/navigation, Terminal `Run`, and expanded Terminal input are direct C4OS-brokered operations and do not invoke AI merely to create a Response Artifact. Security and approval rules still apply. An artifact Reply switches back to Chat/AI semantics. See D-006.
- **User answer:** “this is all correct.”
- **Decision update:** D-006 and feature coverage CHAT-007.
### Q-004 — Agent file-editing flow
- **Status:** accepted-decision through Q-013.
- **Source:** user, context, wireframe.
- **Question:** “When an agent is editing files what is the process flow? what will the user see?”
- **User clarification:** “You might be confusing the user file editing mode vs user chatting \"can you create AGENTS.md for me and update the terms in CONTEXT.md\"? what is the process flow? what will the user see?”
- **Normalized note:** This question is exclusively about an AI turn initiated from Chat that reads and changes one or more files. Direct user editing inside a File artifact is a separate flow and does not answer it.
- **Decision update:** Q-013 and D-009.
### Q-005 — AI interaction with artifacts
- **Status:** accepted-decision with implementation-acceptance follow-up.
- **Source:** user, context, wireframe.
- **Question:** “Can AI interact with artifacts such as file explorer, file editor, browser, terminal?”
- **Evidence answer:** Yes, through Reply/Chat and C4OS-brokered tools—not by giving the model renderer or native authority. Browser Reply updates the Browser artifact/activity; File Reply proposes a diff; Terminal Reply creates a new Terminal result in the same Chat shell. Folder is a permitted Reply target. Its exact snapshot was open when this question was answered and is now resolved by Q-014/D-013.
- **User answer:** “Acceptable answer. This will need to be checked during human acceptance phase once implemented and verified.”
- **Follow-up:** Q-014 is resolved; ART-001 and ART-007 retain the required post-verification human acceptance.
### Q-006 — Chat search results
- **Status:** accepted-decision; reconciled to the complete accepted r013 revision.
- **Source:** user, context, wireframe.
- **Question:** “When I search for chat sessions, where do the results displayed?”
- **Evidence answer:** Results remain in the left project/session navigator. A non-empty query replaces the Projects heading and hierarchy with flat session-title results, and every result identifies its owning Project. Opening a result preserves the query and keeps the selected Chat in the center workspace. Explicit clear or Escape restores Projects with its prior ordering and expansion state. See D-007.
- **User answer:** “We need to update the wireframes to reflect this.”
- **Reconciliation note:** The complete r013 wireframe through Review Round 7 now carries this accepted behavior and its deterministic verification record.
- **Decision update:** D-007.
### Q-007 — Terminal and Browser sharing scope
- **Status:** accepted-decision.
- **Source:** user, context, wireframe.
- **Question:** “What are terminal and browser sessions shared with? workspace level only? project level only? chat session level only?”
- **Evidence answer:** Terminal is Chat-scoped: one environment-qualified persistent shell per Chat, never shared with another Chat. Browser artifacts and their navigation histories belong to the originating Chat. Browser Environment storage scope was unresolved at this point and is now defined by Q-015/Q-021.
- **User answer:** “accepted”
- **Follow-up:** Q-015 and Q-021 are resolved.
### Q-008 — `config.toml` contents
- **Status:** accepted-decision with implementation research complete.
- **Source:** user, context, spec.
- **Question:** “What will config.toml contain?”
- **Evidence answer:** The accepted product contract places C4OS `config.toml` at `~/.c4os/config.toml`; Workspace-, Project-, and Chat-scoped overlays stay in the Workspace archive. It is C4OS-specific, secret-free, and unable to override managed policy.
- **User input:** C4OS Home is `~/.c4os` and contains `config.toml`.
- **Follow-up:** Q-016 is resolved by R-008, D-026, and IS-006.

### Q-009 — Plugin, skill, and MCP storage
- **Status:** accepted-decision with implementation research complete.
- **Source:** user, context, spec.
- **Question:** “Where will plugins, skills and MCP server configs be stored?”
- **Evidence answer:** The logical split is known: immutable plugin content in a C4OS-owned versioned cache; trust/enablement/configuration and MCP definitions in Rust-owned product records; secrets in the vault; skills retained at source-qualified Project, Workspace, user, plugin, or bundled scopes. Exact filesystem paths, editable/exported forms, backup, and portability are not defined.
- **User answer:** “I was expecting app level config to be in ~/.c4os and contains:
  - last loaded workspace (unzipped)
  - app level MCP, skills, plugin configuration
  - app marketplaces config
  - config.toml

  This makes it possible for folders (projects) to have different configs and chat items depending on the workspace loaded.”
- **Normalized decision:** C4OS Home owns app-level configuration and the unpacked last-loaded Workspace; Workspace archives own their Workspace, Project, and Chat overlays. Raw secrets remain vault-only.
- **Follow-up:** Q-017 is resolved by R-009, D-027, and IS-007.

## Follow-Up Ledger

### Q-010 — Canonical Workspace model
- **Status:** accepted-decision.
- **Source:** Q-002, GAP-008.
- **Question:** Is one Workspace a saved C4OS container that owns multiple Projects, with each Project acting as one trusted execution root and each Chat belonging to one Project?
- **User answer:** A Workspace works like a VS Code workspace and is a zip containing Projects plus Workspace-, Project-, and Chat-scoped state.
- **Decision update:** D-012; GAP-008 resolved.

### Q-011 — Saved Workspace lifecycle
- **Status:** evidence-resolved by implementation research.
- **Source:** Q-002, GAP-009.
- **Question:** How is the unpacked Workspace working copy safely repacked, autosaved, locked, recovered, and migrated; when do Open Folder and Clone create a Workspace versus add a Project?
- **Accepted answer:** Open Workspace loads a Workspace zip into the last-loaded working copy under `~/.c4os`.
- **Research disposition:** R-007, D-025, and IS-005 define the manifest, atomic save, recovery, locking, migration, extraction, and Start-action mechanics.

### Q-012 — Onboarding completion and defaults
- **Status:** accepted-decision.
- **Source:** Q-001, GAP-010.
- **Question:** What must succeed before Continue, and how are the first provider, model, runtime, and execution environment chosen?
- **User answers:** “testing must succeed. basically we need a model to start chat.” and “GAP-010 - Suggested default”.
- **Agent recommendation corrected by the user 2026-07-27:** Require a successful current provider test and at least one usable discovered model, but do not insert a model picker or default-confirmation step into onboarding.
- **Normalized decision:** C4OS automatically chooses the production-ready model with the most normalized `supported` features, breaking ties by discovery rank and stable identity. Continue persists it with OpenCode and Local; ordinary model controls own later changes before first-Chat binding.
- **Decision update:** Corrected D-020 resolves GAP-010; SET-001 owns implementation and acceptance.

### Q-013 — Agent change-set review
- **Status:** accepted-decision.
- **Source:** Q-004, GAP-011.
- **Question:** For a Chat request such as “create `AGENTS.md` and update `CONTEXT.md`,” should C4OS show and approve one combined proposed change set before writing, or let the agent write tool-by-tool and show a completed change set afterward?
- **User answer:** “This is okay if the project isnt a git project or editing files outside of the given project folder. Oppositely if it is version controlled then there is no need to ask to change files. I would leave it up to the user to have their own git flow. for example if I dont like the changes, I just revert. If I like it then I commit.”
- **Normalized decision:** For version-controlled targets inside the active Project, C4OS adds no separate change-set approval prompt when effective policy already allows the writes; the agent may write through brokered tools and shows concise activity plus the completed changed-file/diff artifacts. An explicit `Ask` policy still applies. A non-version-controlled Project or any target outside the active Project requires path-and-change-specific approval before writing. C4OS never branches, commits, resets, or reverts automatically; the user owns Git disposition. Exact batching is implementation-flexible as long as guarded changes are approved before effect.
- **Decision update:** D-009 resolves GAP-011; CHAT-009 covers implementation and acceptance.

### Q-014 — Artifact context contract
- **Status:** accepted-decision.
- **Source:** Q-005, GAP-012.
- **Question:** What exact File, Folder, Browser, or Terminal snapshot and permission context should Reply supply to AI?
- **Agent recommendation accepted:** Use a bounded immutable current-state snapshot plus stable artifact reference, with brokered expansion when more context is needed.
- **User answer:** “accepted”
- **Normalized decision:** File supplies selected text or the current document within budget, including explicitly marked unsaved state plus path/type/version. Folder supplies current location, selection, and a bounded non-recursive listing. Browser supplies URL/title/navigation and selected, visible, or bounded extracted content without cookies, credentials, storage, or unrelated history; screenshots are need- and capability-gated. Terminal supplies selected command, working directory, environment identity, process/exit state, and selected or recent bounded output without raw environment values, passwords, or unrelated history. The budget derives deterministically from effective model context, truncation is visible, and only normalized capabilities—not secrets or tokens—are included. The snapshot is frozen for the turn; additional reads are brokered and live state is revalidated before effects.
- **Decision update:** D-013 resolves GAP-012; ART-007 owns implementation and acceptance.

### Q-015 — Browser Environment semantics
- **Status:** accepted-decision through Q-021.
- **Source:** Q-007, GAP-013.
- **Question:** What data and lifecycle do All browsers, Per project, Per chat session, and None share?
- **User answer:** “Q-015 - (all if applicable) cookies, session storage, local storage, indexed db”
- **Normalized decision:** The selected Browser Environment partitions all applicable browser storage categories: cookies, `sessionStorage`, `localStorage`, and IndexedDB, while preserving normal origin/storage semantics. Q-021 defines the accepted lifecycle.
- **Decision update:** D-014 resolves GAP-013.

### Q-016 — C4OS configuration contract
- **Status:** evidence-resolved by implementation research.
- **Source:** Q-008, GAP-014.
- **Question:** What is the schema, scoped precedence, reload, and UI/database reconciliation contract for C4OS `config.toml`?
- **Accepted answer:** The app-level file is `~/.c4os/config.toml`; Workspace-, Project-, and Chat-scoped configuration stays in the Workspace archive.
- **Research disposition:** R-008, D-026, and IS-006 define the strict schema, precedence algorithm, validation, reload, diagnostics, and UI/database reconciliation.

### Q-017 — Extension storage layout
- **Status:** evidence-resolved by implementation research.
- **Source:** Q-009, GAP-015.
- **Question:** Which exact subpaths and formats under `~/.c4os` and Workspace archives own plugin, skill, MCP, marketplace, package, cache, and generated state?
- **Accepted answer:** `~/.c4os` owns app-level configuration and the unpacked last-loaded Workspace; the Workspace archive owns its Workspace/Project/Chat overlays.
- **Research disposition:** R-009, D-027, and IS-007 define exact subpaths, file formats, package/cache separation, backup/reset, and portability while preserving accepted scope ownership and vault-only C4OS credentials.

### Q-018 — Branch behavior
- **Status:** accepted-decision through Q-022.
- **Source:** GAP-016.
- **Question:** Does the visible Branch control fork the C4OS conversation, the runtime-native session, Git, or some combination?
- **User answer:** “Q-018 - applies to the files/folders inside of the project given that the project folder is versioned by git. If not, then that display should be hidden (or removed)”
- **Normalized decision:** Branch is a Git repository control for the active Project folder, not a C4OS conversation or runtime-session fork. It applies only to files/folders inside that repository and is hidden for a non-Git Project. Explicit Git operations remain brokered; Q-022 defines dirty-worktree safety.
- **Decision update:** D-015 resolves GAP-016; CHAT-010 covers implementation and acceptance.

### Q-019 — Removal and retention
- **Status:** accepted-decision.
- **Source:** GAP-017.
- **Question:** What is removed, retained, recoverable, terminated, or exported when a Chat, Project, or Workspace is removed?
- **User answer:** “Q-019 - Agent default; should just mark that it's inactive, thats all.”
- **Normalized decision:** Remove only marks the Chat, Project, or Workspace record inactive and excludes it from active surfaces. It does not delete user files or C4OS data, purge/export records, or terminate scoped processes.
- **Decision update:** D-016 resolves GAP-017; UX-014 owns implementation and acceptance.

### Q-020 — Approval concurrency
- **Status:** accepted-decision.
- **Source:** GAP-018.
- **Question:** How are multiple pending or stale approvals queued and presented across agent actions and runs?
- **User answer:** “Q-20 - Agent default”
- **Normalized decision:** Serialize effectful approvals within a Run Attempt; visibly queue independent-run approvals; bind every prompt to one canonical action; distinguish pending, expired, denied, and completed state; expire approval when target/version state changes.
- **Decision update:** D-017 resolves GAP-018; UX-015 owns implementation and acceptance.

### Q-021 — Browser storage lifecycle
- **Status:** accepted-decision.
- **Source:** Q-015, GAP-013.
- **Question:** For All browsers, Per project, Per chat session, and None, when is each storage partition created, persisted across restart, and destroyed?
- **Agent recommendation accepted:** App-wide persists until explicit clear; Workspace-qualified Project and Chat profiles persist with their records; None is per-artifact ephemeral and destroyed on close; `sessionStorage` follows page/tab lifetime. Browser state stays in a protected local C4OS Home store and outside portable Workspace archives, normal exports, diagnostics, and model context.
- **User answer:** “Q-021 - Agent default”
- **Normalized decision:** All browsers uses one persistent C4OS-wide profile. Per project and Per chat session use persistent profiles keyed by Workspace+Project and Chat identity. None uses an ephemeral per-artifact profile destroyed on close and not restored after restart. Inactivation does not clear profiles; an explicit Clear Browser Data action targets app-wide, Project, Chat, or ephemeral scope.
- **Research correction:** R-004 confirms that public macOS WebKit stores raw website-data bytes in its managed app container. C4OS Home owns the protected scope-to-profile registry and lifecycle; the accepted local-only and exclusion boundaries remain unchanged.
- **Decision update:** D-014 resolves GAP-013; SET-010 owns implementation and acceptance.
### Q-022 — Git Branch dirty-worktree behavior
- **Status:** accepted-decision.
- **Source:** Q-018, GAP-016.
- **Question:** What happens when the user selects or creates a Git branch while the Project worktree has uncommitted or conflicting changes?
- **Agent recommendation accepted:** Never auto-stash, commit, reset, or discard. Allow Git-safe switches that preserve the worktree; otherwise block and explain the conflicting paths.
- **User answer:** “Q-022 - Agent default”
- **Normalized decision:** C4OS performs only explicit brokered Git operations. A branch selection or creation may proceed when Git can preserve the dirty worktree; otherwise C4OS blocks it and identifies conflicts without changing the worktree.
- **Decision update:** D-015 resolves GAP-016; CHAT-010 owns implementation and acceptance.

### Q-023 — First release posture
- **Status:** accepted-decision.
- **Source:** GAP-006, user.
- **Question:** Is the first milestone a local development build, a signed internal build, or a notarized distributable macOS application?
- **User answer:** “GAP-006 - local development build”.
- **Normalized decision:** The first implementation and review milestone is a local macOS development build. Signing, notarization, and distributable updater evidence remain later release gates without reducing the implementation contract.
- **Decision update:** D-018 resolves GAP-006.

### Q-024 — Complete extension delivery and implementation decisions
- **Status:** accepted-decision.
- **Source:** GAP-007, user.
- **Question:** Should Plugins, Skills, and MCP Servers initially deliver real installation/execution or a production UI backed by disabled, metadata-only records until later tasks?
- **User answer:** “Honestly no review slices just build it all to spec (app reflects wireframe functionality), any blockers during implementation, I want the agent to make a decision for me. That's why we are doing heavy research now.”
- **Normalized decision:** Implement the complete accepted contract, including real supported Plugin, Skill, and MCP installation, activation, execution, supervision, revocation, and failure behavior; metadata-only facades do not count as completion. Do not add intermediate product-review slice gates. During post-Freeze implementation, the agent may resolve bounded technical blockers from current research and record its rationale, but may not silently weaken accepted behavior or security boundaries. Final verification and human acceptance still apply.
- **Decision update:** D-019 resolves GAP-007; R-006 selects the concrete extension implementation boundary.
