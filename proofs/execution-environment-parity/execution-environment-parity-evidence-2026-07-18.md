# Execution Environment Parity Evidence — 2026-07-18

## Commands

```sh
node --test proofs/execution-environment-parity/proof.test.mjs
docker version --format '{{json .}}'
ssh -V
```

## Observed signal

- 7 tests passed; 0 failed, skipped, cancelled, or todo.
- Local Desktop wrote and read a real temporary file and cancelled a real Node child process.
- Local, Docker, and SSH contract adapters emitted the same journey categories.
- Runtime paths and artifact IDs/provenance were environment-qualified.
- Host paths did not appear in Docker or SSH runtime paths.
- Approval could not replay across environment, generation, action, or second use.
- Only credential references crossed the contract; no raw-secret pattern appeared.
- Docker client/daemon `20.10.11` ran a digest-pinned Alpine 3.20 container with networking disabled, performed the bind-mounted write, received cancellation, and was removed.
- OpenSSH `10.2p1` ran an isolated loopback server using generated Ed25519 host/client keys and strict host-key verification. The remote process received an explicit authenticated cancellation command and stopped.

## Result

**Passed for Local Desktop, Docker, and the real OpenSSH transport fixture.** A named external host still requires host-specific validation before that host-backed feature is enabled.
