# V1 Memory Acceptance

## Proposed Support Tier

`no_durable_memory_v1`

Accepted on 2026-06-12.

## Scope

V1 keeps durable memory out of scope beyond existing session persistence,
session export, and archived-session delete controls.

## Required Behavior

Given a user runs multiple sessions
When the app prepares model context
Then the app does not automatically inject cross-session memory, summaries, or
learned preferences.

Given a session is deleted through the accepted archived-session delete flow
When remaining sessions continue
Then no separate durable memory record survives from the deleted session.

## Out Of Scope

- Cross-session memory.
- Durable user preferences learned from transcript content.
- Automatic summaries for future context.
- Memory write prompts.
- Memory inspect/edit/delete UI.
- Vector stores or embeddings.
- Background indexing.
- Provider-side memory claims.
- Import/export of memory records.

## Failure Conditions

- The app writes durable memory records from transcript content.
- Deleted sessions leave behind separate app-owned memory records.
- Model context includes cross-session memories without explicit user file or
  session reads.
- UI implies memory, embeddings, or learned preferences exist.
