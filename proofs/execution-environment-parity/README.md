# Execution Environment Parity Proof

Status: passed for Local, Docker, and real loopback OpenSSH transports
Updated: 2026-07-18

## Question

Can Local Desktop, Docker, and Remote SSH preserve one file/terminal/approval/artifact contract while keeping paths, credentials, generations, and provenance environment-qualified?

## Run

```sh
node --test proofs/execution-environment-parity/proof.test.mjs
docker version --format '{{json .}}'
ssh -V
```

## Evidence boundary

The Local Desktop journey writes a real temporary file and cancels a real child process. The Docker journey uses a digest-pinned Alpine 3.20 image, a bind-mounted temporary workspace, disabled networking, a credential reference, and real container cancellation/removal. The SSH journey starts an isolated loopback OpenSSH server with generated host/client keys, strict host-key verification, a credential reference, a real remote write, disconnect, and an authenticated cancellation control message.

## Result

**Passed for the three tested execution transports** on 2026-07-18. The common contract, environment-qualified paths/artifacts, non-replayable approvals, credential references, real writes, and cancellation paths pass for Local Desktop, Docker, and loopback OpenSSH.

An external SSH host is still a deployment fixture, not a missing transport proof. Enabling a particular remote host must remain gated on that host's authentication, filesystem, shell, and cancellation validation.

See `execution-environment-parity-evidence-2026-07-18.md`.
