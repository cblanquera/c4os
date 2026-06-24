use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::path::Path;
use std::sync::{Arc, Mutex, OnceLock};
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminalOutputEvent {
    pub session_id: String,
    pub terminal_kind: String,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminalResize {
    pub rows: u16,
    pub cols: u16,
}

struct PtyHandle {
    writer: Mutex<Box<dyn Write + Send>>,
    pty: Mutex<Box<dyn portable_pty::MasterPty + Send>>,
    child: Mutex<Box<dyn portable_pty::Child + Send>>,
    output: Mutex<String>,
}

type PtyStore = HashMap<String, Arc<PtyHandle>>;

static PTYS: OnceLock<Mutex<PtyStore>> = OnceLock::new();

pub fn start_terminal<F>(
    session_id: &str,
    terminal_kind: &str,
    root: &Path,
    emit: F,
) -> Result<(), String>
where
    F: Fn(TerminalOutputEvent) + Send + 'static,
{
    let key = terminal_key(session_id, terminal_kind);
    if ptys()
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .contains_key(&key)
    {
        return Ok(());
    }

    let pty_system = native_pty_system();
    let pair = pty_system
        .openpty(PtySize {
            rows: 31,
            cols: 102,
            pixel_width: 0,
            pixel_height: 0,
        })
        .map_err(|error| format!("Terminal PTY failed to open: {error}"))?;
    let shell = shell_program();
    let mut command = CommandBuilder::new(&shell);
    command.arg("-i");
    command.cwd(root);
    command.env("C4OS_TRUSTED_ROOT", root.to_string_lossy().as_ref());
    command.env("PS1", "cblanquera@Chris-MacBook-2022 c4os % ");
    command.env("TERM", "xterm-256color");
    command.env("PATH", "/usr/bin:/bin:/usr/sbin:/sbin");
    let child = pair
        .slave
        .spawn_command(command)
        .map_err(|error| format!("Terminal shell failed to start: {error}"))?;
    let mut reader = pair
        .master
        .try_clone_reader()
        .map_err(|error| format!("Terminal reader failed to start: {error}"))?;
    let writer = pair
        .master
        .take_writer()
        .map_err(|error| format!("Terminal writer failed to start: {error}"))?;
    let handle = Arc::new(PtyHandle {
        writer: Mutex::new(writer),
        pty: Mutex::new(pair.master),
        child: Mutex::new(child),
        output: Mutex::new(String::new()),
    });
    ptys()
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .insert(key, Arc::clone(&handle));

    let event_session = session_id.to_string();
    let event_kind = terminal_kind.to_string();
    thread::spawn(move || {
        let mut buffer = [0_u8; 4096];
        loop {
            match reader.read(&mut buffer) {
                Ok(0) => break,
                Ok(bytes) => {
                    let text = String::from_utf8_lossy(&buffer[..bytes]).to_string();
                    handle
                        .output
                        .lock()
                        .unwrap_or_else(|error| error.into_inner())
                        .push_str(&text);
                    emit(TerminalOutputEvent {
                        session_id: event_session.clone(),
                        terminal_kind: event_kind.clone(),
                        text,
                    });
                }
                Err(error) if error.kind() == std::io::ErrorKind::Interrupted => {}
                Err(_) => {
                    thread::sleep(Duration::from_millis(10));
                }
            }
        }
    });

    Ok(())
}

pub fn read_terminal(session_id: &str, terminal_kind: &str) -> Result<String, String> {
    let handle = pty_handle(session_id, terminal_kind)?;
    let mut output = handle
        .output
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    let text = output.clone();
    output.clear();
    Ok(text)
}

pub fn write_terminal(session_id: &str, terminal_kind: &str, input: &str) -> Result<(), String> {
    let handle = pty_handle(session_id, terminal_kind)?;
    let mut writer = handle
        .writer
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    writer
        .write_all(input.as_bytes())
        .and_then(|_| writer.flush())
        .map_err(|error| format!("Terminal input failed: {error}"))
}

pub fn resize_terminal(
    session_id: &str,
    terminal_kind: &str,
    resize: TerminalResize,
) -> Result<(), String> {
    let handle = pty_handle(session_id, terminal_kind)?;
    let result = handle
        .pty
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .resize(PtySize {
            rows: resize.rows.max(1),
            cols: resize.cols.max(1),
            pixel_width: 0,
            pixel_height: 0,
        })
        .map_err(|error| format!("Terminal resize failed: {error}"));
    result
}

pub fn stop_terminal(session_id: &str, terminal_kind: &str) -> Result<(), String> {
    let key = terminal_key(session_id, terminal_kind);
    if let Some(handle) = ptys()
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .remove(&key)
    {
        let _ = handle
            .child
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .kill();
    }
    Ok(())
}

fn pty_handle(session_id: &str, terminal_kind: &str) -> Result<Arc<PtyHandle>, String> {
    ptys()
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .get(&terminal_key(session_id, terminal_kind))
        .cloned()
        .ok_or_else(|| "Terminal session is not running".to_string())
}

fn ptys() -> &'static Mutex<PtyStore> {
    PTYS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn terminal_key(session_id: &str, terminal_kind: &str) -> String {
    format!("{session_id}:{}", normalize_terminal_kind(terminal_kind))
}

fn normalize_terminal_kind(value: &str) -> String {
    if value.trim().eq_ignore_ascii_case("agent") {
        "agent".into()
    } else {
        "user".into()
    }
}

fn shell_program() -> String {
    if Path::new("/bin/zsh").is_file() {
        "/bin/zsh".into()
    } else {
        "/bin/sh".into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;

    #[test]
    fn task_011_pty_executes_shell_commands_and_emits_output() {
        let root = std::env::temp_dir().join("c4os-task-011-pty-output");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("create pty root");
        std::fs::write(root.join("task-011-file.txt"), "terminal").expect("write pty fixture");

        let (sender, receiver) = mpsc::channel();
        start_terminal("task-011-pty-output", "user", &root, move |event| {
            let _ = sender.send(event.text);
        })
        .expect("start pty terminal");
        write_terminal("task-011-pty-output", "user", "ls\r").expect("write ls command");

        let mut output = String::new();
        for _ in 0..20 {
            if let Ok(chunk) = receiver.recv_timeout(Duration::from_millis(100)) {
                output.push_str(&chunk);
                if output.contains("task-011-file.txt") {
                    break;
                }
            }
        }
        stop_terminal("task-011-pty-output", "user").expect("stop pty terminal");

        assert!(
            output.contains("task-011-file.txt"),
            "expected ls output from PTY, got: {output:?}"
        );
    }
}
