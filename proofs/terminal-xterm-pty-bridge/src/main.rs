use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use serde_json::Value;
use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

struct BridgeState {
    writer: Mutex<Box<dyn Write + Send>>,
    pty: Mutex<Box<dyn portable_pty::MasterPty + Send>>,
    clients: Arc<Mutex<Vec<Sender<String>>>>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = workspace_root();
    let proof_root = proof_root();
    let state = Arc::new(start_pty(&root)?);
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let address = listener.local_addr()?;
    println!("terminal-xterm-pty-bridge listening at http://{address}");
    println!("trusted root: {}", root.display());

    for stream in listener.incoming() {
        let stream = stream?;
        let next_state = Arc::clone(&state);
        let next_proof_root = proof_root.clone();
        thread::spawn(move || {
            if let Err(error) = handle_client(stream, next_state, &next_proof_root) {
                eprintln!("bridge request failed: {error}");
            }
        });
    }

    Ok(())
}

fn start_pty(root: &Path) -> Result<BridgeState, Box<dyn std::error::Error>> {
    let pty_system = native_pty_system();
    let pair = pty_system.openpty(PtySize {
        rows: 31,
        cols: 102,
        pixel_width: 0,
        pixel_height: 0,
    })?;
    let mut command = CommandBuilder::new("/bin/sh");
    command.arg("-lc");
    command.arg("export PS1='cblanquera@Chris-MacBook-2022 c4os % '; cd \"$C4OS_TRUSTED_ROOT\" && /bin/sh -i");
    command.cwd(root);
    command.env("C4OS_TRUSTED_ROOT", root.to_string_lossy().as_ref());
    command.env("TERM", "xterm-256color");
    command.env("PATH", "/usr/bin:/bin:/usr/sbin:/sbin");
    let child = pair.slave.spawn_command(command)?;
    std::mem::forget(child);
    let mut reader = pair.master.try_clone_reader()?;
    let writer = pair.master.take_writer()?;
    let clients = Arc::new(Mutex::new(Vec::new()));
    let state = BridgeState {
        writer: Mutex::new(writer),
        pty: Mutex::new(pair.master),
        clients: Arc::clone(&clients),
    };
    thread::spawn(move || {
        let mut buffer = [0_u8; 4096];
        loop {
            match reader.read(&mut buffer) {
                Ok(0) => break,
                Ok(bytes) => {
                    let text = String::from_utf8_lossy(&buffer[..bytes]).to_string();
                    let mut clients = clients.lock().expect("clients");
                    clients.retain(|client| client.send(text.clone()).is_ok());
                }
                Err(error) if error.kind() == std::io::ErrorKind::Interrupted => {}
                Err(_) => {
                    thread::sleep(Duration::from_millis(10));
                }
            }
        }
    });
    Ok(state)
}

fn handle_client(
    mut stream: TcpStream,
    state: Arc<BridgeState>,
    proof_root: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut request_line = String::new();
    reader.read_line(&mut request_line)?;
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or("");
    let path = parts.next().unwrap_or("/");

    let mut content_length = 0_usize;
    loop {
        let mut line = String::new();
        reader.read_line(&mut line)?;
        let trimmed = line.trim_end();
        if trimmed.is_empty() {
            break;
        }
        if let Some(value) = trimmed.strip_prefix("Content-Length: ") {
            content_length = value.parse().unwrap_or(0);
        }
    }

    let mut body = vec![0_u8; content_length];
    if content_length > 0 {
        reader.read_exact(&mut body)?;
    }

    match (method, path) {
        ("GET", "/") => respond_file(&mut stream, &proof_root.join("index.html"), "text/html"),
        ("GET", "/xterm.js") => respond_file(
            &mut stream,
            &proof_root.join("node_modules/@xterm/xterm/lib/xterm.js"),
            "text/javascript",
        ),
        ("GET", "/xterm.css") => respond_file(
            &mut stream,
            &proof_root.join("node_modules/@xterm/xterm/css/xterm.css"),
            "text/css",
        ),
        ("GET", "/events") => respond_events(stream, state),
        ("POST", "/input") => {
            let mut writer = state.writer.lock().expect("writer");
            writer.write_all(&body)?;
            writer.flush()?;
            respond_text(&mut stream, 204, "No Content", "")
        }
        ("POST", "/resize") => {
            let request: Value = serde_json::from_slice(&body).unwrap_or(Value::Null);
            let rows = request.get("rows").and_then(Value::as_u64).unwrap_or(31) as u16;
            let cols = request.get("cols").and_then(Value::as_u64).unwrap_or(102) as u16;
            let _ = state.pty.lock().expect("pty").resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            });
            respond_text(&mut stream, 204, "No Content", "")
        }
        ("GET", "/health") => respond_text(&mut stream, 200, "OK", "ok"),
        _ => respond_text(&mut stream, 404, "Not Found", "not found"),
    }
}

fn respond_file(
    stream: &mut TcpStream,
    path: &Path,
    content_type: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let body = fs::read(path)?;
    write!(
        stream,
        "HTTP/1.1 200 OK\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    )?;
    stream.write_all(&body)?;
    Ok(())
}

fn respond_text(
    stream: &mut TcpStream,
    status: u16,
    reason: &str,
    body: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    write!(
        stream,
        "HTTP/1.1 {status} {reason}\r\nContent-Type: text/plain\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )?;
    Ok(())
}

fn respond_events(
    mut stream: TcpStream,
    state: Arc<BridgeState>,
) -> Result<(), Box<dyn std::error::Error>> {
    let (sender, receiver) = mpsc::channel::<String>();
    state.clients.lock().expect("clients").push(sender);
    write!(
        stream,
        "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nCache-Control: no-cache\r\nConnection: keep-alive\r\n\r\n"
    )?;
    stream.flush()?;
    for chunk in receiver {
        let encoded = serde_json::to_string(&chunk)?;
        write!(stream, "data: {encoded}\n\n")?;
        stream.flush()?;
    }
    Ok(())
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("workspace root")
        .to_path_buf()
}

fn proof_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}
