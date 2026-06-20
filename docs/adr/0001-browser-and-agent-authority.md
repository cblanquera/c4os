# Browser And Agent Authority

Status: accepted

C4OS treats Browser as a user-owned desktop surface, not a restricted preview-only frame. Users may browse local files and public websites, Browser session state is remembered per trusted project, and agents may read/write active-project files or browse public/logged-in pages when the user's request clearly requires it; outside-project file access requires explicit permission and agent file/Browser actions must be recorded. This deliberately favors desktop-user agency and product viability over the earlier constrained-preview-only Browser boundary.

## Consequences

- Browser is MVP scope despite earlier POC risk findings.
- Downloads are excluded from MVP.
- Agent authority, permissions, and audit records are the controlling boundary.
- Implementation must avoid exposing provider secrets, shell state, or app internals through Browser content even though user/agent browsing scope is broad.
