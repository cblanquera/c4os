import { fireEvent, render, screen, within } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { FileArtifact } from "./FileArtifact";
import type { FileArtifactModel, FileArtifactProps } from "./FileArtifact";

const READ_MODEL: FileArtifactModel = {
  artifactId: "artifact:file-readme",
  breadcrumbs: [
    { id: "root", label: "project" },
    { id: "src", label: "src" },
    { id: "readme", isCurrent: true, label: "README.md" },
  ],
  languageLabel: "Markdown",
  state: { content: "first line\nsecond line", phase: "read" },
  title: "README.md",
  versionLabel: "Version 4",
};

/** Builds controlled provider callbacks so each test can assert emitted intent. */
function callbacks(): Required<
  Pick<
    FileArtifactProps,
    | "onAllowApproval"
    | "onApproveProposal"
    | "onBreadcrumbSelect"
    | "onClose"
    | "onCopy"
    | "onDiscard"
    | "onDenyApproval"
    | "onDraftChange"
    | "onEdit"
    | "onExpand"
    | "onRecoverDraft"
    | "onRejectProposal"
    | "onReply"
    | "onResolveConflict"
    | "onSave"
  >
> {
  return {
    onAllowApproval: vi.fn(),
    onApproveProposal: vi.fn(),
    onBreadcrumbSelect: vi.fn(),
    onClose: vi.fn(),
    onCopy: vi.fn(),
    onDiscard: vi.fn(),
    onDenyApproval: vi.fn(),
    onDraftChange: vi.fn(),
    onEdit: vi.fn(),
    onExpand: vi.fn(),
    onRecoverDraft: vi.fn(),
    onRejectProposal: vi.fn(),
    onReply: vi.fn(),
    onResolveConflict: vi.fn(),
    onSave: vi.fn(),
  };
}

describe("FileArtifact", () => {
  it("renders read state, breadcrumbs, line numbers, and contextual actions", () => {
    const actions = callbacks();
    render(<FileArtifact context="inline" model={READ_MODEL} {...actions} />);

    expect(
      screen.getByRole("article", {
        name: "File response artifact: README.md",
      }),
    ).toBeVisible();
    const content = screen.getByRole("list", {
      name: "Contents of README.md",
    });
    expect(within(content).getAllByRole("listitem")).toHaveLength(2);
    expect(screen.getByText("Version 4 · Markdown · Read only")).toBeVisible();

    fireEvent.click(screen.getByRole("button", { name: "src" }));
    fireEvent.click(screen.getByRole("button", { name: "Edit" }));
    fireEvent.click(screen.getByRole("button", { name: "Copy" }));
    fireEvent.click(screen.getByRole("button", { name: "Reply" }));
    fireEvent.click(screen.getByRole("button", { name: "Expand" }));
    expect(actions.onBreadcrumbSelect).toHaveBeenCalledWith(
      "artifact:file-readme",
      "src",
    );
    expect(actions.onEdit).toHaveBeenCalledWith("artifact:file-readme");
    expect(actions.onCopy).toHaveBeenCalledWith(
      "artifact:file-readme",
      "first line\nsecond line",
    );
    expect(actions.onReply).toHaveBeenCalledWith("artifact:file-readme");
    expect(actions.onExpand).toHaveBeenCalledWith("artifact:file-readme");
  });

  it("emits dirty editor changes while contextual Chat stays read-only", () => {
    const actions = callbacks();
    const dirtyModel: FileArtifactModel = {
      ...READ_MODEL,
      state: {
        content: "first line\nsecond line",
        draft: "first line\nchanged line\n",
        phase: "dirty",
      },
    };
    const { rerender } = render(
      <FileArtifact context="focused" model={dirtyModel} {...actions} />,
    );

    const editor = screen.getByRole("textbox", { name: "Edit README.md" });
    expect(editor).toHaveValue("first line\nchanged line\n");
    expect(screen.getByRole("status")).toHaveTextContent("Unsaved changes");
    fireEvent.change(editor, { target: { value: "replacement" } });
    fireEvent.click(screen.getByRole("button", { name: "Save" }));
    fireEvent.click(screen.getByRole("button", { name: "Discard" }));
    expect(actions.onDraftChange).toHaveBeenCalledWith(
      "artifact:file-readme",
      "replacement",
    );
    expect(actions.onSave).toHaveBeenCalledWith(
      "artifact:file-readme",
      "first line\nchanged line\n",
    );
    expect(actions.onDiscard).toHaveBeenCalledWith("artifact:file-readme");

    rerender(
      <FileArtifact context="contextual" model={dirtyModel} {...actions} />,
    );
    expect(
      screen.queryByRole("textbox", { name: "Edit README.md" }),
    ).toBeNull();
    expect(screen.queryByRole("button", { name: "Save" })).toBeNull();
    expect(screen.queryByRole("button", { name: "Discard" })).toBeNull();
    expect(screen.getByText("changed line")).toBeVisible();
    expect(screen.getByRole("button", { name: "src" })).toBeDisabled();
    expect(screen.getByRole("button", { name: "Expand" })).toBeVisible();
  });

  it("captures an exact visible File selection for Reply", () => {
    const actions = callbacks();
    render(<FileArtifact context="inline" model={READ_MODEL} {...actions} />);
    const selection = window.getSelection();
    const range = document.createRange();
    range.selectNodeContents(screen.getByText("second line"));
    selection?.removeAllRanges();
    selection?.addRange(range);

    fireEvent.click(screen.getByRole("button", { name: "Reply" }));

    expect(actions.onReply).toHaveBeenCalledWith("artifact:file-readme", {
      selectedText: "second line",
    });
    selection?.removeAllRanges();
  });

  it("ignores matching text selected outside this File artifact", () => {
    const actions = callbacks();
    render(
      <>
        <p data-testid="outside-selection">second line</p>
        <FileArtifact context="inline" model={READ_MODEL} {...actions} />
      </>,
    );
    const selection = window.getSelection();
    const range = document.createRange();
    range.selectNodeContents(screen.getByTestId("outside-selection"));
    selection?.removeAllRanges();
    selection?.addRange(range);

    fireEvent.click(screen.getByRole("button", { name: "Reply" }));

    expect(actions.onReply).toHaveBeenCalledWith("artifact:file-readme");
    selection?.removeAllRanges();
  });

  it("renders proposal, approval, conflict, and recovery intents", () => {
    const actions = callbacks();
    const proposed: FileArtifactModel = {
      ...READ_MODEL,
      state: {
        content: "old",
        phase: "proposed",
        proposedContent: "new",
        proposalDiff: "-old\n+new",
        proposalSummary: "Replace one line",
      },
    };
    const { rerender } = render(
      <FileArtifact context="inline" model={proposed} {...actions} />,
    );
    expect(screen.getByRole("status")).toHaveTextContent(
      "Proposed change · Replace one line",
    );
    expect(screen.getByLabelText("Proposed File diff")).toHaveTextContent(
      "-old +new",
    );
    fireEvent.click(screen.getByRole("button", { name: "Approve" }));
    fireEvent.click(screen.getByRole("button", { name: "Reject" }));
    expect(actions.onApproveProposal).toHaveBeenCalledWith(
      "artifact:file-readme",
    );
    expect(actions.onRejectProposal).toHaveBeenCalledWith(
      "artifact:file-readme",
    );

    rerender(
      <FileArtifact
        context="inline"
        model={{
          ...READ_MODEL,
          state: {
            approvalId: "approval:file-write",
            approvalSummary: "Waiting for path approval",
            content: "old",
            phase: "approval",
            proposedContent: "new",
          },
        }}
        {...actions}
      />,
    );
    expect(screen.getByRole("status")).toHaveTextContent(
      "Approval pending · Waiting for path approval",
    );
    fireEvent.click(screen.getByRole("button", { name: "Allow" }));
    fireEvent.click(screen.getByRole("button", { name: "Deny" }));
    expect(actions.onAllowApproval).toHaveBeenCalledWith(
      "artifact:file-readme",
      "approval:file-write",
    );
    expect(actions.onDenyApproval).toHaveBeenCalledWith(
      "artifact:file-readme",
      "approval:file-write",
    );

    rerender(
      <FileArtifact
        context="inline"
        model={{
          ...READ_MODEL,
          state: {
            conflictMessage: "The file changed after editing",
            content: "current",
            currentVersionLabel: "Version 5",
            draft: "local",
            phase: "conflict",
          },
        }}
        {...actions}
      />,
    );
    expect(screen.getByRole("alert")).toHaveTextContent("Save conflict");
    fireEvent.click(screen.getByRole("button", { name: "Reload current" }));
    fireEvent.click(screen.getByRole("button", { name: "Keep draft" }));
    expect(actions.onResolveConflict).toHaveBeenNthCalledWith(
      1,
      "artifact:file-readme",
      "reload-current",
    );
    expect(actions.onResolveConflict).toHaveBeenNthCalledWith(
      2,
      "artifact:file-readme",
      "keep-draft",
    );

    rerender(
      <FileArtifact
        context="inline"
        model={{
          ...READ_MODEL,
          state: {
            content: "current",
            draft: "recovered",
            phase: "recovery",
            recoveryMessage: "Unsaved draft from the last session",
          },
        }}
        {...actions}
      />,
    );
    expect(screen.getByRole("status")).toHaveTextContent("Recovered draft");
    fireEvent.click(screen.getByRole("button", { name: "Recover draft" }));
    expect(actions.onRecoverDraft).toHaveBeenCalledWith("artifact:file-readme");
  });

  it("keeps approval decisions controlled, accessible, and contextual-read-only", () => {
    const approvalModel: FileArtifactModel = {
      ...READ_MODEL,
      state: {
        approvalId: "approval:guarded-write",
        approvalSummary: "Approve the exact bounded file diff",
        content: "old",
        phase: "approval",
        proposedContent: "new",
      },
    };
    const { rerender } = render(
      <FileArtifact context="focused" model={approvalModel} />,
    );

    expect(screen.getByRole("button", { name: "Allow" })).toBeDisabled();
    expect(screen.getByRole("button", { name: "Deny" })).toBeDisabled();
    expect(screen.getByRole("status")).toHaveTextContent(
      "Approve the exact bounded file diff",
    );

    rerender(<FileArtifact context="contextual" model={approvalModel} />);
    expect(screen.queryByRole("button", { name: "Allow" })).toBeNull();
    expect(screen.queryByRole("button", { name: "Deny" })).toBeNull();
    expect(screen.getByRole("status")).toHaveTextContent("Approval pending");
  });
});
