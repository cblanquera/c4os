import { fireEvent, render, screen, within } from "@testing-library/react";
import type { ReactElement } from "react";
import { describe, expect, it, vi } from "vitest";

import { ConversationTranscript } from "../../../../../src/frontend/features/conversation/ui";
import type { ConversationTranscriptProps } from "../../../../../src/frontend/features/conversation/ui";
import {
  ConversationFocusComposition,
  type FocusedConversationArtifact,
} from "../../../../../src/frontend/features/conversation/focus/ConversationFocusComposition";

/** Creates the smallest real transcript used by the composition tests. */
function createTranscript(): ReactElement<ConversationTranscriptProps> {
  return (
    <ConversationTranscript
      onCopy={vi.fn()}
      onProvenanceExpandedChange={vi.fn()}
      onReply={vi.fn()}
      onWorkExpandedChange={vi.fn()}
      reducedMotion={false}
      turns={[
        {
          id: "turn-user",
          author: "user",
          markdownSource: "Keep this real transcript.",
          status: "completed",
        },
      ]}
    />
  );
}

interface TestCompositionProps {
  readonly focusedArtifact: FocusedConversationArtifact | null;
  readonly onCloseFocusedArtifact?: (artifactId: string) => void;
  readonly onRestoreChat?: () => void;
}

/** Places the production slots in representative shell-owned hosts. */
function TestComposition({
  focusedArtifact,
  onCloseFocusedArtifact = vi.fn(),
  onRestoreChat = vi.fn(),
}: TestCompositionProps) {
  return (
    <ConversationFocusComposition
      focusedArtifact={focusedArtifact}
      onCloseFocusedArtifact={onCloseFocusedArtifact}
      onRestoreChat={onRestoreChat}
      transcript={createTranscript()}
    >
      {({ center, contextualChat }) => (
        <div>
          <main aria-label="Workspace center">{center}</main>
          <aside aria-label="Project panel">{contextualChat}</aside>
        </div>
      )}
    </ConversationFocusComposition>
  );
}

const FOCUSED_ARTIFACT: FocusedConversationArtifact = {
  content: <article aria-label="Focused file content">Document body</article>,
  id: "artifact-file",
  title: "result.md",
  type: "file",
};

describe("ConversationFocusComposition", () => {
  it("renders the single real transcript in center when focus is inactive", () => {
    render(<TestComposition focusedArtifact={null} />);

    // Ordinary Chat exposes one center transcript and no reserved pane.
    const center = screen.getByRole("region", {
      name: "Conversation center",
    });
    expect(within(center).getByLabelText("Conversation")).toHaveAttribute(
      "data-placement",
      "center",
    );
    expect(screen.getAllByRole("feed")).toHaveLength(1);
    expect(screen.queryByRole("region", { name: "Chat" })).toBeNull();
    expect(screen.queryByLabelText("Focused file content")).toBeNull();
  });

  it("moves the one transcript into contextual Chat while artifact owns center", () => {
    render(<TestComposition focusedArtifact={FOCUSED_ARTIFACT} />);

    // The focused provider surface is the only artifact content in center.
    const workspaceCenter = screen.getByRole("main", {
      name: "Workspace center",
    });
    expect(
      within(workspaceCenter).getByRole("region", { name: "result.md" }),
    ).toContainElement(screen.getByLabelText("Focused file content"));
    expect(within(workspaceCenter).queryByRole("feed")).toBeNull();

    // The original transcript appears once, now labeled for contextual use.
    const contextualChat = screen.getByRole("region", { name: "Chat" });
    expect(
      within(contextualChat).getByLabelText("Contextual conversation"),
    ).toHaveAttribute("data-placement", "context-pane");
    expect(
      within(contextualChat).getByLabelText("Contextual conversation"),
    ).toHaveAttribute("data-focused-artifact-id", "artifact-file");
    expect(screen.getAllByRole("feed")).toHaveLength(1);
    expect(screen.getAllByText("Keep this real transcript.")).toHaveLength(1);
  });

  it("marks provider-owned Close surfaces for the single-row focus grid", () => {
    render(
      <TestComposition
        focusedArtifact={{ ...FOCUSED_ARTIFACT, ownsClose: true }}
      />,
    );

    const focused = screen.getByRole("region", { name: "Focused result.md" });
    expect(focused).toHaveAttribute("data-artifact-owns-close", "true");
    expect(within(focused).queryByRole("button", { name: "Close" })).toBeNull();
  });

  it("preserves the original transcript DOM node across focus transitions", () => {
    const { rerender } = render(<TestComposition focusedArtifact={null} />);
    const originalFeed = screen.getByRole("feed");
    const originalTranscript = screen.getByLabelText("Conversation");

    rerender(<TestComposition focusedArtifact={FOCUSED_ARTIFACT} />);
    expect(screen.getByRole("feed")).toBe(originalFeed);
    expect(screen.getByLabelText("Contextual conversation")).toBe(
      originalTranscript,
    );
    expect(originalTranscript).toHaveAttribute(
      "data-placement",
      "context-pane",
    );

    rerender(<TestComposition focusedArtifact={null} />);
    expect(screen.getByRole("feed")).toBe(originalFeed);
    expect(screen.getByLabelText("Conversation")).toBe(originalTranscript);
    expect(originalTranscript).toHaveAttribute("data-placement", "center");
  });

  it("routes direct Close and contextual Restore without enabling Detach", () => {
    const onCloseFocusedArtifact = vi.fn();
    const onRestoreChat = vi.fn();
    render(
      <TestComposition
        focusedArtifact={FOCUSED_ARTIFACT}
        onCloseFocusedArtifact={onCloseFocusedArtifact}
        onRestoreChat={onRestoreChat}
      />,
    );

    // Focused center and contextual pane expose distinct recovery intents.
    fireEvent.click(screen.getByRole("button", { name: "Close" }));
    expect(onCloseFocusedArtifact).toHaveBeenCalledWith("artifact-file");

    fireEvent.click(screen.getByRole("button", { name: "Restore Chat" }));
    expect(onRestoreChat).toHaveBeenCalledTimes(1);
    expect(screen.getByRole("button", { name: "Detach Chat" })).toBeDisabled();
    expect(screen.getByRole("button", { name: "Detach Chat" })).toHaveAttribute(
      "title",
      "Detached Chat windows are not available",
    );
  });
});
