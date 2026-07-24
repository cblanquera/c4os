import {
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import { useState } from "react";
import { describe, expect, it, vi } from "vitest";

import {
  ActionMenu,
  Button,
  Disclosure,
  Icon,
  IconButton,
  LiveRegion,
  ModalDialog,
  PopoverDialog,
  Resizer,
  ScanRow,
  Switch,
  Tabs,
} from "./index";

/** Renders the controlled resizer contract used by shell panels. */
function ResizerHarness() {
  const [value, setValue] = useState(228);

  return (
    <Resizer
      label="Resize projects panel"
      max={420}
      min={180}
      onValueChange={setValue}
      orientation="vertical"
      step={8}
      value={value}
    />
  );
}

describe("accessible controls", () => {
  it("keeps button activation semantic and gives icon actions a focus tooltip", async () => {
    const onPress = vi.fn();
    render(
      <>
        <Button onPress={onPress}>Create chat</Button>
        <IconButton icon={<Icon name="settings" />} label="Chat information" />
      </>,
    );

    const createButton = screen.getByRole("button", { name: "Create chat" });
    fireEvent.keyDown(createButton, { key: "Enter" });
    fireEvent.keyUp(createButton, { key: "Enter" });
    expect(onPress).toHaveBeenCalledTimes(1);

    const iconButton = screen.getByRole("button", {
      name: "Chat information",
    });
    iconButton.focus();
    expect(await screen.findByRole("tooltip")).toHaveTextContent(
      "Chat information",
    );
  });

  it("dismisses menu and popover overlays with Escape and restores triggers", async () => {
    render(
      <>
        <ActionMenu
          items={[
            {
              id: "rename",
              label: "Rename",
              onAction: vi.fn(),
            },
          ]}
          label="Project actions"
          triggerIcon={<Icon name="more" />}
        />
        <PopoverDialog
          title="Chat information"
          triggerIcon={<Icon name="info" />}
          triggerLabel="Open chat information"
        >
          Runtime and effective capability details.
        </PopoverDialog>
      </>,
    );

    const menuTrigger = screen.getByRole("button", {
      name: "Project actions",
    });
    fireEvent.click(menuTrigger);
    expect(
      await screen.findByRole("menu", { name: "Project actions" }),
    ).toBeVisible();
    fireEvent.keyDown(screen.getByRole("menu"), { key: "Escape" });
    await waitFor(() =>
      expect(
        screen.queryByRole("menu", { name: "Project actions" }),
      ).toBeNull(),
    );
    await waitFor(() => expect(menuTrigger).toHaveFocus());

    const popoverTrigger = screen.getByRole("button", {
      name: "Open chat information",
    });
    popoverTrigger.focus();
    fireEvent.click(popoverTrigger);
    const popover = await screen.findByRole("dialog", {
      name: "Chat information",
    });
    expect(popover).toContainElement(
      screen.getByText("Runtime and effective capability details."),
    );
    fireEvent.keyDown(popover, { key: "Escape" });
    await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
    await waitFor(() => expect(popoverTrigger).toHaveFocus());
  });

  it("contains modal focus, supports Escape and backdrop dismissal, and restores focus", async () => {
    render(
      <div>
        <button type="button">Outside action</button>
        <ModalDialog title="Remove project" triggerLabel="Open removal">
          This keeps files on disk.
        </ModalDialog>
      </div>,
    );

    const trigger = screen.getByRole("button", { name: "Open removal" });
    const outside = screen.getByRole("button", { name: "Outside action" });
    trigger.focus();
    fireEvent.click(trigger);
    const dialog = await screen.findByRole("dialog", {
      name: "Remove project",
    });
    expect(dialog).toBeVisible();
    expect(outside.closest("[aria-hidden=true]")).not.toBeNull();
    await waitFor(() =>
      expect(dialog).toContainElement(document.activeElement as HTMLElement),
    );

    fireEvent.keyDown(document.activeElement ?? dialog, { key: "Escape" });
    await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
    await waitFor(() => expect(trigger).toHaveFocus());

    fireEvent.click(trigger);
    await screen.findByRole("dialog", { name: "Remove project" });
    const backdrop = document.querySelector<HTMLElement>(".c4-modal-overlay");
    expect(backdrop).not.toBeNull();
    fireEvent.pointerDown(backdrop!, {
      button: 0,
      pointerId: 11,
      pointerType: "mouse",
    });
    fireEvent.pointerUp(backdrop!, {
      button: 0,
      pointerId: 11,
      pointerType: "mouse",
    });
    fireEvent.click(backdrop!);
    await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
    await waitFor(() => expect(trigger).toHaveFocus());
  });

  it("connects disclosure and tab keyboard semantics", () => {
    render(
      <>
        <Disclosure title="Runtime details">
          <p>OpenCode is ready.</p>
        </Disclosure>
        <Tabs
          label="Settings sections"
          tabs={[
            { id: "providers", label: "Providers", content: "Provider list" },
            { id: "models", label: "Models", content: "Model list" },
          ]}
        />
      </>,
    );

    const disclosure = screen.getByRole("button", { name: "Runtime details" });
    expect(disclosure).toHaveAttribute("aria-expanded", "false");
    fireEvent.click(disclosure);
    expect(disclosure).toHaveAttribute("aria-expanded", "true");
    expect(screen.getByText("OpenCode is ready.")).toBeVisible();

    const providers = screen.getByRole("tab", { name: "Providers" });
    const models = screen.getByRole("tab", { name: "Models" });
    providers.focus();
    fireEvent.keyDown(providers, { key: "ArrowRight" });
    expect(models).toHaveAttribute("aria-selected", "true");
    expect(screen.getByRole("tabpanel")).toHaveTextContent("Model list");
  });

  it("exposes switch and atomic live-region state", () => {
    const onChange = vi.fn();
    render(
      <>
        <Switch
          description="Applies to future safe actions."
          label="Approve safe actions"
          onChange={onChange}
        />
        <LiveRegion>Response complete.</LiveRegion>
      </>,
    );

    const toggle = screen.getByRole("switch", {
      name: /Approve safe actions/,
    });
    expect(toggle).not.toBeChecked();
    fireEvent.click(toggle);
    expect(toggle).toBeChecked();
    expect(onChange).toHaveBeenLastCalledWith(true);

    const status = screen.getByRole("status");
    expect(status).toHaveAttribute("aria-live", "polite");
    expect(status).toHaveAttribute("aria-atomic", "true");
    expect(status).toHaveTextContent("Response complete.");
  });
});

describe("accessible layout contracts", () => {
  it("publishes resizer range state and handles bounded keyboard and pointer changes", () => {
    render(<ResizerHarness />);

    const resizer = screen.getByRole("separator", {
      name: "Resize projects panel",
    });
    expect(resizer).toHaveAttribute("aria-valuemin", "180");
    expect(resizer).toHaveAttribute("aria-valuemax", "420");
    expect(resizer).toHaveAttribute("aria-valuenow", "228");

    fireEvent.keyDown(resizer, { key: "ArrowRight" });
    expect(resizer).toHaveAttribute("aria-valuenow", "236");
    expect(resizer).toHaveAttribute("aria-valuetext", "236 pixels");
    fireEvent.keyDown(resizer, { key: "End" });
    expect(resizer).toHaveAttribute("aria-valuenow", "420");
    fireEvent.keyDown(resizer, { key: "ArrowRight" });
    expect(resizer).toHaveAttribute("aria-valuenow", "420");

    fireEvent.pointerDown(resizer, {
      button: 0,
      clientX: 200,
      pointerId: 7,
    });
    fireEvent.pointerMove(resizer, { clientX: 150, pointerId: 7 });
    expect(resizer).toHaveAttribute("aria-valuenow", "370");
    fireEvent.pointerUp(resizer, { clientX: 150, pointerId: 7 });
  });

  it("keeps row actions rendered and focusable before hover to prohibit layout shifts", () => {
    render(
      <ScanRow
        actions={<button type="button">Rename</button>}
        description="Local project"
        metadata="Ready"
        title="C4OS"
      />,
    );

    const row = screen.getByRole("article");
    const actions = screen.getByLabelText("Row actions");
    const action = within(actions).getByRole("button", { name: "Rename" });
    expect(row).toHaveAttribute("data-layout", "reserved-actions");
    expect(actions).not.toHaveAttribute("hidden");
    expect(actions).not.toHaveAttribute("aria-hidden");
    action.focus();
    expect(action).toHaveFocus();
  });
});
