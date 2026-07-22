#!/bin/zsh

set -euo pipefail
unsetopt BG_NICE
umask 077

readonly C4OS_TLS_LOCK="/private/tmp/c4os-task-00004-native-tls.lock"
if ! /bin/mkdir "$C4OS_TLS_LOCK" 2>/dev/null; then
  print -u2 "Native TLS verification is already locked at $C4OS_TLS_LOCK."
  print -u2 "Verify that no prior Task 00004 native run is active, then remove only that empty stale lock directory."
  exit 2
fi

typeset C4OS_TLS_TMP=""
typeset -i C4OS_CLEANUP_STARTED=0
typeset -i C4OS_CARGO_CHILD_ACTIVE=0
typeset -i C4OS_CARGO_CHILD_PID=0
typeset -A C4OS_CAPTURED_PROCESS_IDS
typeset -A C4OS_CAPTURED_PROCESS_GROUP_IDS
readonly C4OS_WRAPPER_PROCESS_GROUP_ID="$(/bin/ps -o pgid= -p $$ | /usr/bin/xargs)"

capture_cargo_descendant_tree() {
  if (( ! C4OS_CARGO_CHILD_ACTIVE )); then
    return 0
  fi

  typeset C4OS_PROCESS_LISTING
  C4OS_PROCESS_LISTING="$(/bin/ps -axo pid=,ppid=,pgid=)" || return 1
  typeset -A C4OS_DISCOVERED_PROCESS_IDS
  C4OS_DISCOVERED_PROCESS_IDS[$C4OS_CARGO_CHILD_PID]=1
  typeset -i C4OS_DISCOVERY_CHANGED=1
  typeset C4OS_PROCESS_LINE
  typeset -a C4OS_PROCESS_FIELDS
  typeset -i C4OS_PROCESS_ID
  typeset -i C4OS_PARENT_PROCESS_ID
  typeset -i C4OS_PROCESS_GROUP_ID

  while (( C4OS_DISCOVERY_CHANGED )); do
    C4OS_DISCOVERY_CHANGED=0
    for C4OS_PROCESS_LINE in "${(@f)C4OS_PROCESS_LISTING}"; do
      C4OS_PROCESS_FIELDS=(${=C4OS_PROCESS_LINE})
      (( ${#C4OS_PROCESS_FIELDS[@]} >= 3 )) || continue
      C4OS_PROCESS_ID=$C4OS_PROCESS_FIELDS[1]
      C4OS_PARENT_PROCESS_ID=$C4OS_PROCESS_FIELDS[2]
      if (( ${+C4OS_DISCOVERED_PROCESS_IDS[$C4OS_PARENT_PROCESS_ID]} )) && \
        (( ! ${+C4OS_DISCOVERED_PROCESS_IDS[$C4OS_PROCESS_ID]} )); then
        C4OS_DISCOVERED_PROCESS_IDS[$C4OS_PROCESS_ID]=1
        C4OS_DISCOVERY_CHANGED=1
      fi
    done
  done

  for C4OS_PROCESS_LINE in "${(@f)C4OS_PROCESS_LISTING}"; do
    C4OS_PROCESS_FIELDS=(${=C4OS_PROCESS_LINE})
    (( ${#C4OS_PROCESS_FIELDS[@]} >= 3 )) || continue
    C4OS_PROCESS_ID=$C4OS_PROCESS_FIELDS[1]
    (( ${+C4OS_DISCOVERED_PROCESS_IDS[$C4OS_PROCESS_ID]} )) || continue
    C4OS_PROCESS_GROUP_ID=$C4OS_PROCESS_FIELDS[3]
    C4OS_CAPTURED_PROCESS_IDS[$C4OS_PROCESS_ID]=1
    if (( C4OS_PROCESS_GROUP_ID > 1 )) && \
      [[ "$C4OS_PROCESS_GROUP_ID" != "$C4OS_WRAPPER_PROCESS_GROUP_ID" ]]; then
      C4OS_CAPTURED_PROCESS_GROUP_IDS[$C4OS_PROCESS_GROUP_ID]=1
    fi
  done
}

signal_captured_process_groups() {
  typeset C4OS_SIGNAL="$1"
  typeset C4OS_PROCESS_GROUP_ID
  for C4OS_PROCESS_GROUP_ID in "${(@k)C4OS_CAPTURED_PROCESS_GROUP_IDS}"; do
    /bin/kill "-$C4OS_SIGNAL" -- "-$C4OS_PROCESS_GROUP_ID" 2>/dev/null || true
  done
}

captured_process_tree_is_absent() {
  typeset C4OS_PROCESS_ID
  typeset C4OS_PROCESS_GROUP_ID
  for C4OS_PROCESS_ID in "${(@k)C4OS_CAPTURED_PROCESS_IDS}"; do
    if /bin/kill -0 "$C4OS_PROCESS_ID" 2>/dev/null; then
      return 1
    fi
  done
  for C4OS_PROCESS_GROUP_ID in "${(@k)C4OS_CAPTURED_PROCESS_GROUP_IDS}"; do
    if /bin/kill -0 -- "-$C4OS_PROCESS_GROUP_ID" 2>/dev/null; then
      return 1
    fi
  done
  return 0
}

terminate_cargo_process_tree() {
  if (( ! C4OS_CARGO_CHILD_ACTIVE )); then
    return 0
  fi

  typeset -i C4OS_TERMINATION_STATUS=0
  capture_cargo_descendant_tree || C4OS_TERMINATION_STATUS=1
  signal_captured_process_groups TERM
  /bin/kill -TERM "$C4OS_CARGO_CHILD_PID" 2>/dev/null || true

  typeset -i C4OS_WAIT_ATTEMPT
  for C4OS_WAIT_ATTEMPT in {1..20}; do
    /bin/sleep 0.1
    capture_cargo_descendant_tree || C4OS_TERMINATION_STATUS=1
    signal_captured_process_groups TERM
  done

  signal_captured_process_groups KILL
  /bin/kill -KILL "$C4OS_CARGO_CHILD_PID" 2>/dev/null || true
  if wait "$C4OS_CARGO_CHILD_PID" 2>/dev/null; then
    :
  fi
  C4OS_CARGO_CHILD_ACTIVE=0
  for C4OS_WAIT_ATTEMPT in {1..30}; do
    if captured_process_tree_is_absent; then
      return "$C4OS_TERMINATION_STATUS"
    fi
    /bin/sleep 0.1
  done
  return 1
}

cleanup() {
  typeset -i C4OS_EXIT_STATUS=$?
  if (( C4OS_CLEANUP_STARTED )); then
    exit "$C4OS_EXIT_STATUS"
  fi
  C4OS_CLEANUP_STARTED=1
  trap - EXIT
  trap '' INT TERM HUP
  typeset -i C4OS_CLEANUP_STATUS=0
  set +e
  terminate_cargo_process_tree || C4OS_CLEANUP_STATUS=1
  if [[ -n "$C4OS_TLS_TMP" && -f "$C4OS_TLS_TMP/leaf.pem" ]] && \
    /usr/bin/security verify-cert -q -c "$C4OS_TLS_TMP/leaf.pem" -p ssl -n 127.0.0.1 -L >/dev/null 2>&1; then
    C4OS_CLEANUP_STATUS=1
  fi
  if [[ -z "$C4OS_TLS_TMP" ]]; then
    :
  elif [[ "$C4OS_TLS_TMP" == /private/tmp/c4os-task-00004-tls.* && -d "$C4OS_TLS_TMP" ]]; then
    /bin/rm -rf -- "$C4OS_TLS_TMP" || C4OS_CLEANUP_STATUS=1
  else
    C4OS_CLEANUP_STATUS=1
  fi
  /bin/rmdir "$C4OS_TLS_LOCK" >/dev/null 2>&1 || C4OS_CLEANUP_STATUS=1
  if (( C4OS_CLEANUP_STATUS != 0 )); then
    print -u2 "Task 00004 native TLS cleanup did not restore every process/temp/trust invariant."
    if (( C4OS_EXIT_STATUS == 0 )); then
      C4OS_EXIT_STATUS=1
    fi
  fi
  exit "$C4OS_EXIT_STATUS"
}

handle_signal() {
  typeset -i C4OS_SIGNAL_NUMBER="$1"
  trap '' INT TERM HUP
  exit "$((128 + C4OS_SIGNAL_NUMBER))"
}

trap cleanup EXIT
trap 'handle_signal 2' INT
trap 'handle_signal 15' TERM
trap 'handle_signal 1' HUP

C4OS_TLS_TMP="$(/usr/bin/mktemp -d /private/tmp/c4os-task-00004-tls.XXXXXX)"
readonly C4OS_TLS_TMP
readonly C4OS_TLS_CA="$C4OS_TLS_TMP/ca.pem"
readonly C4OS_TLS_CA_KEY="$C4OS_TLS_TMP/ca-key.pem"
readonly C4OS_TLS_CA_SERIAL="$C4OS_TLS_TMP/ca.srl"
readonly C4OS_TLS_LEAF="$C4OS_TLS_TMP/leaf.pem"
readonly C4OS_TLS_LEAF_KEY="$C4OS_TLS_TMP/leaf-key.pem"
readonly C4OS_TLS_CSR="$C4OS_TLS_TMP/leaf.csr"
readonly C4OS_TLS_EXT="$C4OS_TLS_TMP/leaf.ext"
readonly C4OS_PROVIDER_CREDENTIAL_FILE="$C4OS_TLS_TMP/provider-credential"
readonly C4OS_PROJECT_ROOT="${0:A:h:h}"
readonly C4OS_PROVIDER_EVIDENCE="$C4OS_PROJECT_ROOT/output/native/task-00004-provider-evidence.json"
readonly C4OS_RESOURCE_ROOT="${1:-$C4OS_PROJECT_ROOT/target/debug/bundle/macos/C4OS.app/Contents/Resources}"
readonly C4OS_CARGO_EXECUTABLE="${commands[cargo]:-}"

if [[ ! -d "$C4OS_RESOURCE_ROOT" ]]; then
  print -u2 "Built C4OS.app resources were not found at $C4OS_RESOURCE_ROOT"
  exit 2
fi
if [[ -z "$C4OS_CARGO_EXECUTABLE" || ! -x "$C4OS_CARGO_EXECUTABLE" ]]; then
  print -u2 "A fixed executable Cargo path is required for native verification."
  exit 2
fi
/bin/mkdir -p "$C4OS_PROJECT_ROOT/output/native"
/bin/rm -f -- "$C4OS_PROVIDER_EVIDENCE" "$C4OS_PROVIDER_EVIDENCE.tmp"

/usr/bin/openssl req -x509 -newkey rsa:2048 -nodes \
  -keyout "$C4OS_TLS_CA_KEY" \
  -out "$C4OS_TLS_CA" \
  -days 1 \
  -subj "/CN=C4OS Task 00004 Ephemeral Root" \
  -addext "basicConstraints=critical,CA:TRUE,pathlen:0" \
  -addext "keyUsage=critical,keyCertSign,cRLSign" >/dev/null 2>&1

/usr/bin/openssl req -newkey rsa:2048 -nodes \
  -keyout "$C4OS_TLS_LEAF_KEY" \
  -out "$C4OS_TLS_CSR" \
  -subj "/CN=localhost" >/dev/null 2>&1

/usr/bin/printf '%s\n' \
  'basicConstraints=critical,CA:FALSE' \
  'keyUsage=critical,digitalSignature,keyEncipherment' \
  'extendedKeyUsage=serverAuth' \
  'subjectAltName=IP:127.0.0.1,DNS:localhost' >"$C4OS_TLS_EXT"

/usr/bin/openssl x509 -req \
  -in "$C4OS_TLS_CSR" \
  -CA "$C4OS_TLS_CA" \
  -CAkey "$C4OS_TLS_CA_KEY" \
  -CAserial "$C4OS_TLS_CA_SERIAL" \
  -CAcreateserial \
  -out "$C4OS_TLS_LEAF" \
  -days 1 \
  -extfile "$C4OS_TLS_EXT" >/dev/null 2>&1

/usr/bin/openssl verify -CAfile "$C4OS_TLS_CA" "$C4OS_TLS_LEAF" >/dev/null
if /usr/bin/security verify-cert -q -c "$C4OS_TLS_LEAF" -p ssl -n 127.0.0.1 -L >/dev/null 2>&1; then
  print -u2 "Ephemeral leaf unexpectedly verified through machine trust before the private capability run."
  exit 2
fi
/usr/bin/openssl rand -hex 32 >"$C4OS_PROVIDER_CREDENTIAL_FILE"
/bin/chmod 600 "$C4OS_PROVIDER_CREDENTIAL_FILE"

cd "$C4OS_PROJECT_ROOT"
C4OS_BUNDLED_RESOURCE_ROOT="$C4OS_RESOURCE_ROOT" \
C4OS_NATIVE_TLS_CA="$C4OS_TLS_CA" \
C4OS_NATIVE_TLS_CERT="$C4OS_TLS_LEAF" \
C4OS_NATIVE_TLS_KEY="$C4OS_TLS_LEAF_KEY" \
C4OS_NATIVE_PROVIDER_CREDENTIAL_FILE="$C4OS_PROVIDER_CREDENTIAL_FILE" \
C4OS_NATIVE_PROVIDER_EVIDENCE="$C4OS_PROVIDER_EVIDENCE" \
  /usr/bin/python3 -c \
    'import os, sys; os.getpgrp() == os.getpid() or os.setsid(); os.execv(sys.argv[1], sys.argv[1:])' \
    "$C4OS_CARGO_EXECUTABLE" test --test runtime_production \
    packaged_production_peers_complete_the_app_owned_native_golden_paths \
    -- --ignored --exact --test-threads=1 &
C4OS_CARGO_CHILD_PID=$!
C4OS_CARGO_CHILD_ACTIVE=1
typeset -i C4OS_GROUP_WAIT_ATTEMPT
typeset C4OS_CARGO_PROCESS_GROUP_ID=""
for C4OS_GROUP_WAIT_ATTEMPT in {1..20}; do
  C4OS_CARGO_PROCESS_GROUP_ID="$(/bin/ps -o pgid= -p "$C4OS_CARGO_CHILD_PID" 2>/dev/null | /usr/bin/xargs)"
  if [[ "$C4OS_CARGO_PROCESS_GROUP_ID" == "$C4OS_CARGO_CHILD_PID" ]]; then
    break
  fi
  /bin/sleep 0.05
done
if [[ "$C4OS_CARGO_PROCESS_GROUP_ID" != "$C4OS_CARGO_CHILD_PID" ]]; then
  print -u2 "Cargo native verification did not enter its isolated process group."
  exit 2
fi
C4OS_CAPTURED_PROCESS_IDS[$C4OS_CARGO_CHILD_PID]=1
C4OS_CAPTURED_PROCESS_GROUP_IDS[$C4OS_CARGO_PROCESS_GROUP_ID]=1
typeset -i C4OS_CARGO_STATUS=0
if wait "$C4OS_CARGO_CHILD_PID"; then
  C4OS_CARGO_STATUS=0
else
  C4OS_CARGO_STATUS=$?
fi
C4OS_CARGO_CHILD_ACTIVE=0
exit "$C4OS_CARGO_STATUS"
