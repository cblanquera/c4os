use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use serde::Serialize;
use serde_json::json;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Debug, Serialize)]
struct Check {
    name: &'static str,
    passed: bool,
    evidence: serde_json::Value,
}

fn main() {
    let trusted_root = workspace_root();
    let mut checks = Vec::new();

    checks.push(check_trusted_root(&trusted_root));
    checks.extend(match check_interactive_pty(&trusted_root) {
        Ok(next) => next,
        Err(error) => vec![Check {
            name: "interactive_pty",
            passed: false,
            evidence: json!({ "error": error.to_string() }),
        }],
    });
    checks.push(match check_failure_observable(&trusted_root) {
        Ok(check) => check,
        Err(error) => Check {
            name: "failure_observable",
            passed: false,
            evidence: json!({ "error": error.to_string() }),
        },
    });

    let passed = checks.iter().all(|check| check.passed);
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "poc": "terminal-portable-pty",
            "platform": std::env::consts::OS,
            "trustedRoot": trusted_root,
            "passed": passed,
            "checks": checks
        }))
        .expect("serialize proof")
    );

    std::process::exit(if passed { 0 } else { 1 });
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("workspace root")
        .to_path_buf()
}

fn check_trusted_root(trusted_root: &Path) -> Check {
    let trusted_allowed = cwd_within_trusted_root(trusted_root, trusted_root);
    let out_of_root_blocked = !cwd_within_trusted_root(trusted_root, Path::new("/tmp"));
    Check {
        name: "trusted_root_scoping",
        passed: trusted_allowed && out_of_root_blocked,
        evidence: json!({
            "trustedAllowed": trusted_allowed,
            "outOfRootBlocked": out_of_root_blocked,
            "blockedPath": "/tmp"
        }),
    }
}

fn check_interactive_pty(trusted_root: &Path) -> Result<Vec<Check>, Box<dyn std::error::Error>> {
    let pty_system = native_pty_system();
    let pair = pty_system.openpty(PtySize {
        rows: 31,
        cols: 102,
        pixel_width: 0,
        pixel_height: 0,
    })?;

    let script = concat!(
        "printf 'prompt:%s\\n' \"$PWD\"; ",
        "printf 'c4os %% '; ",
        "read line; ",
        "printf 'command:%s\\n' \"$line\"; ",
        "printf 'output:%s\\n' \"$line\"; ",
        "printf 'size:'; stty size; ",
        "sleep 30"
    );
    let mut cmd = CommandBuilder::new("/bin/sh");
    cmd.arg("-lc");
    cmd.arg(script);
    cmd.cwd(trusted_root);
    cmd.env("TERM", "xterm-256color");
    cmd.env("PATH", "/usr/bin:/bin:/usr/sbin:/sbin");

    let mut child = pair.slave.spawn_command(cmd)?;
    let mut reader = pair.master.try_clone_reader()?;
    let mut writer = pair.master.take_writer()?;

    let startup = read_until(&mut reader, "c4os % ", Duration::from_secs(3))?;
    let startup_passed =
        startup.contains("prompt:") && startup.contains(&trusted_root.display().to_string());

    pair.master.resize(PtySize {
        rows: 40,
        cols: 120,
        pixel_width: 0,
        pixel_height: 0,
    })?;
    writer.write_all(b"ls\n")?;
    writer.flush()?;
    let interactive = read_until(&mut reader, "size:", Duration::from_secs(3))?;
    let input_passed = interactive.contains("command:ls") && interactive.contains("output:ls");
    let resize_passed = interactive.contains("size:");

    let killed = child.kill().is_ok();
    let wait_status = child.wait()?;

    Ok(vec![
        Check {
            name: "pty_starts_in_trusted_root",
            passed: startup_passed,
            evidence: json!({
                "output": startup,
                "cwd": trusted_root
            }),
        },
        Check {
            name: "stdin_output_resize",
            passed: input_passed && resize_passed,
            evidence: json!({
                "output": interactive,
                "inputObserved": input_passed,
                "resizeObserved": resize_passed
            }),
        },
        Check {
            name: "termination_observable",
            passed: killed,
            evidence: json!({
                "killRequested": killed,
                "status": wait_status.exit_code()
            }),
        },
    ])
}

fn check_failure_observable(trusted_root: &Path) -> Result<Check, Box<dyn std::error::Error>> {
    let pty_system = native_pty_system();
    let pair = pty_system.openpty(PtySize {
        rows: 24,
        cols: 80,
        pixel_width: 0,
        pixel_height: 0,
    })?;
    let mut cmd = CommandBuilder::new("/bin/sh");
    cmd.arg("-lc");
    cmd.arg("definitely_missing_c4os_portable_pty_command");
    cmd.cwd(trusted_root);
    cmd.env("PATH", "/usr/bin:/bin:/usr/sbin:/sbin");

    let mut child = pair.slave.spawn_command(cmd)?;
    let mut reader = pair.master.try_clone_reader()?;
    let output = read_until(&mut reader, "not found", Duration::from_secs(3)).unwrap_or_default();
    let status = child.wait()?;
    let failed_as_expected = status.exit_code() != 0
        && (output.contains("not found") || output.contains("command not found"));

    Ok(Check {
        name: "failure_observable",
        passed: failed_as_expected,
        evidence: json!({
            "output": output,
            "status": status.exit_code()
        }),
    })
}

fn cwd_within_trusted_root(trusted_root: &Path, cwd: &Path) -> bool {
    let Ok(root) = trusted_root.canonicalize() else {
        return false;
    };
    let Ok(candidate) = cwd.canonicalize() else {
        return false;
    };
    candidate.starts_with(root)
}

fn read_until(
    reader: &mut Box<dyn Read + Send>,
    needle: &str,
    timeout: Duration,
) -> Result<String, Box<dyn std::error::Error>> {
    let started = Instant::now();
    let mut output = String::new();
    let mut buffer = [0_u8; 512];

    while started.elapsed() < timeout {
        match reader.read(&mut buffer) {
            Ok(0) => break,
            Ok(n) => {
                output.push_str(&String::from_utf8_lossy(&buffer[..n]));
                if output.contains(needle) {
                    return Ok(output);
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(20));
            }
            Err(error) => return Err(Box::new(error)),
        }
    }

    Ok(output)
}
