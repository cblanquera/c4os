# Task 00013 Onboarding And Settings Integration Acceptance Evidence

Date: 2026-07-23

Target: macOS 26.5.2 arm64, production-composed debug `C4OS.app`

Verified Task 00012 base: `eedc3e8`

Disposable acceptance home: `/private/tmp/c4os-task-00013-native.8qulpH` (owner-only mode `0700`)

## Golden path

1. A first launch with no provider stays on production onboarding and exposes the explicit session-only fallback when macOS credential storage is unavailable.
2. One OpenAI-compatible profile stores only an opaque credential reference. Credential mutation and the real connection test require two serialized Action Gateway approvals.
3. The deterministic provider fixture returns two production-ready models. C4OS selects the recommended model and confirms the OpenCode and Local defaults before entering Workspace Start.
4. The native Settings menu and `Cmd+,` enter the single production Settings router. Providers, Models, Runtimes, Configuration, Advanced Policies, Plugins, Skills, and MCP Servers project authoritative service state without QA controls.
5. Provider/model writers remain disabled while approval is pending. Endpoint drift requires credential re-entry, a deterministic `503` produces a bounded degraded state, and the original endpoint recovers to two models.
6. After a controlled restart, the profile and selected model identity remain durable while the session-only credential is absent. Testing stays disabled until the user accepts session-only storage and re-enters the key.
7. Settings Back restores Workspace Start. The 622 px and 762 px native layouts remain internally contained.

## Automated verification

Implementation writers were frozen for the final matrix. Cargo-producing commands ran one at a time.

| Gate | Final result | Measured time |
| --- | --- | ---: |
| `npm run format:check` | passed | approximately 2.6 s |
| `npm run lint` | passed with zero errors | approximately 5.9 s |
| `npm run typecheck` | passed | approximately 4.3 s |
| `npm test` | 69 files and 361/361 tests passed | 10.43 s |
| production web build | passed | 0.61 s Vite build |
| QA web build | passed | 0.62 s Vite build |
| `npm run test:e2e -- --workers=1` | 43/43 passed; the suite contains one more scenario than the 42-test handoff baseline | 15.6 s |
| `cargo check --lib` | passed | passed |
| `cargo fmt --all -- --check` | passed | passed |
| full Rust workspace regression | passed across the library, every integration target, and doc tests; library 251 passed and 4 expected ignored tiers | serialized final matrix |
| exact host STDIO rerun | 2/2 passed after the sandbox-only workspace run could not launch the fixture | passed |
| `npm run protocol:generate` | Rust export 1/1 passed; generated protocol output was clean apart from the explicitly excluded unrelated `MarketplaceSnapshot.ts` whitespace diff | passed |
| `npm run tauri:build` | rebuilt `target/debug/bundle/macos/C4OS.app` and copied the pinned OpenCode/Pi resources | approximately 2 min 50 s |
| `npm run bundle:opencode-sdk:verify` | 1/1 passed | 1.35 s test time |
| `npm run bundle:opencode-assets:verify` | 1/1 passed | 7.12 s test time |
| `npm run bundle:pi:verify` | 1/1 passed | 12.04 s test time |
| ignored MCP Streamable HTTP tier | 2/2 passed | 3.54 s |
| ignored OpenCode bundle/native/loopback tier | 7/7 passed | 20.35 s |
| ignored OpenCode stream tier | 3/3 passed | 0.06 s |
| ignored packaged production-runtime tier | 4/4 passed | 131.10 s |
| `scripts/run-task-00004-native-golden.zsh` | existing private-TLS OpenCode/Pi native golden 1/1 passed against the rebuilt bundle | 57.68 s |

The full Playwright matrix passed only after the five failures from the prior 37/42 run were repaired. The already-green focused Terminal scenario and four QA route/Settings scenarios were not repeated unchanged. The complete final matrix ran once and passed 43/43.

The native build completed at 23:13 PST. Final rebuilt-native captures span approximately 23:31 through 23:38 PST. The complete native launch, degraded/recovery, restart, security inspection, diagnostic cleanup, and default-home reconciliation interval was approximately 20 minutes. Total Task 00013 wall time was not continuously machine-timed.

## Native macOS matrix

| Check | Result |
| --- | --- |
| Production launch gate | passed; first launch stayed on `/onboarding`, showed no QA route or folder-picker diagnostic, and required an explicit credential-storage choice |
| Credential boundary | passed; the key field remained secure, snapshots projected only presence/reference state, and credential mutation required an Action Gateway approval |
| Real provider test | passed; the production provider service contacted the isolated authenticated fixture and discovered exactly two models |
| Serialized approvals | passed; Save & Test continued from credential approval into connection-test approval without overlapping writes |
| Writer freeze | passed; Add/Edit/Test/Delete/availability/model selection and Model Refresh/bulk/row writers were disabled while their approval was pending |
| Recommended defaults | passed; OpenAI: GPT-5 Mini, OpenCode, and Local were shown and persisted for new Chats |
| Workspace Start | passed before restart and remained the launch destination after restart |
| Providers | passed; one enabled profile, selected model, bounded status, edit/test/delete controls, session-only notice, and no QA picker |
| Models | passed; provider/model identity, two routes, effective-runtime degradation, refresh approval, and disabled writers were truthful |
| Runtimes | passed; saved OpenCode default and unavailable local-installation evidence were explicit; existing Chat bindings remained out of scope for mutation |
| Configuration | passed; live app configuration, approval preset, restore behavior, shell boundary, Browser Environment choices, dirty/revert/save state, and Advanced entry were present |
| Advanced Policies | passed; all 28 category rules, seven groups, editable category choice, separate effective result, guardrails, exceptions tab, dirty/revert/save state, and policy version were present |
| Plugins | passed; marketplace, installed/directory, refresh/add, and empty states used the production extension service |
| Skills | passed; source-qualified discovery, precedence, search, and empty state used the production extension service |
| MCP Servers | passed; production add and empty states used the native MCP service |
| Native menu and shortcut | passed; the C4OS menu exposed `Settings…`, both the menu item and `Cmd+,` entered Providers, and Back restored Workspace Start |
| Responsive layout | passed at 1100×761, 762×761, and 622×761; navigation compressed and content stayed internally contained |
| Degraded and recovery | passed; a deterministic provider `503` produced `Connection failed`, recovery returned to two models, and final focus returned to the `Provider profiles` heading |
| Restart | passed; profile/default identity survived, the session-only credential did not, testing stayed disabled until fallback acceptance and key re-entry, and two approvals restored connectivity |
| Diagnostics | passed; `app_diagnostics` contained zero records in the isolated home |
| Final cleanup | passed; C4OS, provider fixture, OpenCode, and Pi acceptance processes were absent |

## Secret, persistence, and log inspection

- The exact synthetic sentinel was absent from the isolated home, app SQLite database, WAL, configuration, Browser profile state, and all 90 immutable security events plus 45 current security records.
- The C4OS process group contained no sentinel in its argv or inherited environment.
- The isolated home created no credential vault file because the accepted fallback was session-only.
- `config.toml` retained only schema version 1, model route `provider:task-00013-fixture::openai/gpt-5-mini`, runtime `opencode`, and environment `local`.
- The deterministic fixture necessarily received the synthetic sentinel through its isolated startup channel. That fixture process was excluded from the C4OS process-group assertion and was stopped after acceptance.
- A diagnostic direct-executable attempt aborted before initialization, after which Computer Use transparently launched a default-home instance. Its uniquely identified fixture provider row was removed after a recoverable SQLite backup at `/private/tmp/c4os-default-home-before-task13-cleanup.sqlite3`; the default home was then reopened, verified at blank onboarding, and closed. No raw sentinel was present there.

## Review artifacts

| Artifact | Review purpose | Dimensions | SHA-256 |
| --- | --- | ---: | --- |
| `task-00013-credential-storage-unavailable.jpg` | explicit no-plaintext credential fallback | 1100×761 | `6973b07a258fde122a79bcca8067112f680f4b2a322e8dbaa16be80a95d48a88` |
| `task-00013-session-only-choice.jpg` | accepted in-memory-only credential mode | 1100×761 | `3f2d1550e78b19028f37fad13ede4e4e6b7f99f006a8a1b9b8d78cf1b70d5eb0` |
| `task-00013-provider-test-approval.jpg` | real provider connection approval | 1100×761 | `58528a62b0f57165e98e5113d1c4fc58bf98da0bb21e392a9f8f2548c5089cca` |
| `task-00013-onboarding-passed.jpg` | two models and recommended initial defaults | 1100×761 | `eddd461aaf9b9589b10ce2e1e3f88af6426607d5bb183355dfc8c6cbf103aec4` |
| `task-00013-settings-providers.jpg` | production Providers state without QA controls | 1100×761 | `2c028c0b5cc6c7fce55a9c997f70c23a877512fe3fdd5af369aee435a1e17733` |
| `task-00013-settings-models.jpg` | refresh approval and disabled model writers | 1100×761 | `bbaaf50aae2ae1e145363d1dfd9c20837fe8560dd0e990b8af75146d5e2c28e0` |
| `task-00013-settings-runtimes.jpg` | saved runtime default and unavailable installation state | 1100×761 | `63c98b804f8e0261dbce68d5194ca1167cd1c55b15180b2099b1fc8113bc2405` |
| `task-00013-settings-configuration.jpg` | production configuration editing surface | 1100×761 | `f30edaf869bbb36559adfa28121a04958c7014a2a613e652ff010033e7228533` |
| `task-00013-settings-advanced-policies.jpg` | 28-rule policy editor and guardrails | 1100×761 | `e57fda46b7d8d7ed7c9b17aaf03552cd5ed4724e92e03d9b7e03e6656389a77e` |
| `task-00013-settings-plugins.jpg` | production Plugins empty state | 1100×761 | `857aece444edea6292f6ece6acdb8be456281519e0f1ffc379fba7b12e0dfd28` |
| `task-00013-settings-skills.jpg` | production Skills empty state | 1100×761 | `2f46c09b9a5629df8a27552c868e0284e932e34b6e36d749898a2827a062c55c` |
| `task-00013-settings-mcp.jpg` | production MCP empty state | 1100×761 | `a227a43238ecf13aadfca9b321c6e6b25e30bf97cb76d008056261f7e4f9adbe` |
| `task-00013-settings-intermediate-native.jpg` | 762 px contained Settings layout | 762×761 | `ba4389e13a571dfaef3594aaf1d08843b9e9168660b91ac37ba14891f17c5374` |
| `task-00013-settings-compressed-native.jpg` | 622 px compressed navigation and contained content | 622×761 | `12f4f9996a640f27ac536873132fbf891914d6945f53d5e78065481488ec29b8` |
| `task-00013-provider-save-approval-writers-disabled.jpg` | provider save approval with every writer disabled | 1102×761 | `4524985833fc68569ad3258e3796fccbec491f553c93c950e56e69003f08b626` |
| `task-00013-provider-degraded-network.jpg` | deterministic provider failure | 1102×761 | `731c5bbcaa2d618ae7d2a6e9e53ac0d74a0500b7297962a066a53014baaaf71b` |
| `task-00013-provider-recovered.jpg` | provider recovery to two models | 1102×761 | `018f23f3a89475777e286294689cfed9c4eb660b181ba965475b2b919617a4b8` |
| `task-00013-workspace-start-after-restart.jpg` | durable post-restart launch destination | 1100×761 | `0049f98318a0696b5d9845f3e56e454fe53efb51a232eb67ea882779516f8431` |
| `task-00013-session-credential-reentry-required.jpg` | missing session credential after restart | 1100×761 | `76c57050568a0788c7f68900c0772344deb661664203a54fdfe22137fff6ad49` |
| `task-00013-session-credential-reentry-after-accept.jpg` | fallback accepted while testing remains disabled | 1100×761 | `858ee7de4bb8d6ed2a894877e3eee6c031d41411604c9ed9beff34977f06062e` |
| `task-00013-session-credential-recovered-after-restart.jpg` | re-entered credential and recovered model selection | 1100×761 | `340fc4348867c72e44f5308375be32b7012d420e3c1ec83dd9fee9751ad2bab7` |

## Scope and limitations

- The provider is an isolated authenticated deterministic fixture, not an external commercial account. It proves the production connection path, credential boundary, capability normalization, failure behavior, and recovery without transmitting user data.
- Runtime Settings truthfully reported that no managed local runtime installation had been published in the fresh home. The exact bundled OpenCode/Pi execution paths are covered by the seven native OpenCode tests, four packaged runtime tests, and private-TLS native golden.
- Update, integrated diagnostics/recovery composition, final security classification, and final native/accessibility audit remain owned by Tasks 00014 through 00015C.
- The accepted Rust/security P2 is a crash-consistency window that may leave an encrypted orphan provider credential after a durable profile failure. It does not expose plaintext or bypass authority; Task 00015A must classify the residual.
- No push, pull request, signing, notarization, distribution, or deferred-scope expansion occurred.
- The unrelated `src/frontend/generated/MarketplaceSnapshot.ts` whitespace diff and Task 00006 screenshot change remain outside this evidence and checkpoint.

## Context promotion review

No Context File change is required. Task 00013 composes the already accepted launch, provider, credential, model, runtime, policy, configuration, extension, MCP, native Settings, and restart boundaries. The implementation-specific integration decision is recorded in research ledger RBL-019.

## Agent Acceptance

PASS 2026-07-23.

- Rust/security reviewer: P0=0, P1=0, P2=1, P3=0.
- Renderer/integration reviewer: P0=0, P1=0, P2=0, P3=0.
- Policy/configuration reviewer: P0=0, P1=0, P2=0, P3=0.

The only retained finding is the non-blocking encrypted-orphan crash-consistency P2 described above. Task 00013 satisfies the required P0=0 and P1=0 acceptance gate.
