import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { TerminalArtifact } from "./TerminalArtifact";
import type { TerminalArtifactModel } from "./terminal-types";

vi.mock("./TerminalViewport", () => ({
  TerminalViewport: ({
    accessibleLabel,
    onOutputWritten,
    onResize,
    onSelectionChange,
    outputSequence,
  }: {
    readonly accessibleLabel: string;
    readonly onOutputWritten?: (outputSequence: number) => void;
    readonly onResize?: (columns: number, rows: number) => void;
    readonly onSelectionChange?: (selection: string) => void;
    readonly outputSequence: number;
  }) => (
    <div aria-label={accessibleLabel} data-testid="terminal-viewport">
      <button
        onClick={() => onSelectionChange?.("selected output")}
        type="button"
      >
        Select output
      </button>
      <button onClick={() => onResize?.(100, 36)} type="button">
        Resize terminal
      </button>
      <button onClick={() => onOutputWritten?.(outputSequence)} type="button">
        Commit terminal output
      </button>
    </div>
  ),
}));

const RUNNING_MODEL: TerminalArtifactModel = {
  artifactId: "artifact:terminal-1",
  pendingApprovalId: null,
  terminalSessionId: "terminal:session-1",
  commandId: "command:1",
  commandSequence: 1,
  command: "npm test",
  workingDirectoryDisplay: "/project",
  shellPath: "/bin/zsh",
  environmentId: "local",
  environmentGeneration: 2,
  processGeneration: 3,
  shellProcessId: 412,
  foregroundProcessGroupId: 413,
  columns: 80,
  rows: 24,
  outputBase64: encode(
    "\u001b[31mfirst line\u001b[0m\nselected output\nraw xterm only\n",
  ),
  outputText: "first line\nselected output\n",
  outputSequence: 2,
  retainedBytes: 27,
  droppedBytes: 0,
  phase: "running",
  exitCode: null,
  statusMessage: null,
  stdinReady: false,
  stopAvailable: true,
  promptReady: false,
  shellReplaced: false,
};

const requiredProps = () => ({
  onPromptValueChange: vi.fn(),
  onStdinValueChange: vi.fn(),
  promptValue: "",
  reducedMotion: false,
  stdinValue: "",
});

describe("TerminalArtifact", () => {
  it("renders an immutable inline command card without repeating its command", () => {
    const onCopy = vi.fn();
    const onExpand = vi.fn();
    const onStop = vi.fn();
    render(
      <TerminalArtifact
        {...requiredProps()}
        context="inline"
        model={RUNNING_MODEL}
        onCopy={onCopy}
        onExpand={onExpand}
        onStop={onStop}
      />,
    );

    const artifact = screen.getByRole("article", {
      name: "Terminal response artifact: npm test",
    });
    expect(artifact).toBeVisible();
    expect(screen.getByLabelText("Output for npm test").textContent).toBe(
      "first line\nselected output\n",
    );
    expect(screen.queryByText("raw xterm only")).not.toBeInTheDocument();
    expect(screen.queryByTestId("terminal-viewport")).not.toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "Copy" }));
    fireEvent.click(screen.getByRole("button", { name: "Expand" }));
    fireEvent.click(screen.getByRole("button", { name: "Stop npm test" }));
    expect(onCopy).toHaveBeenCalledWith(
      "artifact:terminal-1",
      "npm test\nfirst line\nselected output\n",
    );
    expect(onExpand).toHaveBeenCalledWith("artifact:terminal-1");
    expect(onStop).toHaveBeenCalledWith("artifact:terminal-1", "command:1");
  });

  it("keeps contextual cards read-only and scoped Reply selection local", () => {
    const onReply = vi.fn();
    const { rerender } = render(
      <>
        <p data-testid="outside">selected output</p>
        <TerminalArtifact
          {...requiredProps()}
          context="inline"
          model={RUNNING_MODEL}
          onReply={onReply}
          onStop={vi.fn()}
        />
      </>,
    );
    const output = screen.getByLabelText("Output for npm test");
    const outputRange = document.createRange();
    outputRange.selectNodeContents(output);
    const selection = window.getSelection();
    selection?.removeAllRanges();
    selection?.addRange(outputRange);
    fireEvent.click(screen.getByRole("button", { name: "Reply" }));
    expect(onReply).toHaveBeenLastCalledWith("artifact:terminal-1", {
      selectedText: "first line\nselected output\n",
    });

    const outsideRange = document.createRange();
    outsideRange.selectNodeContents(screen.getByTestId("outside"));
    selection?.removeAllRanges();
    selection?.addRange(outsideRange);
    fireEvent.click(screen.getByRole("button", { name: "Reply" }));
    expect(onReply).toHaveBeenLastCalledWith("artifact:terminal-1");

    rerender(
      <TerminalArtifact
        {...requiredProps()}
        context="contextual"
        model={RUNNING_MODEL}
        onReply={onReply}
        onStop={vi.fn()}
      />,
    );
    expect(
      screen.queryByRole("button", { name: "Stop npm test" }),
    ).not.toBeInTheDocument();
    expect(screen.queryByRole("form")).not.toBeInTheDocument();
    selection?.removeAllRanges();
  });

  it("uses focused xterm selection and emits only controlled resize intent", () => {
    const onReply = vi.fn();
    const onResize = vi.fn();
    const onOutputWritten = vi.fn();
    render(
      <TerminalArtifact
        {...requiredProps()}
        context="focused"
        model={RUNNING_MODEL}
        onOutputWritten={onOutputWritten}
        onReply={onReply}
        onResize={onResize}
      />,
    );

    expect(screen.getByTestId("terminal-viewport")).toBeVisible();
    expect(screen.queryByText("first line")).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Select output" }));
    fireEvent.click(screen.getByRole("button", { name: "Reply" }));
    expect(onReply).toHaveBeenCalledWith("artifact:terminal-1", {
      selectedText: "selected output",
    });

    fireEvent.click(screen.getByRole("button", { name: "Resize terminal" }));
    expect(onResize).toHaveBeenCalledWith({
      artifactId: "artifact:terminal-1",
      terminalSessionId: "terminal:session-1",
      columns: 100,
      rows: 36,
    });
    fireEvent.click(
      screen.getByRole("button", { name: "Commit terminal output" }),
    );
    expect(onOutputWritten).toHaveBeenCalledWith(2);
  });

  it("separates ordinary stdin from the reusable next-command prompt", () => {
    const onStdinValueChange = vi.fn();
    const onSubmitStdin = vi.fn();
    const onPromptValueChange = vi.fn();
    const onRunNext = vi.fn();
    const { rerender } = render(
      <TerminalArtifact
        context="focused"
        model={{
          ...RUNNING_MODEL,
          phase: "stdinReady",
          stdinReady: true,
        }}
        onPromptValueChange={onPromptValueChange}
        onRunNext={onRunNext}
        onStdinValueChange={onStdinValueChange}
        onSubmitStdin={onSubmitStdin}
        promptValue=""
        reducedMotion
        stdinValue="yes"
      />,
    );

    const stdin = screen.getByRole("textbox", { name: "Process input" });
    fireEvent.change(stdin, { target: { value: "no" } });
    fireEvent.submit(screen.getByRole("form", { name: "Input for npm test" }));
    expect(onStdinValueChange).toHaveBeenCalledWith("no");
    expect(onSubmitStdin).toHaveBeenCalledWith({
      artifactId: "artifact:terminal-1",
      commandId: "command:1",
      input: "yes",
    });
    expect(onRunNext).not.toHaveBeenCalled();

    rerender(
      <TerminalArtifact
        context="focused"
        model={{
          ...RUNNING_MODEL,
          phase: "completed",
          exitCode: 0,
          promptReady: true,
          stdinReady: false,
          stopAvailable: false,
        }}
        onPromptValueChange={onPromptValueChange}
        onRunNext={onRunNext}
        onStdinValueChange={onStdinValueChange}
        onSubmitStdin={onSubmitStdin}
        promptValue="pwd"
        reducedMotion
        stdinValue=""
      />,
    );

    expect(screen.getAllByText("Completed · exit 0")).toHaveLength(2);
    const prompt = screen.getByRole("textbox", { name: "Next command" });
    fireEvent.change(prompt, { target: { value: "ls" } });
    fireEvent.submit(screen.getByRole("form", { name: "Terminal prompt" }));
    expect(onPromptValueChange).toHaveBeenCalledWith("ls");
    expect(onRunNext).toHaveBeenCalledWith({
      artifactId: "artifact:terminal-1",
      terminalSessionId: "terminal:session-1",
      command: "pwd",
    });
  });

  it("presents interruption, truncation, shell replacement, and failure text", () => {
    const { rerender } = render(
      <TerminalArtifact
        {...requiredProps()}
        context="inline"
        model={{
          ...RUNNING_MODEL,
          droppedBytes: 4_096,
          exitCode: 130,
          phase: "interrupted",
          shellReplaced: true,
          stopAvailable: false,
        }}
      />,
    );
    expect(
      screen.getAllByText("Interrupted · exit 130 · shell replaced"),
    ).toHaveLength(2);
    expect(screen.getByText(/4,096 bytes dropped/)).toBeVisible();

    rerender(
      <TerminalArtifact
        {...requiredProps()}
        context="inline"
        model={{
          ...RUNNING_MODEL,
          exitCode: 2,
          phase: "failed",
          statusMessage: "The process could not start.",
          stopAvailable: false,
        }}
      />,
    );
    expect(screen.getByRole("alert")).toHaveTextContent(
      "The process could not start.",
    );
    expect(screen.getAllByText("Failed · exit 2")).toHaveLength(2);
  });

  it("keeps stopping non-interactive even when stale readiness flags remain true", () => {
    render(
      <TerminalArtifact
        context="focused"
        model={{
          ...RUNNING_MODEL,
          phase: "stopping",
          promptReady: true,
          stdinReady: true,
          stopAvailable: true,
        }}
        onPromptValueChange={vi.fn()}
        onRunNext={vi.fn()}
        onStdinValueChange={vi.fn()}
        onStop={vi.fn()}
        onSubmitStdin={vi.fn()}
        promptValue="pwd"
        reducedMotion={false}
        stdinValue="yes"
      />,
    );

    expect(screen.getAllByText("Stopping…")).toHaveLength(2);
    expect(
      screen.queryByRole("button", { name: "Stop npm test" }),
    ).not.toBeInTheDocument();
    expect(
      screen.queryByRole("form", { name: "Input for npm test" }),
    ).not.toBeInTheDocument();
    expect(
      screen.queryByRole("form", { name: "Terminal prompt" }),
    ).not.toBeInTheDocument();
  });

  it("binds approval controls to the exact pending approval", () => {
    const onAllowApproval = vi.fn();
    const onDenyApproval = vi.fn();
    const { rerender } = render(
      <TerminalArtifact
        context="focused"
        model={{
          ...RUNNING_MODEL,
          pendingApprovalId: "approval:terminal-1",
          phase: "approvalWaiting",
          promptReady: true,
          stdinReady: true,
          stopAvailable: true,
        }}
        onAllowApproval={onAllowApproval}
        onDenyApproval={onDenyApproval}
        onPromptValueChange={vi.fn()}
        onRunNext={vi.fn()}
        onStdinValueChange={vi.fn()}
        onStop={vi.fn()}
        onSubmitStdin={vi.fn()}
        promptValue="pwd"
        reducedMotion={false}
        stdinValue="yes"
      />,
    );

    expect(
      screen.getByRole("group", { name: "Approval for npm test" }),
    ).toBeVisible();
    fireEvent.click(screen.getByRole("button", { name: "Deny" }));
    fireEvent.click(screen.getByRole("button", { name: "Allow" }));
    expect(onDenyApproval).toHaveBeenCalledWith(
      "artifact:terminal-1",
      "approval:terminal-1",
    );
    expect(onAllowApproval).toHaveBeenCalledWith(
      "artifact:terminal-1",
      "approval:terminal-1",
    );
    expect(
      screen.queryByRole("button", { name: "Stop npm test" }),
    ).not.toBeInTheDocument();
    expect(
      screen.queryByRole("form", { name: "Input for npm test" }),
    ).not.toBeInTheDocument();
    expect(
      screen.queryByRole("form", { name: "Terminal prompt" }),
    ).not.toBeInTheDocument();

    rerender(
      <TerminalArtifact
        {...requiredProps()}
        context="focused"
        model={{
          ...RUNNING_MODEL,
          pendingApprovalId: null,
          phase: "approvalWaiting",
        }}
        onAllowApproval={onAllowApproval}
        onDenyApproval={onDenyApproval}
      />,
    );
    expect(
      screen.queryByRole("group", { name: "Approval for npm test" }),
    ).not.toBeInTheDocument();

    rerender(
      <TerminalArtifact
        {...requiredProps()}
        context="focused"
        model={{
          ...RUNNING_MODEL,
          pendingApprovalId: "approval:stale",
          phase: "running",
        }}
        onAllowApproval={onAllowApproval}
        onDenyApproval={onDenyApproval}
      />,
    );
    expect(
      screen.getByRole("group", { name: "Approval for npm test" }),
    ).toBeVisible();
  });

  it("gives a pending stdin approval precedence over all live controls", () => {
    render(
      <TerminalArtifact
        context="focused"
        model={{
          ...RUNNING_MODEL,
          pendingApprovalId: "approval:stdin-1",
          phase: "stdinReady",
          promptReady: true,
          stdinReady: true,
          stopAvailable: true,
        }}
        onAllowApproval={vi.fn()}
        onDenyApproval={vi.fn()}
        onPromptValueChange={vi.fn()}
        onRunNext={vi.fn()}
        onStdinValueChange={vi.fn()}
        onStop={vi.fn()}
        onSubmitStdin={vi.fn()}
        promptValue="pwd"
        reducedMotion={false}
        stdinValue="yes"
      />,
    );

    expect(screen.getAllByText("Waiting for input")).toHaveLength(2);
    expect(
      screen.getByRole("group", { name: "Approval for npm test" }),
    ).toBeVisible();
    expect(
      screen.queryByRole("button", { name: "Stop npm test" }),
    ).not.toBeInTheDocument();
    expect(
      screen.queryByRole("form", { name: "Input for npm test" }),
    ).not.toBeInTheDocument();
    expect(
      screen.queryByRole("form", { name: "Terminal prompt" }),
    ).not.toBeInTheDocument();
  });
});

function encode(value: string): string {
  const bytes = new TextEncoder().encode(value);
  let binary = "";
  for (const byte of bytes) binary += String.fromCharCode(byte);
  return window.btoa(binary);
}
