import {
  act,
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import { useState } from "react";
import { describe, expect, it, vi } from "vitest";

import { Composer } from "./Composer";
import type { ComposerAttachment, ComposerMode, ComposerProps } from "./index";

/** Supplies controlled source state while preserving observable callback behavior. */
function ComposerHarness(
  props: Omit<ComposerProps, "onValueChange" | "value"> & {
    readonly initialValue?: string;
    readonly onValueChange?: (value: string) => void;
  },
) {
  const { initialValue = "", onValueChange, ...composerProps } = props;
  const [value, setValue] = useState(initialValue);

  /** Keeps the harness controlled before reporting the exact source update. */
  const handleValueChange = (nextValue: string) => {
    setValue(nextValue);
    onValueChange?.(nextValue);
  };

  return (
    <Composer
      {...composerProps}
      onValueChange={handleValueChange}
      value={value}
    />
  );
}

/** Returns required inert callbacks so each test can override only its concern. */
function requiredProps(
  overrides: Partial<ComposerProps> = {},
): Omit<ComposerProps, "value"> {
  return {
    mode: "chat",
    onModeChange: vi.fn(),
    onRemoveAttachment: vi.fn(),
    onRemoveReply: vi.fn(),
    onSubmit: vi.fn(),
    onValueChange: vi.fn(),
    ...overrides,
  };
}

describe("Composer", () => {
  it("exposes a single-selection mode menu and restores its trigger on Escape", async () => {
    const onModeChange = vi.fn();
    render(
      <ComposerHarness
        {...requiredProps({ onModeChange })}
        initialValue="Describe the change"
      />,
    );

    const trigger = screen.getByRole("button", { name: "Composer mode" });
    expect(trigger).toHaveTextContent("Chat");
    act(() => trigger.focus());
    fireEvent.click(trigger);

    const menu = await screen.findByRole("menu");
    const options = within(menu).getAllByRole("menuitemradio");
    expect(options.map((option) => option.textContent)).toEqual([
      "Chat",
      "Files",
      "Browser",
      "Terminal",
    ]);
    expect(
      within(menu).getByRole("menuitemradio", { name: "Chat" }),
    ).toHaveAttribute("aria-checked", "true");

    fireEvent.keyDown(menu, { key: "Escape" });
    await waitFor(() => expect(screen.queryByRole("menu")).toBeNull());
    expect(trigger).toHaveFocus();

    fireEvent.click(trigger);
    const reopenedMenu = await screen.findByRole("menu");
    fireEvent.click(
      within(reopenedMenu).getByRole("menuitemradio", { name: "Files" }),
    );
    expect(onModeChange).toHaveBeenCalledWith("files");
  });

  it("sends source on Enter while leaving Shift+Enter to native multiline editing", () => {
    const onSubmit = vi.fn();
    render(
      <ComposerHarness
        {...requiredProps({ onSubmit })}
        initialValue="First line"
      />,
    );

    const editor = screen.getByRole("textbox", { name: "Message" });
    fireEvent.keyDown(editor, { key: "Enter", shiftKey: true });
    expect(onSubmit).not.toHaveBeenCalled();

    fireEvent.keyDown(editor, { key: "Enter" });
    expect(onSubmit).toHaveBeenCalledWith({
      attachmentIds: [],
      mode: "chat",
      source: "First line",
    });
  });

  it("wraps and toggles selected source with platform Markdown shortcuts", () => {
    render(
      <ComposerHarness {...requiredProps()} initialValue="make this bold" />,
    );

    const editor = screen.getByRole("textbox", {
      name: "Message",
    }) as HTMLTextAreaElement;
    editor.focus();
    editor.setSelectionRange(10, 14);
    fireEvent.keyDown(editor, { key: "b", metaKey: true });
    expect(editor).toHaveValue("make this **bold**");
    expect(editor.selectionStart).toBe(12);
    expect(editor.selectionEnd).toBe(16);

    fireEvent.keyDown(editor, { key: "b", ctrlKey: true });
    expect(editor).toHaveValue("make this bold");
    expect(editor.selectionStart).toBe(10);
    expect(editor.selectionEnd).toBe(14);

    editor.setSelectionRange(10, 14);
    fireEvent.keyDown(editor, { key: "i", metaKey: true });
    expect(editor).toHaveValue("make this *bold*");
  });

  it("creates a safe Markdown link when a URL is pasted over selected source", () => {
    render(
      <ComposerHarness {...requiredProps()} initialValue="Read the docs" />,
    );

    const editor = screen.getByRole("textbox", {
      name: "Message",
    }) as HTMLTextAreaElement;
    editor.setSelectionRange(9, 13);
    fireEvent.paste(editor, {
      clipboardData: {
        getData: () => "https://example.com/guide",
      },
    });
    expect(editor).toHaveValue("Read the [docs](https://example.com/guide)");

    editor.setSelectionRange(0, 4);
    fireEvent.paste(editor, {
      clipboardData: {
        getData: () => "javascript:alert(1)",
      },
    });
    expect(editor).toHaveValue("Read the [docs](https://example.com/guide)");
  });

  it("shows newest attachments first without renumbering and allows attachment-only send", () => {
    const attachments: readonly ComposerAttachment[] = [
      {
        id: "attachment-1",
        fileName: "brief.md",
        sizeLabel: "4 KB",
        compatibility: "ready",
        referenceNumber: 1,
      },
      {
        id: "attachment-2",
        fileName: "screen.png",
        sizeLabel: "2 MB",
        compatibility: "needs-vision",
        previewUrl: "data:image/png;base64,AA==",
        referenceNumber: 2,
      },
      {
        id: "attachment-3",
        fileName: "notes.txt",
        sizeLabel: "10 KB",
        compatibility: "converted",
        referenceNumber: 3,
      },
    ];
    const onRemoveAttachment = vi.fn();
    const onSubmit = vi.fn();
    render(
      <ComposerHarness
        {...requiredProps({
          attachments,
          onRemoveAttachment,
          onSubmit,
        })}
      />,
    );

    const list = within(
      screen.getByRole("region", {
        name: "Draft attachments",
      }),
    ).getByRole("list");
    expect(
      within(list)
        .getAllByRole("listitem")
        .map(
          (item) =>
            item.querySelector(".conversation-composer__attachment-reference")
              ?.textContent,
        ),
    ).toEqual(["#3", "#2", "#1"]);
    expect(within(list).getByText("Needs Vision")).toBeVisible();
    expect(within(list).getByText("Converted")).toBeVisible();

    fireEvent.click(
      screen.getByRole("button", {
        name: "Remove screen.png attachment #2",
      }),
    );
    expect(onRemoveAttachment).toHaveBeenCalledWith("attachment-2");

    fireEvent.click(screen.getByRole("button", { name: "Send" }));
    expect(onSubmit).toHaveBeenCalledWith({
      attachmentIds: ["attachment-1", "attachment-2", "attachment-3"],
      mode: "chat",
      source: "",
    });
  });

  it("blocks submission and exposes every authoritative conflict resolution", () => {
    const onConflictAction = vi.fn();
    render(
      <ComposerHarness
        {...requiredProps({
          conflict: {
            attachmentId: "attachment-2",
            title: "screen.png needs vision",
            description: "The active model cannot read this image.",
            actions: [
              "use-compatible-model",
              "convert",
              "remove-file",
              "cancel",
            ],
          },
          onConflictAction,
        })}
        initialValue="Describe this image"
      />,
    );

    expect(screen.getByRole("alert")).toHaveTextContent(
      "The active model cannot read this image.",
    );
    expect(screen.getByRole("button", { name: "Send" })).toBeDisabled();

    for (const [name, action] of [
      ["Use compatible model", "use-compatible-model"],
      ["Convert", "convert"],
      ["Remove file", "remove-file"],
      ["Cancel", "cancel"],
    ] as const) {
      fireEvent.click(screen.getByRole("button", { name }));
      expect(onConflictAction).toHaveBeenLastCalledWith(action);
    }
  });

  it("renders Reply as natural-language Chat with a removable reference and no mode trigger", () => {
    const onRemoveReply = vi.fn();
    const onSubmit = vi.fn();
    render(
      <ComposerHarness
        {...requiredProps({
          mode: "reply",
          onRemoveReply,
          onSubmit,
          replyReference: {
            id: "artifact-9",
            kind: "terminal",
            label: "Terminal command",
            excerpt: "npm test failed in workspace",
          },
        })}
        initialValue="Retry the focused test"
      />,
    );

    expect(screen.queryByRole("button", { name: "Composer mode" })).toBeNull();
    expect(
      screen.getByRole("region", { name: "Reply reference" }),
    ).toHaveTextContent("Terminal commandnpm test failed in workspace");
    fireEvent.click(
      screen.getByRole("button", { name: "Remove reply reference" }),
    );
    expect(onRemoveReply).toHaveBeenCalledTimes(1);

    fireEvent.keyDown(screen.getByRole("textbox", { name: "Reply" }), {
      key: "Enter",
    });
    expect(onSubmit).toHaveBeenCalledWith({
      attachmentIds: [],
      mode: "reply",
      replyReferenceId: "artifact-9",
      source: "Retry the focused test",
    });
  });

  it("shows only the mode-specific control slots and primary input semantics", () => {
    const controls = {
      model: <button type="button">Sonnet 4</button>,
      reasoning: <button type="button">Reasoning: High</button>,
      approval: <button type="button">Ask for approval</button>,
      branch: <button type="button">main</button>,
    };
    const { rerender } = render(
      <Composer {...requiredProps({ controls, mode: "chat" })} value="hello" />,
    );

    expect(screen.getByRole("group", { name: "Model" })).toBeVisible();
    expect(
      screen.getByRole("group", { name: "Reasoning effort" }),
    ).toBeVisible();
    expect(
      screen.getByRole("group", { name: "Approval preset" }),
    ).toBeVisible();
    expect(screen.getByRole("group", { name: "Git branch" })).toBeVisible();

    rerender(
      <Composer
        {...requiredProps({ controls, mode: "files" })}
        value="/project/README.md"
      />,
    );
    expect(
      screen.getByRole("textbox", { name: "File or folder path" }),
    ).toBeVisible();
    expect(screen.queryByRole("group", { name: "Model" })).toBeNull();
    expect(screen.getByRole("group", { name: "Git branch" })).toBeVisible();
    expect(screen.getByRole("button", { name: "Open" })).toBeEnabled();

    for (const [mode, name, action] of [
      ["browser", "Web address", "Open"],
      ["terminal", "Terminal command", "Run"],
    ] as const) {
      rerender(
        <Composer
          {...requiredProps({ controls, mode: mode as ComposerMode })}
          value="command"
        />,
      );
      expect(screen.getByRole("textbox", { name })).toBeVisible();
      expect(screen.queryByRole("group", { name: "Git branch" })).toBeNull();
      expect(screen.getByRole("button", { name: action })).toBeEnabled();
    }
  });
});
