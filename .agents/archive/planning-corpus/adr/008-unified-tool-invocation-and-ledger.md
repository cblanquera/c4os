# ADR-008: Unified Tool Invocation And Ledger

Status: Provisional.

## Context

The research recommends modeling every runtime capability as a tool invocation with a normalized policy envelope, regardless of whether it originates from OpenCode, MCP, plugin code, or the GUI. The architecture includes a Tool Ledger as an append-only record of tool calls, arguments, approvals, outputs, and artifacts.

The review says per-tool approval is necessary but insufficient because safe individual calls can compose into unsafe data flows.

## Decision

Represent executable capabilities through a unified tool-call model and persist them in a tool ledger.

This is provisional because the enforcement point, ledger integrity level, and data-flow tracking are unresolved.

## Alternatives Considered

 - Unified tool envelope.
 - Special-case shell/filesystem calls.
 - Separate MCP and local tool models.

## Alternatives That Should Be Considered

 - Mandatory tool proxy/gateway.
 - Event-sourced tool ledger.
 - Information-flow-aware ledger.
 - Ledger signing or hash chaining if audit integrity is required.

## Tradeoffs

A unified tool model supports approvals, audit, replay, policy simulation, and consistent UX.

Normalizing tools can hide important distinctions between reading a file, writing a file, launching a process, making a network request, or using credentials.

Per-tool logs do not automatically prevent harmful tool chains.

## Consequences

 - Tool schema becomes a core platform contract.
 - Every runtime integration must prove that tool calls cannot bypass the ledger.
 - Redaction and output truncation must be part of the tool model.

## Follow-Up Questions

 - Can all OpenCode, MCP, browser, Git, shell, plugin, and artifact actions be forced through the ledger?
 - Does the policy engine track data movement between tools?
 - Is the ledger tamper-evident?
 - How are secrets redacted from tool inputs and outputs?

## ADR Recommendation

Keep this ADR high priority and resolve it with ADR-004.

