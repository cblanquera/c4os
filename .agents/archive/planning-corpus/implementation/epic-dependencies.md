# Epic Dependencies

## Dependency Graph

```mermaid
flowchart TD
  E1["Epic 1: App Shell And Local Foundation"]
  E2["Epic 2: Project And Git Boundary"]
  E3["Epic 3: Provider And Model Setup"]
  E4["Epic 4: Hardened OpenCode Adapter"]
  E5["Epic 5: Approval Gateway And Policy Engine"]
  E6["Epic 6: File, Shell, And Tool Execution"]
  E7["Epic 7: Session UI And User Control"]
  E8["Epic 8: Review Surfaces And Artifacts"]
  E9["Epic 9: MVP QA And Release Gates"]

  E1 --> E2
  E1 --> E3
  E1 --> E4
  E3 --> E4
  E2 --> E5
  E4 --> E5
  E2 --> E6
  E5 --> E6
  E4 --> E7
  E5 --> E7
  E6 --> E7
  E2 --> E8
  E6 --> E8
  E7 --> E8
  E8 --> E9
```

## Critical Path

1. Epic 1: App Shell And Local Foundation.
2. Epic 3: Provider And Model Setup.
3. Epic 4: Hardened OpenCode Adapter.
4. Epic 5: Approval Gateway And Policy Engine.
5. Epic 6: File, Shell, And Tool Execution.
6. Epic 7: Session UI And User Control.
7. Epic 8: Review Surfaces And Artifacts.
8. Epic 9: MVP QA And Release Gates.

Epic 2 should run immediately after Epic 1 because project-root path resolution
is needed by approval policy, shell working-directory policy, Git display, and
file access.

## Dependency Rules

- No runtime tool execution before project-root path resolution exists.
- No protected action execution before Approval Gateway exists.
- No model-backed session before OpenRouter credential reference capture exists.
- No OpenCode-backed session start before instruction-source disclosure exists.
- No public MVP release before Tauri app-build validation and product QA pass.
