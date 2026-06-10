# FINDING-003 Shell Security Policy

Date: 2026-06-10

## Question

Are MVP shell redaction, truncation, destructive-command classification,
environment filtering, and unclassifiable-command rules concrete enough for
implementation and testing?

## Decision

Shell execution remains in MVP under the current-user, normal-network posture,
but only with the mandatory policy below.

The policy does not claim sandboxing, network isolation, perfect shell
classification, automatic rollback, or safe raw-output retention.

## Environment Filtering

MVP shell commands receive a backend-built environment. They must not inherit
the full interactive shell environment.

### Always Keep

| Variable | Reason |
| --- | --- |
| `PATH` | Required for local toolchains. Must be backend-constructed or sanitized. |
| `HOME` | Required by common tools. |
| `USER` | Required by local tooling. |
| `LOGNAME` | Common POSIX identity variable. |
| `SHELL` | Required by shell-launched tools. |
| `PWD` | Set to approved working directory. |
| `LANG` | Locale stability. |
| `LC_*` | Locale stability. |
| `TERM` | Terminal compatibility. |
| `TMPDIR` | Local temp behavior. |
| `TMP` | Local temp behavior. |
| `TEMP` | Local temp behavior. |

### Conditionally Keep

Keep these only when present and not secret-shaped:

| Variable | Condition |
| --- | --- |
| `NODE_OPTIONS` | Allowed only if value does not contain `--require`, `--import`, `--eval`, inspector flags, or file paths outside project/toolchain roots. Otherwise strip. |
| `NPM_CONFIG_*` | Keep non-auth config such as registry/cache/proxy only after secret-name filtering. Strip auth tokens and password-like values. |
| `YARN_*` | Keep non-auth tool config after secret-name filtering. |
| `PNPM_*` | Keep non-auth tool config after secret-name filtering. |
| `CARGO_*` | Keep non-auth build config after secret-name filtering. |
| `RUSTUP_*` | Keep non-auth toolchain config after secret-name filtering. |
| `GIT_*` | Keep only non-auth display/config vars such as `GIT_TERMINAL_PROMPT=0`; strip credential helpers, askpass, author tokens, and command injection hooks. |
| `CI` | Keep for deterministic test behavior. |
| `NO_COLOR` | Keep for output readability. |
| `FORCE_COLOR` | Keep only if UI can handle ANSI safely; otherwise strip. |
| Proxy variables | `HTTP_PROXY`, `HTTPS_PROXY`, `NO_PROXY`, lowercase variants may be kept only if the approval prompt discloses normal network access and values are not credential URLs. |

### Always Strip

Strip variables whose names match these case-insensitive patterns:

- `*_KEY`
- `*_TOKEN`
- `*_SECRET`
- `*_PASSWORD`
- `*_PASS`
- `*_PWD`
- `*_CREDENTIAL*`
- `*_AUTH*`
- `*_SESSION*`
- `*_COOKIE*`
- `*_BEARER*`
- `*_PRIVATE*`
- `OPENAI_*`
- `OPENROUTER_*`
- `ANTHROPIC_*`
- `GOOGLE_API_KEY`
- `GEMINI_*`
- `AWS_ACCESS_KEY_ID`
- `AWS_SECRET_ACCESS_KEY`
- `AWS_SESSION_TOKEN`
- `AWS_PROFILE`
- `AZURE_*`
- `GCP_*`
- `GOOGLE_APPLICATION_CREDENTIALS`
- `GH_TOKEN`
- `GITHUB_TOKEN`
- `GIT_ASKPASS`
- `SSH_ASKPASS`
- `NPM_TOKEN`
- `NODE_AUTH_TOKEN`
- `PYPI_TOKEN`
- `TWINE_PASSWORD`
- `DOCKER_*TOKEN*`
- `KUBECONFIG`

Also strip any variable whose value matches a secret-shaped redaction rule.

## Redaction Matchers

Redaction applies before persisted summaries, model context, approval records,
and logs are written.

Mandatory matcher categories:

| Category | Examples |
| --- | --- |
| Known secret values | Values loaded from keychain references or stripped env vars. |
| Provider keys | `sk-...`, `sk-or-...`, OpenAI/OpenRouter/Anthropic-like key prefixes. |
| GitHub tokens | `ghp_`, `gho_`, `ghu_`, `ghs_`, `github_pat_`. |
| Cloud keys | AWS access keys, Azure/GCP token-shaped strings, bearer tokens. |
| Package tokens | npm, PyPI, Docker, registry auth tokens. |
| Private keys | PEM blocks such as `-----BEGIN ... PRIVATE KEY-----`. |
| Credential URLs | URLs containing `user:password@host`. |
| JWT-like values | Three base64url segments separated by dots. |
| Long high-entropy values | Alphanumeric/base64/base64url strings of 32+ chars with entropy above implementation threshold. |
| SSH material | OpenSSH private keys and `ssh-rsa`/`ssh-ed25519` private material. |

Known limitations:

- Redaction is best effort and pattern-based.
- Redaction does not prove a command is safe.
- Redaction must prefer false positives over secret leakage.
- If redaction produces ambiguous or unstable output, persistence fails closed
  to an output-omitted marker.

## Truncation And Output Omission

Persisted shell output records use bounded summaries only.

MVP limits:

| Field | Limit |
| --- | --- |
| stdout summary | 24 KiB after redaction |
| stderr summary | 24 KiB after redaction |
| combined persisted output summary | 32 KiB after redaction |
| maximum persisted lines | 400 combined stdout/stderr lines |
| maximum line length | 1,000 characters after redaction |
| command string in persisted metadata | 4 KiB |
| working directory path | 1 KiB |
| affected-file summary | 200 paths or 16 KiB, whichever comes first |
| approval prompt command preview | 2 KiB with clear truncation marker |

If output exceeds a limit:

- keep the earliest useful failure/status lines and the final tail lines;
- include safe labels, not raw hidden byte counts;
- preserve exit code, command status, start/end timestamps, and working
  directory;
- never store raw stdout/stderr as fallback.

Persist only this omission marker when safe summarization fails:

```text
output_omitted
```

with allowed labels from the safe-label list below.

## Safe Reason Labels

Allowed labels:

- `truncated_by_size`
- `truncated_by_line_count`
- `truncated_by_line_length`
- `redacted_known_secret`
- `redacted_secret_pattern`
- `redacted_credential_url`
- `redacted_private_key`
- `output_omitted_safe_summary_failed`
- `output_omitted_binary_or_control_data`
- `output_omitted_untrusted_encoding`
- `output_omitted_redaction_uncertain`
- `metadata_omitted_path_limit`

Forbidden labels:

- labels containing raw secret names when the name itself is sensitive;
- labels containing redacted values;
- labels containing raw byte offsets, hashes, entropy scores, or exact hidden
  byte counts.

## Forbidden Persisted Metadata

Do not persist:

- raw stdout or stderr;
- full live terminal buffers;
- redacted substrings;
- raw secret values;
- sensitive raw byte counts;
- offsets of redacted matches;
- hashes/fingerprints of redacted secrets;
- raw prompt replay blobs;
- shell process environment dumps;
- full command-expanded scripts generated from aliases/functions;
- raw output export bundles;
- dedicated raw-output copy payloads.

## Destructive Command Classification

High-risk shell commands require fresh one-time approval every time. Session
allow must not apply.

| Category | Examples | Handling |
| --- | --- | --- |
| Recursive deletion | `rm -rf`, `find . -delete`, `git clean -fdx` | Fresh one-time approval. |
| Broad deletion target | `/`, `$HOME`, `~`, project root, `..`, glob roots like `*` | Fresh one-time approval or block if outside root. |
| Recursive permission/ownership change | `chmod -R`, `chown -R`, `chgrp -R` | Fresh one-time approval. |
| Disk/device writes | `dd`, `mkfs`, `diskutil erase*`, writes to `/dev/*` | Block or fresh one-time approval only if in MVP scope; default block. |
| Force Git state changes | `git reset --hard`, `git checkout -f`, `git restore --source`, `git clean`, branch delete | Fresh one-time approval and Git-state classification. |
| Package removal/prune | `npm uninstall`, `npm prune`, `pnpm remove`, `yarn remove`, `pip uninstall`, `cargo uninstall` | Fresh one-time approval. |
| Lockfile/package manager mutation | install/update/add/remove commands | Approval required; destructive if removal/prune/force. |
| Process killing | `kill -9`, `pkill`, `killall` | Fresh one-time approval. |
| Network exfiltration shape | `curl ... | sh`, `wget ... | sh`, upload commands with broad paths | Fresh one-time approval or block when unclassifiable. |
| Encoded/eval execution | `eval`, `bash -c` with generated input, `python -c`, base64 decode piped to shell | Treat as unclassifiable high risk. |

Non-examples that may be session-allow eligible when scoped inside project and
not otherwise risky:

- `npm test`
- `npm run build`
- `pnpm test`
- `yarn test`
- `cargo test`
- `go test ./...`
- `pytest`
- `git status`
- `git diff`
- `ls`

Git inspection is not approval-gated, but runtime-performed inspection remains
logged. Git state changes are never covered by ordinary shell session allow.

## Unclassifiable Command Handling

Treat these as conservative high risk:

- shell substitutions: `` `...` `` and `$(...)`;
- eval-like execution: `eval`, `exec`, `source` with dynamic target;
- encoded commands: base64, hex, compressed payloads piped to interpreters;
- nested interpreters: `sh -c`, `bash -c`, `node -e`, `python -c`, `ruby -e`;
- generated scripts written then executed in one command;
- aliases/functions that the backend cannot inspect;
- broad globs affecting many paths;
- command strings with control characters or untrusted encodings;
- here-doc scripts that include destructive or opaque bodies;
- commands that depend on external shell startup files.

Handling:

- do not apply session allow;
- show one-time approval only when target scope is still inside project and
  command is not explicitly blocked;
- block if outside-root, secret-deny, or target scope cannot be determined;
- label prompt with `classification_uncertain`;
- return structured denial when blocked.

## Approval Prompt Requirements

Every shell approval prompt must show:

- command preview;
- working directory;
- risk category;
- whether session allow is available;
- current OS user execution disclosure;
- filtered environment disclosure;
- normal network access disclosure;
- no automatic rollback disclosure;
- affected path summary when available;
- classification uncertainty when applicable.

Destructive or unclassifiable prompts must offer only:

- allow once;
- deny.

They must not offer session allow.

## Runtime Boundary Requirements

These must be enforced before implementation is considered ready:

- backend approval happens before shell execution;
- denied shell command produces structured runtime denial;
- shell environment is backend-built and filtered;
- persisted shell output uses bounded redacted/truncated summaries;
- summary failure stores output omission, not raw output;
- later model context receives only persisted summary, omission marker, and
  safe reason labels.

## Matrix Result

Mark these gates as passed as policy gates:

- Shell environment policy.
- Shell redaction policy.
- Shell truncation policy.
- Destructive command policy.
- Unclassifiable command handling.

Implementation still needs tests for these rules. This artifact resolves the
planning gap by making the rules exact enough to implement and verify.
