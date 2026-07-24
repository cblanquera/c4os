import { fireEvent, render, screen, within } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import {
  ConversationTranscript,
  PendingConversationPrompt,
} from "../../../../../src/frontend/features/conversation/ui/ConversationTranscript";
import { SafeMarkdown } from "../../../../../src/frontend/features/conversation/ui/SafeMarkdown";
import type {
  AssistantTranscriptTurn,
  ConversationTranscriptTurn,
} from "../../../../../src/frontend/features/conversation/ui/types";

const ASSISTANT_TURN: AssistantTranscriptTurn = {
  id: "turn-assistant",
  author: "assistant",
  markdownSource: "Done with **two files**. https://example.com/result",
  status: "completed",
  announceCompletion: true,
  modelLabel: "GPT-5",
  work: {
    kind: "activity",
    summary: "Checked the workspace",
    durationLabel: "Worked for 4s",
    isExpanded: true,
    progress: ["Inspected the selected files."],
    details: [
      {
        id: "activity-read",
        label: "Read project files",
        detail: "2 files",
        state: "completed",
      },
    ],
  },
  provenance: {
    adapter: "OCAdapter 1.4",
    capabilitySummary: "Text and tools",
    environment: "Local",
    isExpanded: true,
    runtime: "OpenCode 1.18.3",
  },
  artifact: {
    id: "artifact-file",
    type: "file",
    title: "result.md",
    summary: "File response",
    focusSupported: true,
    isFocused: false,
  },
};

const TURNS: readonly ConversationTranscriptTurn[] = [
  {
    id: "turn-user",
    author: "user",
    markdownSource: "Please inspect `<unsafe>`.",
    status: "completed",
  },
  ASSISTANT_TURN,
];

function createProps() {
  return {
    onArtifactFocusRequest: vi.fn(),
    onCopy: vi.fn(),
    onLinkActivate: vi.fn(),
    onProvenanceExpandedChange: vi.fn(),
    onReply: vi.fn(),
    onWorkExpandedChange: vi.fn(),
    reducedMotion: false,
    turns: TURNS,
  };
}

describe("ConversationTranscript", () => {
  it("renders immutable user and C4OS surfaces in accepted assistant order", () => {
    const props = createProps();
    const { container } = render(<ConversationTranscript {...props} />);

    expect(screen.getByRole("feed").children).toHaveLength(2);
    expect(screen.getByLabelText("User message")).not.toHaveTextContent("You");

    const assistant = screen.getByLabelText("C4OS response");
    const directSections = Array.from(assistant.children);
    expect(directSections[0]).toHaveClass("conversation-work");
    expect(assistant).toHaveTextContent("ActivityWorked for 4s");
    expect(assistant).toHaveTextContent("Inspected the selected files.");
    expect(assistant).toHaveTextContent("C4OSGPT-5");
    expect(assistant).toHaveTextContent("OpenCode 1.18.3");
    expect(assistant).toHaveTextContent("Text and tools");
    expect(assistant).not.toHaveTextContent("Work summary");
    expect(container.querySelector("script")).toBeNull();
  });

  it("keeps Copy and Reply keyboard reachable and preserves source Markdown", () => {
    const props = createProps();
    render(<ConversationTranscript {...props} />);

    const user = screen.getByLabelText("User message");
    const actions = within(user).getByLabelText("Message actions");
    const copy = within(actions).getByRole("button", { name: "Copy" });
    copy.focus();
    expect(copy).toHaveFocus();
    fireEvent.click(copy);
    expect(props.onCopy).toHaveBeenCalledWith(
      "turn-user",
      "Please inspect `<unsafe>`.",
    );

    fireEvent.click(within(actions).getByRole("button", { name: "Reply" }));
    expect(props.onReply).toHaveBeenCalledWith(
      "turn-user",
      "Please inspect `<unsafe>`.",
    );
  });

  it("discloses the supplied and truncated immutable Artifact Reply context", () => {
    const props = createProps();
    render(
      <ConversationTranscript
        {...props}
        turns={[
          {
            id: "turn-reply",
            author: "user",
            markdownSource: "Update this file.",
            status: "completed",
            replyContext: {
              artifactId: "artifact-1",
              providerType: "file",
              providerVersion: 1,
              recordRevision: 3,
              stableReference:
                "artifact:artifact-1:record:3:resource:2:sha256:abc",
              suppliedBytes: 12_000,
              maximumBytes: 16_384,
              omittedBytes: 400,
              truncated: true,
              unsaved: true,
              segments: [
                {
                  source: "unsaved-draft",
                  text: "retained draft",
                  omittedBytes: 400,
                },
              ],
              capabilities: [
                {
                  capabilityId: "artifact.write",
                  access: "approvalRequired",
                  reasonCode: "live-version-revalidation",
                },
              ],
            },
          },
        ]}
      />,
    );

    const disclosure = screen.getByText(/Reply to file/).closest("details");
    expect(disclosure).not.toBeNull();
    fireEvent.click(within(disclosure!).getByText(/Reply to file/));
    expect(disclosure).toHaveTextContent("includes unsaved draft");
    expect(disclosure).toHaveTextContent("400 B omitted");
    expect(disclosure).toHaveTextContent("retained draft");
    expect(disclosure).toHaveTextContent(
      "artifact.write: approvalRequired (live-version-revalidation)",
    );
  });

  it("publishes controlled work, provenance, link, and artifact-focus intents", () => {
    const props = createProps();
    render(<ConversationTranscript {...props} />);

    fireEvent.click(screen.getByRole("button", { name: /Activity/i }));
    expect(props.onWorkExpandedChange).toHaveBeenCalledWith(
      "turn-assistant",
      false,
    );

    fireEvent.click(screen.getByRole("button", { name: "Run details" }));
    expect(props.onProvenanceExpandedChange).toHaveBeenCalledWith(
      "turn-assistant",
      false,
    );

    fireEvent.click(
      screen.getByRole("link", { name: "https://example.com/result" }),
    );
    expect(props.onLinkActivate).toHaveBeenCalledWith(
      "https://example.com/result",
    );

    fireEvent.click(screen.getByRole("button", { name: "Expand" }));
    expect(props.onArtifactFocusRequest).toHaveBeenCalledWith("artifact-file");
  });

  it("measures the real scroll host and scrolls to the latest message", () => {
    const props = createProps();
    const { container } = render(
      <main className="shell-workspace__stage">
        <ConversationTranscript {...props} />
      </main>,
    );
    const host = container.querySelector<HTMLElement>(
      ".shell-workspace__stage",
    );
    expect(host).not.toBeNull();
    Object.defineProperties(host as HTMLElement, {
      clientHeight: { configurable: true, value: 100 },
      scrollHeight: { configurable: true, value: 300 },
      scrollTop: { configurable: true, value: 50, writable: true },
      scrollTo: { configurable: true, value: vi.fn() },
    });
    fireEvent.scroll(host as HTMLElement);

    fireEvent.click(
      screen.getByRole("button", { name: "Scroll to latest message" }),
    );
    expect((host as HTMLElement).scrollTo).toHaveBeenCalledWith({
      behavior: "smooth",
      top: 300,
    });
  });

  it("uses the strict greater-than-24px scroll affordance threshold", () => {
    const props = createProps();
    const { container } = render(
      <main className="shell-workspace__stage">
        <ConversationTranscript {...props} />
      </main>,
    );
    const host = container.querySelector<HTMLElement>(
      ".shell-workspace__stage",
    ) as HTMLElement;
    Object.defineProperties(host, {
      clientHeight: { configurable: true, value: 100 },
      scrollHeight: { configurable: true, value: 124 },
      scrollTop: { configurable: true, value: 0, writable: true },
    });
    fireEvent.scroll(host);
    expect(
      screen.queryByRole("button", { name: "Scroll to latest message" }),
    ).toBeNull();

    Object.defineProperty(host, "scrollHeight", {
      configurable: true,
      value: 80,
    });
    fireEvent.scroll(host);
    expect(
      screen.queryByRole("button", { name: "Scroll to latest message" }),
    ).toBeNull();
  });

  it("exposes streaming busy state, completion announcements, and reduced-motion parity", () => {
    const props = createProps();
    const streaming: AssistantTranscriptTurn = {
      ...ASSISTANT_TURN,
      status: "streaming",
      responseVisible: false,
      announceCompletion: false,
      work: { ...ASSISTANT_TURN.work, isExpanded: false },
    };
    const { rerender } = render(
      <ConversationTranscript {...props} turns={[streaming]} reducedMotion />,
    );

    const response = screen.getByLabelText("C4OS response");
    expect(response).toHaveAttribute("aria-busy", "true");
    expect(screen.getByRole("button", { name: /Activity/i })).toHaveAttribute(
      "aria-expanded",
      "true",
    );
    expect(response).not.toHaveTextContent("C4OSGPT-5");
    expect(screen.getByLabelText("Conversation")).toHaveAttribute(
      "data-reduced-motion",
      "true",
    );

    rerender(<ConversationTranscript {...props} turns={[ASSISTANT_TURN]} />);
    expect(screen.getByLabelText("C4OS response")).toHaveAttribute(
      "aria-busy",
      "false",
    );
    expect(screen.getByRole("status")).toHaveTextContent(
      "C4OS response complete.",
    );
  });

  it("marks one real transcript for contextual-pane and focused-artifact composition", () => {
    const props = createProps();
    render(
      <ConversationTranscript
        {...props}
        placement="context-pane"
        focusedArtifactId="artifact-file"
      />,
    );

    const transcript = screen.getByLabelText("Contextual conversation");
    expect(transcript).toHaveAttribute("data-placement", "context-pane");
    expect(transcript).toHaveAttribute(
      "data-focused-artifact-id",
      "artifact-file",
    );
    expect(screen.getByLabelText("file response artifact")).toHaveAttribute(
      "data-artifact-placement",
      "context-pane",
    );
  });

  it("routes Stop and Retry while omitting an empty failure bubble", () => {
    const props = createProps();
    const onCancelAttempt = vi.fn();
    const onRetryAttempt = vi.fn();
    const streaming: AssistantTranscriptTurn = {
      ...ASSISTANT_TURN,
      status: "streaming",
    };
    const { container, rerender } = render(
      <ConversationTranscript
        {...props}
        onCancelAttempt={onCancelAttempt}
        onRetryAttempt={onRetryAttempt}
        turns={[streaming]}
      />,
    );

    fireEvent.click(screen.getByRole("button", { name: "Stop response" }));
    expect(onCancelAttempt).toHaveBeenCalledWith("turn-assistant");

    rerender(
      <ConversationTranscript
        {...props}
        onCancelAttempt={onCancelAttempt}
        onRetryAttempt={onRetryAttempt}
        turns={[
          {
            ...ASSISTANT_TURN,
            markdownSource: "",
            status: "failed",
          },
        ]}
      />,
    );
    expect(container.querySelector(".conversation-turn__bubble")).toBeNull();
    fireEvent.click(screen.getByRole("button", { name: "Retry response" }));
    expect(onRetryAttempt).toHaveBeenCalledWith("turn-assistant");
  });
});

describe("SafeMarkdown", () => {
  it("renders safe semantics without HTML injection or unsafe-scheme navigation", () => {
    const onLinkActivate = vi.fn();
    const { container } = render(
      <SafeMarkdown
        source={
          "# Heading\n<script>window.bad = true</script>\n[unsafe](javascript:alert(1))\n[safe](https://example.com/path)"
        }
        onLinkActivate={onLinkActivate}
      />,
    );

    expect(screen.getByRole("heading", { name: "Heading" })).toBeVisible();
    expect(container.querySelector("script")).toBeNull();
    expect(
      screen.getByText("<script>window.bad = true</script>"),
    ).toBeVisible();
    expect(screen.getByText("[unsafe](javascript:alert(1))")).toBeVisible();
    expect(screen.queryByRole("link", { name: "unsafe" })).toBeNull();

    fireEvent.click(screen.getByRole("link", { name: "safe" }));
    expect(onLinkActivate).toHaveBeenCalledWith("https://example.com/path");
  });
});

describe("PendingConversationPrompt", () => {
  it("renders the accepted memory-only pending Chat question", () => {
    render(<PendingConversationPrompt projectName="C4OS" />);

    expect(
      screen.getByRole("heading", {
        name: "What do you want to build in C4OS?",
      }),
    ).toBeVisible();
  });
});
