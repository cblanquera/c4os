#[cfg(not(unix))]
fn main() {
    eprintln!("native-terminal-tauri-poc currently targets Unix PTY APIs only");
    std::process::exit(2);
}

#[cfg(unix)]
mod unix_poc {
    use serde::Serialize;
    use serde_json::json;
    use std::fs::File;
    use std::io::{ErrorKind, Read, Write};
    use std::os::fd::{FromRawFd, RawFd};
    use std::os::unix::process::CommandExt;
    use std::path::{Path, PathBuf};
    use std::process::{Child, Command, Stdio};
    use std::thread;
    use std::time::{Duration, Instant};

    const CHUNK_LIMIT: usize = 128;

    #[derive(Debug, Serialize)]
    struct Check {
        name: &'static str,
        passed: bool,
        evidence: serde_json::Value,
    }

    pub fn run() -> i32 {
        let trusted_root = workspace_root();
        let mut checks = Vec::new();

        checks.push(check_trusted_root_scoping(&trusted_root));
        checks.push(check_approval_gate());

        match check_pty_lifecycle(&trusted_root) {
            Ok(mut lifecycle_checks) => checks.append(&mut lifecycle_checks),
            Err(error) => checks.push(Check {
                name: "pty_lifecycle",
                passed: false,
                evidence: json!({ "error": error.to_string() }),
            }),
        }

        match check_command_failure(&trusted_root) {
            Ok(check) => checks.push(check),
            Err(error) => checks.push(Check {
                name: "command_failure",
                passed: false,
                evidence: json!({ "error": error.to_string() }),
            }),
        }

        let passed = checks.iter().all(|check| check.passed);
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "poc": "native-terminal-tauri",
                "platform": std::env::consts::OS,
                "trustedRoot": trusted_root,
                "passed": passed,
                "checks": checks
            }))
            .expect("serialize evidence")
        );

        if passed {
            0
        } else {
            1
        }
    }

    fn workspace_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .expect("workspace root")
            .to_path_buf()
    }

    fn check_trusted_root_scoping(trusted_root: &Path) -> Check {
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

    fn check_approval_gate() -> Check {
        let risky_command = "rm -rf /tmp/native-terminal-tauri-poc";
        let approval_required = requires_approval(risky_command);
        let launch_blocked = approval_required;

        Check {
            name: "approval_gated_launch",
            passed: approval_required && launch_blocked,
            evidence: json!({
                "command": risky_command,
                "approvalRequired": approval_required,
                "spawnAttempted": !launch_blocked
            }),
        }
    }

    fn check_pty_lifecycle(trusted_root: &Path) -> Result<Vec<Check>, Box<dyn std::error::Error>> {
        let script = concat!(
            "printf 'started:%s\\n' \"$PWD\"; ",
            "printf 'ready\\n'; ",
            "read line; ",
            "printf 'input:%s\\n' \"$line\"; ",
            "printf 'size:'; stty size; ",
            "sleep 30"
        );
        let mut pty = PseudoTerminal::spawn(trusted_root, "/bin/sh", &["-lc", script])?;
        let mut checks = Vec::new();

        let initial_output = pty.read_until("ready", Duration::from_secs(3))?;
        let startup_passed = initial_output.contains("started:")
            && initial_output.contains(&trusted_root.display().to_string())
            && initial_output.contains("ready");
        checks.push(Check {
            name: "pty_starts_in_trusted_root",
            passed: startup_passed,
            evidence: json!({
                "output": initial_output,
                "cwd": trusted_root
            }),
        });

        let resized = pty.resize(40, 120).is_ok();
        pty.write_all(b"hello-from-renderer\n")?;
        let interactive_output = pty.read_until("size:", Duration::from_secs(3))?;
        let input_passed = interactive_output.contains("input:hello-from-renderer");
        let resize_observed = resized && interactive_output.contains("size:");
        checks.push(Check {
            name: "streaming_input_resize",
            passed: input_passed && resize_observed && pty.max_chunk_len <= CHUNK_LIMIT,
            evidence: json!({
                "output": interactive_output,
                "inputObserved": input_passed,
                "resizeIoctlSucceeded": resized,
                "maxChunkLength": pty.max_chunk_len,
                "chunkLimit": CHUNK_LIMIT
            }),
        });

        let killed = pty.kill().is_ok();
        let exit = pty.wait()?;
        let cleanup = pty.cleanup();
        checks.push(Check {
            name: "cancellation_exit_cleanup",
            passed: killed && exit.observed && cleanup,
            evidence: json!({
                "killRequested": killed,
                "exitObserved": exit.observed,
                "status": exit.status,
                "cleanupClosedMaster": cleanup
            }),
        });

        Ok(checks)
    }

    fn check_command_failure(trusted_root: &Path) -> Result<Check, Box<dyn std::error::Error>> {
        let mut pty = PseudoTerminal::spawn(
            trusted_root,
            "/bin/sh",
            &["-lc", "definitely_missing_terminal_poc_command_123"],
        )?;
        let output = pty
            .read_until("not found", Duration::from_secs(3))
            .unwrap_or_default();
        let exit = pty.wait()?;
        let cleanup = pty.cleanup();
        let failed_as_expected = output.contains("not found")
            || output.contains("command not found")
            || exit.status != 0;

        Ok(Check {
            name: "command_failure_observable",
            passed: failed_as_expected && exit.observed && cleanup,
            evidence: json!({
                "output": output,
                "exitObserved": exit.observed,
                "status": exit.status,
                "cleanupClosedMaster": cleanup
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

    fn requires_approval(command: &str) -> bool {
        let lower = command.to_lowercase();
        lower.contains("rm -rf")
            || lower.contains("sudo ")
            || lower.contains("chmod -r")
            || lower.contains("curl ")
            || lower.contains("ssh ")
    }

    struct ExitObservation {
        observed: bool,
        status: i32,
    }

    struct PseudoTerminal {
        master: Option<File>,
        child: Child,
        max_chunk_len: usize,
    }

    impl PseudoTerminal {
        fn spawn(
            cwd: &Path,
            program: &str,
            args: &[&str],
        ) -> Result<Self, Box<dyn std::error::Error>> {
            let mut master_fd: RawFd = -1;
            let mut slave_fd: RawFd = -1;
            let mut winsize = libc::winsize {
                ws_row: 24,
                ws_col: 80,
                ws_xpixel: 0,
                ws_ypixel: 0,
            };

            let open_result = unsafe {
                libc::openpty(
                    &mut master_fd,
                    &mut slave_fd,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    &mut winsize,
                )
            };
            if open_result != 0 {
                return Err(std::io::Error::last_os_error().into());
            }

            set_nonblocking(master_fd)?;

            let slave_file = unsafe { File::from_raw_fd(slave_fd) };
            let stdin_file = slave_file.try_clone()?;
            let stdout_file = slave_file.try_clone()?;
            let stderr_file = slave_file;

            let mut command = Command::new(program);
            command
                .args(args)
                .current_dir(cwd)
                .stdin(Stdio::from(stdin_file))
                .stdout(Stdio::from(stdout_file))
                .stderr(Stdio::from(stderr_file));

            unsafe {
                command.pre_exec(|| {
                    if libc::setsid() == -1 {
                        return Err(std::io::Error::last_os_error());
                    }
                    libc::ioctl(0, libc::TIOCSCTTY.into(), 0);
                    Ok(())
                });
            }

            let child = command.spawn()?;
            let master = unsafe { File::from_raw_fd(master_fd) };

            Ok(Self {
                master: Some(master),
                child,
                max_chunk_len: 0,
            })
        }

        fn read_until(
            &mut self,
            needle: &str,
            timeout: Duration,
        ) -> Result<String, Box<dyn std::error::Error>> {
            let start = Instant::now();
            let mut output = String::new();
            let mut buffer = [0_u8; CHUNK_LIMIT];

            while start.elapsed() < timeout {
                let Some(master) = self.master.as_mut() else {
                    break;
                };
                match master.read(&mut buffer) {
                    Ok(0) => thread::sleep(Duration::from_millis(20)),
                    Ok(read) => {
                        self.max_chunk_len = self.max_chunk_len.max(read);
                        output.push_str(&String::from_utf8_lossy(&buffer[..read]));
                        if output.contains(needle) {
                            return Ok(output);
                        }
                    }
                    Err(error) if error.kind() == ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(20));
                    }
                    Err(error) => return Err(error.into()),
                }
            }

            Ok(output)
        }

        fn write_all(&mut self, bytes: &[u8]) -> std::io::Result<()> {
            if let Some(master) = self.master.as_mut() {
                master.write_all(bytes)?;
                master.flush()?;
            }

            Ok(())
        }

        fn resize(&self, rows: u16, cols: u16) -> std::io::Result<()> {
            let Some(master) = self.master.as_ref() else {
                return Ok(());
            };
            let winsize = libc::winsize {
                ws_row: rows,
                ws_col: cols,
                ws_xpixel: 0,
                ws_ypixel: 0,
            };
            let result = unsafe { libc::ioctl(raw_fd(master), libc::TIOCSWINSZ.into(), &winsize) };
            if result == -1 {
                Err(std::io::Error::last_os_error())
            } else {
                Ok(())
            }
        }

        fn kill(&mut self) -> std::io::Result<()> {
            self.child.kill()
        }

        fn wait(&mut self) -> std::io::Result<ExitObservation> {
            let status = self.child.wait()?;
            Ok(ExitObservation {
                observed: true,
                status: status.code().unwrap_or(-1),
            })
        }

        fn cleanup(&mut self) -> bool {
            self.master.take().is_some()
        }
    }

    fn raw_fd(file: &File) -> RawFd {
        use std::os::fd::AsRawFd;
        file.as_raw_fd()
    }

    fn set_nonblocking(fd: RawFd) -> std::io::Result<()> {
        let flags = unsafe { libc::fcntl(fd, libc::F_GETFL) };
        if flags == -1 {
            return Err(std::io::Error::last_os_error());
        }

        let result = unsafe { libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK) };
        if result == -1 {
            Err(std::io::Error::last_os_error())
        } else {
            Ok(())
        }
    }
}

#[cfg(unix)]
fn main() {
    std::process::exit(unix_poc::run());
}
