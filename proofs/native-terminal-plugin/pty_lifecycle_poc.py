#!/usr/bin/env python3
"""Disposable Native Terminal Plugin POC.

This is intentionally not product code. It proves whether backend-owned PTY
lifecycle, trusted-root scoping, streaming, cancellation, cleanup, and
approval-gating are feasible with standard OS primitives.
"""

from __future__ import annotations

import fcntl
import json
import os
import pty
import select
import signal
import struct
import termios
import time
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any


REPO_ROOT = Path(__file__).resolve().parents[2]
TODAY = time.strftime("%Y-%m-%d", time.gmtime())
EVIDENCE_PATH = Path(__file__).resolve().parent / f"native-terminal-plugin-evidence-{TODAY}.md"
TRUSTED_ROOT = REPO_ROOT
DISALLOWED_ROOT = Path("/tmp").resolve()


@dataclass
class CheckResult:
    name: str
    status: str
    duration_ms: int
    evidence: dict[str, Any] = field(default_factory=dict)
    error: str | None = None


@dataclass
class PtyRun:
    command: str
    cwd: Path
    session_id: str = "poc-session"
    run_id: str = "poc-run"
    pid: int | None = None
    fd: int | None = None
    events: list[dict[str, Any]] = field(default_factory=list)
    exit_status: int | str | None = None
    cancelled: bool = False

    def start(self) -> None:
        assert_trusted_cwd(self.cwd)
        self.pid, self.fd = pty.fork()
        if self.pid == 0:
            os.chdir(self.cwd)
            os.environ.clear()
            os.environ.update(
                {
                    "PATH": "/bin:/usr/bin",
                    "TERM": "xterm-256color",
                    "C4OS_POC_SESSION_ID": self.session_id,
                    "C4OS_POC_RUN_ID": self.run_id,
                }
            )
            os.execl("/bin/bash", "bash", "--noprofile", "--norc", "-c", self.command)

        assert self.fd is not None
        flags = fcntl.fcntl(self.fd, fcntl.F_GETFL)
        fcntl.fcntl(self.fd, fcntl.F_SETFL, flags | os.O_NONBLOCK)
        self.events.append(
            {
                "kind": "started",
                "pid": self.pid,
                "cwd": str(self.cwd),
                "sessionId": self.session_id,
                "runId": self.run_id,
            }
        )

    def write_input(self, value: str) -> None:
        assert self.fd is not None
        os.write(self.fd, value.encode())
        self.events.append({"kind": "input", "bytes": len(value.encode())})

    def resize(self, rows: int, cols: int) -> None:
        assert self.fd is not None
        winsize = struct.pack("HHHH", rows, cols, 0, 0)
        fcntl.ioctl(self.fd, termios.TIOCSWINSZ, winsize)
        self.events.append({"kind": "resize", "rows": rows, "cols": cols})

    def stream_until(self, needle: str | None = None, timeout: float = 3.0) -> str:
        assert self.fd is not None
        deadline = time.monotonic() + timeout
        chunks: list[str] = []
        while time.monotonic() < deadline:
            readable, _, _ = select.select([self.fd], [], [], 0.05)
            if readable:
                try:
                    data = os.read(self.fd, 4096)
                except BlockingIOError:
                    data = b""
                except OSError:
                    break
                if not data:
                    break
                text = data.decode(errors="replace")
                chunks.append(text)
                self.events.append({"kind": "output", "text": text})
                if needle and needle in "".join(chunks):
                    break
            if self._reap_if_finished():
                break
        return "".join(chunks)

    def cancel(self) -> None:
        if self.pid is None:
            return
        try:
            os.kill(self.pid, signal.SIGTERM)
            self.cancelled = True
            self.events.append({"kind": "cancel", "signal": "SIGTERM", "target": "process"})
        except ProcessLookupError:
            pass
        if not self.wait_for_exit(1.0):
            try:
                os.kill(self.pid, signal.SIGKILL)
                self.events.append({"kind": "cancel", "signal": "SIGKILL", "target": "process"})
            except ProcessLookupError:
                pass
            self.wait_for_exit(1.0)

    def cleanup(self) -> None:
        self._reap_if_finished()
        if self.pid is not None and self.exit_status is None:
            try:
                os.kill(self.pid, signal.SIGTERM)
                self.events.append({"kind": "cleanup-signal", "signal": "SIGTERM", "target": "process"})
                time.sleep(0.05)
            except ProcessLookupError:
                pass
            if not self._reap_if_finished():
                try:
                    os.kill(self.pid, signal.SIGKILL)
                    self.events.append({"kind": "cleanup-signal", "signal": "SIGKILL", "target": "process"})
                except ProcessLookupError:
                    pass
                self._reap_if_finished()
        self._reap_if_finished()
        if self.fd is not None:
            try:
                os.close(self.fd)
            except OSError:
                pass
            self.events.append({"kind": "cleanup", "fdClosed": True})
            self.fd = None

    def _reap_if_finished(self) -> bool:
        if self.pid is None or self.exit_status is not None:
            return self.exit_status is not None
        try:
            finished_pid, status = os.waitpid(self.pid, os.WNOHANG)
        except ChildProcessError:
            return True
        if finished_pid == 0:
            return False
        if os.WIFEXITED(status):
            self.exit_status = os.WEXITSTATUS(status)
        elif os.WIFSIGNALED(status):
            self.exit_status = f"signal:{os.WTERMSIG(status)}"
        else:
            self.exit_status = status
        self.events.append({"kind": "exit", "status": self.exit_status})
        return True

    def wait_for_exit(self, timeout: float) -> bool:
        deadline = time.monotonic() + timeout
        while time.monotonic() < deadline:
            if self._reap_if_finished():
                return True
            time.sleep(0.02)
        return self._reap_if_finished()


def main() -> int:
    result: dict[str, Any] = {
        "startedAt": iso_now(),
        "trustedRoot": str(TRUSTED_ROOT),
        "disallowedRoot": str(DISALLOWED_ROOT),
        "checks": [],
        "policy": {
            "rendererDirectSpawn": "blocked by design; renderer submits requests only",
            "approvalBeforeLaunch": "required for high-impact commands",
            "trustedRoot": str(TRUSTED_ROOT),
            "environment": "minimal PATH/TERM only; provider credentials omitted",
            "outputHandling": "terminal output is untrusted event text",
        },
    }

    checks = [
        ("project-scoped PTY starts and streams output", check_streaming),
        ("PTY accepts input and associates output with session/run", check_input_and_audit),
        ("PTY resize event is observable", check_resize),
        ("trusted-root policy blocks out-of-root cwd", check_trusted_root_block),
        ("approval gate blocks high-impact command before launch", check_approval_gate_block),
        ("failure states are observable", check_failure_states),
        ("cancellation terminates and cleanup closes PTY", check_cancel_cleanup),
    ]

    for name, fn in checks:
        started = time.monotonic()
        try:
            evidence = fn()
            result["checks"].append(
                CheckResult(name, "pass", elapsed_ms(started), evidence).to_dict()
            )
        except Exception as error:  # noqa: BLE001 - POC records all failure modes.
            result["checks"].append(
                CheckResult(name, "fail", elapsed_ms(started), error=str(error)).to_dict()
            )
            result["finishedAt"] = iso_now()
            write_evidence(result)
            raise

    result["finishedAt"] = iso_now()
    result["summary"] = summarize(result["checks"])
    write_evidence(result)
    print(f"native-terminal-plugin-poc:{result['summary']['status']}")
    print(f"evidence:{EVIDENCE_PATH}")
    return 0


def check_streaming() -> dict[str, Any]:
    run = PtyRun("printf 'stream-one\\n'; printf 'stream-two\\n'", TRUSTED_ROOT)
    try:
        run.start()
        output = run.stream_until("stream-two", timeout=2.0)
        assert_contains(output, "stream-one")
        assert_contains(output, "stream-two")
        run.stream_until(timeout=0.2)
        return {"output": sanitize(output), "events": run.events, "exitStatus": run.exit_status}
    finally:
        run.cleanup()


def check_input_and_audit() -> dict[str, Any]:
    run = PtyRun("read value; printf 'echo:%s\\n' \"$value\"", TRUSTED_ROOT)
    try:
        run.start()
        run.write_input("hello-from-renderer-request\n")
        output = run.stream_until("echo:hello-from-renderer-request", timeout=2.0)
        assert_contains(output, "echo:hello-from-renderer-request")
        assert run.events[0]["sessionId"] == "poc-session"
        assert run.events[0]["runId"] == "poc-run"
        return {"output": sanitize(output), "events": run.events, "exitStatus": run.exit_status}
    finally:
        run.cleanup()


def check_resize() -> dict[str, Any]:
    run = PtyRun("printf 'resize-ready\\n'", TRUSTED_ROOT)
    try:
        run.start()
        run.resize(33, 101)
        output = run.stream_until("resize-ready", timeout=2.0)
        assert_contains(output, "resize-ready")
        resize_events = [event for event in run.events if event["kind"] == "resize"]
        assert resize_events and resize_events[0]["rows"] == 33
        return {"output": sanitize(output), "resizeEvents": resize_events}
    finally:
        run.cleanup()


def check_trusted_root_block() -> dict[str, Any]:
    try:
        assert_trusted_cwd(DISALLOWED_ROOT)
    except PermissionError as error:
        return {"blocked": True, "reason": str(error)}
    raise AssertionError("out-of-root cwd was not blocked")


def check_approval_gate_block() -> dict[str, Any]:
    decision = approval_decision("rm -rf ./target", source="agent")
    if decision["action"] != "blocked":
        raise AssertionError("high-impact command was not blocked before launch")
    return decision


def check_failure_states() -> dict[str, Any]:
    run = PtyRun("definitely-missing-command-c4os-poc", TRUSTED_ROOT)
    try:
        run.start()
        output = run.stream_until(timeout=2.0)
        assert "not found" in output or "not found" in json.dumps(run.events)
        return {"output": sanitize(output), "events": run.events, "exitStatus": run.exit_status}
    finally:
        run.cleanup()


def check_cancel_cleanup() -> dict[str, Any]:
    run = PtyRun("printf 'sleeping\\n'; exec sleep 30", TRUSTED_ROOT)
    try:
        run.start()
        output = run.stream_until("sleeping", timeout=2.0)
        assert_contains(output, "sleeping")
        run.cancel()
        if run.exit_status is None:
            raise AssertionError("cancelled PTY did not exit")
        return {
            "output": sanitize(output),
            "events": run.events,
            "exitStatus": run.exit_status,
            "cancelled": run.cancelled,
        }
    finally:
        run.cleanup()


def approval_decision(command: str, source: str) -> dict[str, Any]:
    high_impact_markers = ["rm -rf", "sudo", "curl ", "security ", "chmod -R", "chown -R"]
    if source == "agent" and any(marker in command for marker in high_impact_markers):
        return {
            "action": "blocked",
            "reason": "high-impact command requires explicit approval",
            "command": command,
            "source": source,
            "launched": False,
        }
    return {"action": "allow", "command": command, "source": source, "launched": False}


def assert_trusted_cwd(cwd: Path) -> None:
    resolved = cwd.resolve()
    if not resolved.is_relative_to(TRUSTED_ROOT):
        raise PermissionError(f"cwd outside trusted root: {resolved}")


def assert_contains(value: str, needle: str) -> None:
    if needle not in value:
        raise AssertionError(f"expected {needle!r} in {value!r}")


def elapsed_ms(started: float) -> int:
    return int((time.monotonic() - started) * 1000)


def summarize(checks: list[dict[str, Any]]) -> dict[str, Any]:
    passed = sum(1 for check in checks if check["status"] == "pass")
    failed = sum(1 for check in checks if check["status"] == "fail")
    return {"status": "passed" if failed == 0 else "failed", "passed": passed, "failed": failed}


def sanitize(value: str) -> str:
    return value.replace(str(REPO_ROOT), "$REPO_ROOT")


def iso_now() -> str:
    return time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())


def write_evidence(result: dict[str, Any]) -> None:
    summary = result.get("summary") or summarize(result["checks"])
    lines = [
        f"# Native Terminal Plugin POC Evidence: {TODAY}",
        "",
        f"Status: {summary['status']}",
        f"Started: {result['startedAt']}",
        f"Finished: {result.get('finishedAt', '')}",
        "",
        "## Scope",
        "",
        "Disposable stdlib PTY lifecycle POC for backend-owned terminal",
        "execution semantics. This is not product Terminal implementation.",
        "",
        "## Checks",
        "",
        *[
            f"- {check['status'].upper()}: {check['name']} ({check['duration_ms']}ms)"
            for check in result["checks"]
        ],
        "",
        "## Security Findings",
        "",
        f"- Renderer direct spawn: {result['policy']['rendererDirectSpawn']}",
        f"- Approval before launch: {result['policy']['approvalBeforeLaunch']}",
        f"- Trusted root: {result['policy']['trustedRoot']}",
        f"- Environment: {result['policy']['environment']}",
        f"- Output handling: {result['policy']['outputHandling']}",
        "",
        "## Limits",
        "",
        "- This POC uses Python stdlib PTY primitives, not Rust/Tauri PTY crates.",
        "- It proves backend-owned PTY semantics are viable on this macOS host.",
        "- Cross-platform Windows/Linux behavior and Tauri capability wiring",
        "  still require a follow-up native Rust/Tauri POC.",
        "",
        "## Raw Result",
        "",
        "```json",
        json.dumps(result, indent=2),
        "```",
        "",
    ]
    EVIDENCE_PATH.write_text("\n".join(lines), encoding="utf-8")


def _check_result_to_dict(self: CheckResult) -> dict[str, Any]:
    payload = {
        "name": self.name,
        "status": self.status,
        "duration_ms": self.duration_ms,
    }
    if self.evidence:
        payload["evidence"] = self.evidence
    if self.error:
        payload["error"] = self.error
    return payload


CheckResult.to_dict = _check_result_to_dict  # type: ignore[attr-defined]


if __name__ == "__main__":
    raise SystemExit(main())
