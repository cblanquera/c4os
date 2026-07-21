use std::fs;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};

use c4os_lib::execution::terminal::{
    MAX_TERMINAL_DRAIN_BYTES, MAX_TERMINAL_DRAIN_EVENTS, MAX_TERMINAL_RETAINED_SESSIONS,
    TerminalAcknowledgeRequest, TerminalCommandIdentity, TerminalCompletedSessionRecord,
    TerminalDimensions, TerminalDrainRequest, TerminalError, TerminalEvent, TerminalEventKind,
    TerminalExecuteRequest, TerminalLifecycle, TerminalResizeRequest, TerminalRestartRecord,
    TerminalSessionKey, TerminalStdinRequest, TerminalStopRequest, TerminalSupervisor,
    TerminalSupervisorConfiguration,
};
use c4os_lib::execution::{
    ExecutionEnvironmentIdentity, ExecutionEnvironmentKind, TrustedProjectRoot,
};
use tempfile::TempDir;

const TEST_TIMEOUT: Duration = Duration::from_secs(8);

fn shell_path() -> PathBuf {
    ["/bin/zsh", "/bin/bash", "/bin/sh"]
        .into_iter()
        .map(PathBuf::from)
        .find(|path| path.is_file())
        .expect("an absolute system shell")
}

fn local_environment() -> ExecutionEnvironmentIdentity {
    ExecutionEnvironmentIdentity::new(ExecutionEnvironmentKind::Local, "local-test", 1)
        .expect("local environment")
}

fn key(chat_id: &str) -> TerminalSessionKey {
    TerminalSessionKey::new("workspace-terminal-test", chat_id).expect("Terminal key")
}

fn identity(session_id: &str, command_id: &str, sequence: u64) -> TerminalCommandIdentity {
    TerminalCommandIdentity::new(session_id, command_id, sequence).expect("command identity")
}

fn execute_request(
    root: &TrustedProjectRoot,
    key: &TerminalSessionKey,
    session_id: &str,
    command_id: &str,
    command_sequence: u64,
    process_generation: u64,
    command_line: &str,
) -> TerminalExecuteRequest {
    TerminalExecuteRequest {
        key: key.clone(),
        command: identity(session_id, command_id, command_sequence),
        process_generation,
        environment: local_environment(),
        dimensions: TerminalDimensions::new(80, 24).expect("dimensions"),
        trusted_project_root: root.clone(),
        shell_path: shell_path(),
        command_line: command_line.into(),
    }
}

fn is_terminal_event_for(event: &TerminalEvent, command_id: &str) -> bool {
    event.command_id == command_id
        && matches!(
            event.kind,
            TerminalEventKind::Completed { .. }
                | TerminalEventKind::Interrupted130 { .. }
                | TerminalEventKind::RecoveredInterrupted { .. }
                | TerminalEventKind::Failed { .. }
        )
}

fn pump_events(
    supervisor: &mut TerminalSupervisor,
    key: &TerminalSessionKey,
    session_id: &str,
    generation: u64,
    cursor: &mut u64,
    observed: &mut Vec<TerminalEvent>,
) {
    let events = supervisor
        .drain_events(TerminalDrainRequest {
            key: key.clone(),
            terminal_session_id: session_id.into(),
            process_generation: generation,
            after_chunk_sequence: *cursor,
            maximum_events: MAX_TERMINAL_DRAIN_EVENTS,
            maximum_bytes: MAX_TERMINAL_DRAIN_BYTES,
        })
        .expect("drain Terminal events");
    if let Some(sequence) = events.iter().map(|event| event.chunk_sequence).max() {
        *cursor = sequence;
        observed.extend(events);
        supervisor
            .acknowledge_output(TerminalAcknowledgeRequest {
                key: key.clone(),
                terminal_session_id: session_id.into(),
                process_generation: generation,
                through_chunk_sequence: *cursor,
            })
            .expect("acknowledge Terminal events");
    }
}

fn collect_until_terminal(
    supervisor: &mut TerminalSupervisor,
    key: &TerminalSessionKey,
    session_id: &str,
    generation: u64,
    command_id: &str,
) -> Vec<TerminalEvent> {
    let deadline = Instant::now() + TEST_TIMEOUT;
    let mut cursor = 0;
    let mut observed = Vec::new();
    loop {
        pump_events(
            supervisor,
            key,
            session_id,
            generation,
            &mut cursor,
            &mut observed,
        );
        if observed
            .iter()
            .any(|event| is_terminal_event_for(event, command_id))
        {
            return observed;
        }
        assert!(
            Instant::now() < deadline,
            "timed out waiting for command {command_id}; events={observed:?}"
        );
        thread::sleep(Duration::from_millis(10));
    }
}

fn wait_until_started(
    supervisor: &mut TerminalSupervisor,
    key: &TerminalSessionKey,
    session_id: &str,
    generation: u64,
    command_id: &str,
) {
    let deadline = Instant::now() + TEST_TIMEOUT;
    let mut cursor = 0;
    let mut observed = Vec::new();
    loop {
        pump_events(
            supervisor,
            key,
            session_id,
            generation,
            &mut cursor,
            &mut observed,
        );
        if observed.iter().any(|event| {
            event.command_id == command_id
                && matches!(event.kind, TerminalEventKind::Started { .. })
        }) {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "timed out waiting for command {command_id} readiness; events={observed:?}"
        );
        thread::sleep(Duration::from_millis(10));
    }
}

fn output_bytes(events: &[TerminalEvent], command_id: &str) -> Vec<u8> {
    events
        .iter()
        .filter(|event| event.command_id == command_id)
        .filter_map(|event| match &event.kind {
            TerminalEventKind::Output { bytes, .. } => Some(bytes.as_slice()),
            _ => None,
        })
        .flatten()
        .copied()
        .collect()
}

fn wait_for_file_while_pumping(
    supervisor: &mut TerminalSupervisor,
    key: &TerminalSessionKey,
    session_id: &str,
    generation: u64,
    path: &Path,
) {
    let deadline = Instant::now() + TEST_TIMEOUT;
    let mut cursor = 0;
    let mut observed = Vec::new();
    while !path.is_file() {
        pump_events(
            supervisor,
            key,
            session_id,
            generation,
            &mut cursor,
            &mut observed,
        );
        assert!(Instant::now() < deadline, "timed out waiting for {path:?}");
        thread::sleep(Duration::from_millis(10));
    }
}

fn process_exists(process_id: u32) -> bool {
    // SAFETY: signal zero performs existence/permission checking and does not
    // deliver a signal. The production code never uses a persisted PID this way.
    let result = unsafe { libc::kill(process_id as i32, 0) };
    result == 0 || std::io::Error::last_os_error().raw_os_error() == Some(libc::EPERM)
}

fn wait_for_process_exit(process_id: u32) {
    let deadline = Instant::now() + TEST_TIMEOUT;
    while process_exists(process_id) {
        assert!(
            Instant::now() < deadline,
            "process {process_id} survived scoped shutdown"
        );
        thread::sleep(Duration::from_millis(10));
    }
}

#[test]
fn construction_has_no_effect_and_golden_path_reuses_shell_with_exact_exit_and_cwd() {
    let project = TempDir::new().expect("Project");
    fs::create_dir(project.path().join("nested")).expect("nested folder");
    let root = TrustedProjectRoot::open(project.path()).expect("trusted root");
    let key = key("chat-golden");
    let first = execute_request(
        &root,
        &key,
        "terminal-golden",
        "command-one",
        1,
        1,
        "printf 'alpha'; cd nested; printf \"'beta'\"; false",
    );
    let mut supervisor = TerminalSupervisor::new();

    assert_eq!(supervisor.session_count(), 0);
    assert_eq!(supervisor.live_session_count(), 0);
    assert!(!project.path().join("side-effect").exists());

    let first_snapshot = supervisor
        .execute_authorized(first)
        .expect("authorized first command");
    let first_pid = first_snapshot.process_id.expect("live PID");
    let first_events =
        collect_until_terminal(&mut supervisor, &key, "terminal-golden", 1, "command-one");
    assert!(first_events.iter().all(|event| {
        event.key == key
            && event.terminal_session_id == "terminal-golden"
            && event.process_generation == 1
            && event.chunk_sequence > 0
    }));
    assert!(first_events.iter().any(|event| matches!(
        &event.kind,
        TerminalEventKind::Started {
            process_id,
            foreground_process_group_id,
            ..
        } if *process_id == first_pid && *foreground_process_group_id > 0
    )));
    assert_eq!(output_bytes(&first_events, "command-one"), b"alpha'beta'");
    let physical_nested = root.canonical_root().join("nested");
    assert!(first_events.iter().any(|event| matches!(
        &event.kind,
        TerminalEventKind::Completed {
            exit_code: 1,
            working_directory,
        } if working_directory == &physical_nested
    )));

    let second_snapshot = supervisor
        .execute_authorized(execute_request(
            &root,
            &key,
            "terminal-golden",
            "command-two",
            2,
            1,
            "pwd",
        ))
        .expect("same-Chat shell reuse");
    assert_eq!(second_snapshot.process_id, Some(first_pid));
    let second_events =
        collect_until_terminal(&mut supervisor, &key, "terminal-golden", 1, "command-two");
    let second_output = output_bytes(&second_events, "command-two");
    assert!(second_output.starts_with(physical_nested.as_os_str().as_encoded_bytes()));
    assert!(second_events.iter().any(|event| matches!(
        &event.kind,
        TerminalEventKind::Completed {
            exit_code: 0,
            working_directory,
        } if working_directory == &physical_nested
    )));
}

#[test]
fn chats_are_isolated_and_resize_and_raw_stdin_target_only_the_active_command() {
    let project = TempDir::new().expect("Project");
    let root = TrustedProjectRoot::open(project.path()).expect("trusted root");
    let key_a = key("chat-a");
    let key_b = key("chat-b");
    let mut supervisor = TerminalSupervisor::new();
    let snapshot_a = supervisor
        .execute_authorized(execute_request(
            &root,
            &key_a,
            "terminal-a",
            "read-a",
            1,
            1,
            // Legitimate output deliberately equals the submitted secret. The
            // PTY echo must be absent while the command's own bytes remain.
            "read value; printf '%s' \"$value\"",
        ))
        .expect("Chat A");
    let snapshot_b = supervisor
        .execute_authorized(execute_request(
            &root,
            &key_b,
            "terminal-b",
            "read-b",
            1,
            1,
            "read value; printf 'B:%s' \"$value\"",
        ))
        .expect("Chat B");
    assert_eq!(supervisor.live_session_count(), 2);
    assert_ne!(snapshot_a.process_id, snapshot_b.process_id);
    assert_eq!(
        supervisor
            .session_snapshots()
            .into_iter()
            .map(|snapshot| snapshot.key.chat_id)
            .collect::<Vec<_>>(),
        vec!["chat-a", "chat-b"]
    );

    assert!(matches!(
        supervisor.submit_stdin_authorized(TerminalStdinRequest {
            key: key_a.clone(),
            command: identity("terminal-a", "read-a", 1),
            process_generation: 1,
            bytes: b"premature-secret\n".to_vec(),
        }),
        Err(TerminalError::CommandNotReady)
    ));

    assert!(matches!(
        supervisor.execute_authorized(execute_request(
            &root,
            &key_a,
            "terminal-a",
            "overlap",
            2,
            1,
            "true",
        )),
        Err(TerminalError::CommandAlreadyRunning)
    ));
    supervisor
        .resize_authorized(TerminalResizeRequest {
            key: key_a.clone(),
            terminal_session_id: "terminal-a".into(),
            process_generation: 1,
            dimensions: TerminalDimensions::new(132, 43).expect("resized dimensions"),
        })
        .expect("resize Chat A");
    assert_eq!(
        supervisor
            .live_session(&key_a)
            .expect("Chat A snapshot")
            .dimensions,
        TerminalDimensions::new(132, 43).expect("dimensions")
    );
    wait_until_started(&mut supervisor, &key_a, "terminal-a", 1, "read-a");
    wait_until_started(&mut supervisor, &key_b, "terminal-b", 1, "read-b");
    supervisor
        .submit_stdin_authorized(TerminalStdinRequest {
            key: key_a.clone(),
            command: identity("terminal-a", "read-a", 1),
            process_generation: 1,
            bytes: b"o".to_vec(),
        })
        .expect("first secret stdin chunk for Chat A");
    supervisor
        .submit_stdin_authorized(TerminalStdinRequest {
            key: key_a.clone(),
            command: identity("terminal-a", "read-a", 1),
            process_generation: 1,
            bytes: b"ne\n".to_vec(),
        })
        .expect("second secret stdin chunk for Chat A");
    supervisor
        .submit_stdin_authorized(TerminalStdinRequest {
            key: key_b.clone(),
            command: identity("terminal-b", "read-b", 1),
            process_generation: 1,
            bytes: b"two\n".to_vec(),
        })
        .expect("stdin Chat B");
    let events_a = collect_until_terminal(&mut supervisor, &key_a, "terminal-a", 1, "read-a");
    let events_b = collect_until_terminal(&mut supervisor, &key_b, "terminal-b", 1, "read-b");
    assert_eq!(output_bytes(&events_a, "read-a"), b"one");
    assert_eq!(output_bytes(&events_b, "read-b"), b"B:two");
    assert!(
        !output_bytes(&events_a, "read-a")
            .windows(3)
            .any(|window| window == b"two")
    );
    assert!(matches!(
        supervisor.submit_stdin_authorized(TerminalStdinRequest {
            key: key_a,
            command: identity("terminal-a", "read-a", 1),
            process_generation: 2,
            bytes: b"stale\n".to_vec(),
        }),
        Err(TerminalError::StaleGeneration)
    ));
}

#[test]
fn short_partial_output_is_available_before_command_completion() {
    let project = TempDir::new().expect("Project");
    let root = TrustedProjectRoot::open(project.path()).expect("trusted root");
    let terminal_key = key("chat-partial-output");
    let mut supervisor = TerminalSupervisor::new();
    supervisor
        .execute_authorized(execute_request(
            &root,
            &terminal_key,
            "terminal-partial-output",
            "partial-output",
            1,
            1,
            "printf tick; /bin/sleep 3; printf done",
        ))
        .expect("execute streaming command");

    let deadline = Instant::now() + Duration::from_secs(2);
    let mut cursor = 0;
    let mut observed = Vec::new();
    while Instant::now() < deadline {
        pump_events(
            &mut supervisor,
            &terminal_key,
            "terminal-partial-output",
            1,
            &mut cursor,
            &mut observed,
        );
        if output_bytes(&observed, "partial-output") == b"tick" {
            break;
        }
        thread::sleep(Duration::from_millis(10));
    }
    assert_eq!(output_bytes(&observed, "partial-output"), b"tick");
    assert!(
        !observed
            .iter()
            .any(|event| is_terminal_event_for(event, "partial-output"))
    );

    let completed = collect_until_terminal(
        &mut supervisor,
        &terminal_key,
        "terminal-partial-output",
        1,
        "partial-output",
    );
    assert_eq!(output_bytes(&completed, "partial-output"), b"done");
}

#[test]
fn event_and_reader_bounds_drop_old_output_with_explicit_accounting() {
    let project = TempDir::new().expect("Project");
    let root = TrustedProjectRoot::open(project.path()).expect("trusted root");
    let key = key("chat-bounds");
    let configuration = TerminalSupervisorConfiguration {
        reader_channel_capacity: 2,
        reader_frames_per_drain: 64,
        maximum_pending_events: 8,
        maximum_pending_output_bytes: 8 * 1_024,
        ..TerminalSupervisorConfiguration::default()
    };
    let mut supervisor =
        TerminalSupervisor::with_configuration(configuration).expect("bounded supervisor");
    supervisor
        .execute_authorized(execute_request(
            &root,
            &key,
            "terminal-bounds",
            "bounded-output",
            1,
            1,
            "i=0; while [ $i -lt 4096 ]; do printf '0123456789abcdef'; i=$((i+1)); done",
        ))
        .expect("large output command");

    let deadline = Instant::now() + TEST_TIMEOUT;
    let final_events = loop {
        let events = supervisor
            .drain_events(TerminalDrainRequest {
                key: key.clone(),
                terminal_session_id: "terminal-bounds".into(),
                process_generation: 1,
                after_chunk_sequence: 0,
                maximum_events: MAX_TERMINAL_DRAIN_EVENTS,
                maximum_bytes: MAX_TERMINAL_DRAIN_BYTES,
            })
            .expect("bounded drain");
        if events
            .iter()
            .any(|event| is_terminal_event_for(event, "bounded-output"))
        {
            break events;
        }
        assert!(Instant::now() < deadline, "bounded command timed out");
        thread::sleep(Duration::from_millis(10));
    };
    let snapshot = supervisor.live_session(&key).expect("bounded snapshot");
    assert!(snapshot.pending_event_count <= 8);
    assert!(snapshot.pending_output_bytes <= 8 * 1_024);
    assert!(final_events.iter().all(|event| match &event.kind {
        TerminalEventKind::Output { bytes, .. } => bytes.len() <= 8 * 1_024,
        _ => true,
    }));
    let accounted_drop = final_events
        .iter()
        .filter_map(|event| match event.kind {
            TerminalEventKind::Output {
                dropped_bytes_before,
                ..
            } => Some(dropped_bytes_before),
            _ => None,
        })
        .sum::<u64>()
        .saturating_add(snapshot.output_bytes_dropped);
    assert!(accounted_drop > 0, "old output must be explicitly dropped");
}

#[test]
fn stop_interrupts_the_live_foreground_group_and_normalizes_exit_to_130() {
    let project = TempDir::new().expect("Project");
    let root = TrustedProjectRoot::open(project.path()).expect("trusted root");
    let key = key("chat-stop");
    let mut supervisor = TerminalSupervisor::new();
    supervisor
        .execute_authorized(execute_request(
            &root,
            &key,
            "terminal-stop",
            "long-command",
            1,
            1,
            "/bin/sh -c 'echo $$ > descendant.pid; exec /bin/sleep 30'",
        ))
        .expect("long command");
    let descendant_file = project.path().join("descendant.pid");
    wait_for_file_while_pumping(&mut supervisor, &key, "terminal-stop", 1, &descendant_file);
    let descendant_pid = fs::read_to_string(&descendant_file)
        .expect("descendant PID")
        .trim()
        .parse::<u32>()
        .expect("numeric descendant PID");
    let disposition = supervisor
        .stop_authorized(TerminalStopRequest {
            key: key.clone(),
            command: identity("terminal-stop", "long-command", 1),
            process_generation: 1,
        })
        .expect("Stop");
    assert!(disposition.foreground_process_group_id > 0);
    let events = collect_until_terminal(&mut supervisor, &key, "terminal-stop", 1, "long-command");
    assert!(
        events.iter().any(|event| matches!(
            event.kind,
            TerminalEventKind::Interrupted130 {
                shell_replaced: false,
                ..
            }
        )),
        "ordinary Stop must preserve the persistent shell: {events:?}"
    );
    wait_for_process_exit(descendant_pid);

    supervisor
        .execute_authorized(execute_request(
            &root,
            &key,
            "terminal-stop",
            "after-stop",
            2,
            1,
            "printf resumed",
        ))
        .expect("same shell remains reusable after Stop");
    let resumed = collect_until_terminal(&mut supervisor, &key, "terminal-stop", 1, "after-stop");
    assert_eq!(output_bytes(&resumed, "after-stop"), b"resumed");
    assert!(resumed.iter().any(|event| matches!(
        event.kind,
        TerminalEventKind::Completed { exit_code: 0, .. }
    )));
}

#[test]
fn stop_timeout_replaces_stubborn_shell_and_requires_the_next_generation() {
    let project = TempDir::new().expect("Project");
    let root = TrustedProjectRoot::open(project.path()).expect("trusted root");
    let key = key("chat-stop-timeout");
    let configuration = TerminalSupervisorConfiguration {
        stop_timeout: Duration::from_millis(100),
        shutdown_timeout: Duration::from_secs(1),
        ..TerminalSupervisorConfiguration::default()
    };
    let mut supervisor =
        TerminalSupervisor::with_configuration(configuration).expect("short Stop timeout");
    supervisor
        .execute_authorized(execute_request(
            &root,
            &key,
            "terminal-stubborn",
            "stubborn-command",
            1,
            1,
            "/bin/sh -c 'trap \"\" INT; echo $$ > stubborn.pid; exec /bin/sleep 30'",
        ))
        .expect("stubborn command");
    wait_for_file_while_pumping(
        &mut supervisor,
        &key,
        "terminal-stubborn",
        1,
        &project.path().join("stubborn.pid"),
    );
    supervisor
        .stop_authorized(TerminalStopRequest {
            key: key.clone(),
            command: identity("terminal-stubborn", "stubborn-command", 1),
            process_generation: 1,
        })
        .expect("Stop stubborn command");
    let events = collect_until_terminal(
        &mut supervisor,
        &key,
        "terminal-stubborn",
        1,
        "stubborn-command",
    );
    assert!(events.iter().any(|event| matches!(
        event.kind,
        TerminalEventKind::Interrupted130 {
            shell_replaced: true,
            ..
        }
    )));
    assert_eq!(
        supervisor
            .live_session(&key)
            .expect("dormant shell")
            .lifecycle,
        TerminalLifecycle::DormantAfterReplacement
    );
    assert!(matches!(
        supervisor.execute_authorized(execute_request(
            &root,
            &key,
            "terminal-stubborn",
            "stale-command",
            2,
            1,
            "true",
        )),
        Err(TerminalError::StaleGeneration)
    ));
    assert_eq!(
        supervisor
            .live_session(&key)
            .expect("still dormant")
            .lifecycle,
        TerminalLifecycle::DormantAfterReplacement,
        "a stale request must not consume the recovery record"
    );
    supervisor
        .execute_authorized(execute_request(
            &root,
            &key,
            "terminal-stubborn",
            "replacement-command",
            2,
            2,
            "printf recovered",
        ))
        .expect("next-generation replacement shell");
    let replacement = collect_until_terminal(
        &mut supervisor,
        &key,
        "terminal-stubborn",
        2,
        "replacement-command",
    );
    assert_eq!(
        output_bytes(&replacement, "replacement-command"),
        b"recovered"
    );
}

#[test]
fn restart_reconciliation_ignores_persisted_pid_and_retains_under_root_cwd() {
    let project = TempDir::new().expect("Project");
    fs::create_dir(project.path().join("nested")).expect("nested folder");
    let root = TrustedProjectRoot::open(project.path()).expect("trusted root");
    let key = key("chat-recovery");
    let persisted_pid = std::process::id();
    let mut supervisor = TerminalSupervisor::new();
    let recovered = supervisor
        .reconcile_restart(TerminalRestartRecord {
            key: key.clone(),
            command: identity("terminal-recovery", "lost-command", 4),
            process_generation: 7,
            environment: local_environment(),
            dimensions: TerminalDimensions::new(90, 30).expect("dimensions"),
            trusted_project_root: root.clone(),
            shell_path: shell_path(),
            working_directory: project.path().join("nested"),
            persisted_process_id: Some(persisted_pid),
        })
        .expect("restart reconciliation");
    assert_eq!(recovered.lifecycle, TerminalLifecycle::DormantAfterRecovery);
    assert_eq!(recovered.process_id, None);
    assert!(process_exists(persisted_pid));

    let recovered_events = supervisor
        .drain_events(TerminalDrainRequest {
            key: key.clone(),
            terminal_session_id: "terminal-recovery".into(),
            process_generation: 7,
            after_chunk_sequence: 0,
            maximum_events: MAX_TERMINAL_DRAIN_EVENTS,
            maximum_bytes: MAX_TERMINAL_DRAIN_BYTES,
        })
        .expect("recovery events");
    let physical_nested = root.canonical_root().join("nested");
    assert!(recovered_events.iter().any(|event| matches!(
        &event.kind,
        TerminalEventKind::RecoveredInterrupted {
            persisted_process_id_ignored: true,
            working_directory,
        } if working_directory == &physical_nested
    )));
    assert!(matches!(
        supervisor.execute_authorized(execute_request(
            &root,
            &key,
            "terminal-recovery",
            "stale-restart",
            5,
            7,
            "true",
        )),
        Err(TerminalError::StaleGeneration)
    ));
    let live = supervisor
        .execute_authorized(execute_request(
            &root,
            &key,
            "terminal-recovery",
            "resumed-command",
            5,
            8,
            "pwd",
        ))
        .expect("replacement after restart");
    assert_ne!(live.process_id, Some(persisted_pid));
    let events = collect_until_terminal(
        &mut supervisor,
        &key,
        "terminal-recovery",
        8,
        "resumed-command",
    );
    assert!(
        output_bytes(&events, "resumed-command")
            .starts_with(physical_nested.as_os_str().as_encoded_bytes())
    );
}

#[test]
fn completed_session_restore_retains_cwd_without_fabricating_an_interruption() {
    let project = TempDir::new().expect("Project");
    fs::create_dir(project.path().join("nested")).expect("nested folder");
    let root = TrustedProjectRoot::open(project.path()).expect("trusted root");
    let key = key("chat-completed-restore");
    let mut supervisor = TerminalSupervisor::new();
    let restored = supervisor
        .restore_completed_session(TerminalCompletedSessionRecord {
            key: key.clone(),
            command: identity("terminal-completed", "completed-command", 2),
            process_generation: 4,
            environment: local_environment(),
            dimensions: TerminalDimensions::new(90, 30).expect("dimensions"),
            trusted_project_root: root.clone(),
            shell_path: shell_path(),
            working_directory: project.path().join("nested"),
        })
        .expect("completed session restore");
    let physical_nested = root.canonical_root().join("nested");
    assert_eq!(restored.lifecycle, TerminalLifecycle::DormantAfterRestart);
    assert_eq!(restored.process_id, None);
    assert_eq!(restored.process_generation, 4);
    assert_eq!(restored.next_command_sequence, 3);
    assert_eq!(restored.working_directory, physical_nested);
    assert_eq!(restored.pending_event_count, 0);
    assert!(
        supervisor
            .drain_events(TerminalDrainRequest {
                key: key.clone(),
                terminal_session_id: "terminal-completed".into(),
                process_generation: 4,
                after_chunk_sequence: 0,
                maximum_events: MAX_TERMINAL_DRAIN_EVENTS,
                maximum_bytes: MAX_TERMINAL_DRAIN_BYTES,
            })
            .expect("no fabricated recovery event")
            .is_empty()
    );

    supervisor
        .execute_authorized(execute_request(
            &root,
            &key,
            "terminal-completed",
            "resumed-command",
            3,
            5,
            "pwd",
        ))
        .expect("replacement after completed restore");
    let events = collect_until_terminal(
        &mut supervisor,
        &key,
        "terminal-completed",
        5,
        "resumed-command",
    );
    assert!(
        output_bytes(&events, "resumed-command")
            .starts_with(physical_nested.as_os_str().as_encoded_bytes())
    );
}

#[test]
fn replacement_generation_drain_excludes_unacknowledged_recovery_events() {
    let project = TempDir::new().expect("Project");
    let root = TrustedProjectRoot::open(project.path()).expect("trusted root");
    let key = key("chat-generation-filter");
    let mut supervisor = TerminalSupervisor::new();
    supervisor
        .reconcile_restart(TerminalRestartRecord {
            key: key.clone(),
            command: identity("terminal-generation-filter", "lost-command", 1),
            process_generation: 1,
            environment: local_environment(),
            dimensions: TerminalDimensions::new(80, 24).expect("dimensions"),
            trusted_project_root: root.clone(),
            shell_path: shell_path(),
            working_directory: root.canonical_root().to_path_buf(),
            persisted_process_id: Some(std::process::id()),
        })
        .expect("unacknowledged recovery");

    supervisor
        .execute_authorized(execute_request(
            &root,
            &key,
            "terminal-generation-filter",
            "replacement-command",
            2,
            2,
            "printf replacement",
        ))
        .expect("replacement shell");
    let events = collect_until_terminal(
        &mut supervisor,
        &key,
        "terminal-generation-filter",
        2,
        "replacement-command",
    );
    assert!(events.iter().all(|event| event.process_generation == 2));
    assert!(
        events
            .iter()
            .all(|event| !matches!(event.kind, TerminalEventKind::RecoveredInterrupted { .. }))
    );
    assert_eq!(output_bytes(&events, "replacement-command"), b"replacement");
}

#[test]
fn scoped_shutdown_and_drop_terminate_only_owned_sessions() {
    let project = TempDir::new().expect("Project");
    let root = TrustedProjectRoot::open(project.path()).expect("trusted root");
    let key_a = key("chat-shutdown-a");
    let key_b = key("chat-shutdown-b");
    let mut supervisor = TerminalSupervisor::new();
    let pid_a = supervisor
        .execute_authorized(execute_request(
            &root,
            &key_a,
            "terminal-shutdown-a",
            "sleep-a",
            1,
            1,
            "/bin/sleep 30",
        ))
        .expect("session A")
        .process_id
        .expect("PID A");
    let pid_b = supervisor
        .execute_authorized(execute_request(
            &root,
            &key_b,
            "terminal-shutdown-b",
            "sleep-b",
            1,
            1,
            "/bin/sleep 30",
        ))
        .expect("session B")
        .process_id
        .expect("PID B");
    assert!(supervisor.shutdown_session(&key_a).expect("shutdown A"));
    wait_for_process_exit(pid_a);
    assert!(process_exists(pid_b));
    assert!(supervisor.live_session(&key_b).is_some());
    assert_eq!(supervisor.live_session_count(), 1);
    drop(supervisor);
    wait_for_process_exit(pid_b);
}

#[test]
fn retained_session_capacity_is_validated_and_denies_only_new_chat_keys_without_side_effects() {
    let invalid = TerminalSupervisorConfiguration {
        maximum_retained_sessions: 0,
        ..TerminalSupervisorConfiguration::default()
    };
    assert!(matches!(
        TerminalSupervisor::with_configuration(invalid),
        Err(TerminalError::InvalidRequest(_))
    ));
    let excessive = TerminalSupervisorConfiguration {
        maximum_retained_sessions: MAX_TERMINAL_RETAINED_SESSIONS + 1,
        ..TerminalSupervisorConfiguration::default()
    };
    assert!(matches!(
        TerminalSupervisor::with_configuration(excessive),
        Err(TerminalError::InvalidRequest(_))
    ));

    let project = TempDir::new().expect("Project");
    let root = TrustedProjectRoot::open(project.path()).expect("trusted root");
    let retained_key = key("chat-capacity-retained");
    let denied_key = key("chat-capacity-denied");
    let configuration = TerminalSupervisorConfiguration {
        maximum_retained_sessions: 1,
        ..TerminalSupervisorConfiguration::default()
    };
    let mut supervisor = TerminalSupervisor::with_configuration(configuration)
        .expect("one retained Terminal session");

    supervisor
        .reconcile_restart(TerminalRestartRecord {
            key: retained_key.clone(),
            command: identity("terminal-capacity", "recovered-command", 1),
            process_generation: 1,
            environment: local_environment(),
            dimensions: TerminalDimensions::new(80, 24).expect("dimensions"),
            trusted_project_root: root.clone(),
            shell_path: shell_path(),
            working_directory: root.canonical_root().to_path_buf(),
            persisted_process_id: None,
        })
        .expect("one dormant retained session");
    assert_eq!(supervisor.session_count(), 1);
    assert_eq!(supervisor.live_session_count(), 0);

    assert!(matches!(
        supervisor.reconcile_restart(TerminalRestartRecord {
            key: denied_key.clone(),
            command: identity("terminal-denied", "denied-recovery", 1),
            process_generation: 1,
            environment: local_environment(),
            dimensions: TerminalDimensions::new(80, 24).expect("dimensions"),
            trusted_project_root: root.clone(),
            shell_path: shell_path(),
            working_directory: root.canonical_root().to_path_buf(),
            persisted_process_id: None,
        }),
        Err(TerminalError::SessionCapacityExceeded { maximum: 1 })
    ));
    assert!(supervisor.live_session(&denied_key).is_none());

    let denied_side_effect = project.path().join("capacity-side-effect");
    assert!(matches!(
        supervisor.execute_authorized(execute_request(
            &root,
            &denied_key,
            "terminal-denied",
            "denied-execution",
            1,
            1,
            "touch capacity-side-effect",
        )),
        Err(TerminalError::SessionCapacityExceeded { maximum: 1 })
    ));
    assert!(!denied_side_effect.exists());
    assert_eq!(supervisor.session_count(), 1);
    assert_eq!(supervisor.live_session_count(), 0);

    supervisor
        .execute_authorized(execute_request(
            &root,
            &retained_key,
            "terminal-capacity",
            "replacement-command",
            2,
            2,
            "printf replacement",
        ))
        .expect("existing dormant key may replace at capacity");
    let replacement = collect_until_terminal(
        &mut supervisor,
        &retained_key,
        "terminal-capacity",
        2,
        "replacement-command",
    );
    assert_eq!(
        output_bytes(&replacement, "replacement-command"),
        b"replacement"
    );

    supervisor
        .execute_authorized(execute_request(
            &root,
            &retained_key,
            "terminal-capacity",
            "reuse-command",
            3,
            2,
            "printf reuse",
        ))
        .expect("existing live key may reuse at capacity");
    let reuse = collect_until_terminal(
        &mut supervisor,
        &retained_key,
        "terminal-capacity",
        2,
        "reuse-command",
    );
    assert_eq!(output_bytes(&reuse, "reuse-command"), b"reuse");
    assert_eq!(supervisor.session_count(), 1);
    assert_eq!(supervisor.live_session_count(), 1);
    assert!(!denied_side_effect.exists());
}
