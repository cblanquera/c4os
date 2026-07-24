import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { FolderArtifact } from "../../../../../src/frontend/features/artifacts/folder/FolderArtifact";
import type { FolderArtifactModel } from "../../../../../src/frontend/features/artifacts/folder/FolderArtifact";

const READY_MODEL: FolderArtifactModel = {
  artifactId: "artifact:folder-src",
  breadcrumbs: [
    { id: "root", label: "project" },
    { id: "src", isCurrent: true, label: "src" },
  ],
  entries: [
    { id: "folder-components", kind: "folder", name: "components" },
    {
      id: "file-app",
      kind: "file",
      metadata: "4 KB · TypeScript",
      name: "App.tsx",
    },
    { id: "file-main", kind: "file", name: "main.tsx" },
  ],
  listing: { phase: "ready" },
  listingLimit: 2,
  selectedEntryId: "file-app",
  title: "src",
};

describe("FolderArtifact", () => {
  it("bounds rows and emits breadcrumb, refresh, navigation, and conversion intents", () => {
    const onBreadcrumbSelect = vi.fn();
    const onConvertFile = vi.fn();
    const onCopy = vi.fn();
    const onNavigateFolder = vi.fn();
    const onRefresh = vi.fn();
    render(
      <FolderArtifact
        context="inline"
        model={READY_MODEL}
        onBreadcrumbSelect={onBreadcrumbSelect}
        onConvertFile={onConvertFile}
        onCopy={onCopy}
        onNavigateFolder={onNavigateFolder}
        onRefresh={onRefresh}
      />,
    );

    expect(screen.getAllByRole("listitem")).toHaveLength(4);
    expect(
      screen.getByText("2 shown · 1 omitted by listing limit"),
    ).toBeVisible();
    expect(
      screen.getByRole("button", { name: "Open file App.tsx" }),
    ).toBeVisible();
    expect(
      screen.queryByRole("button", { name: "Open file main.tsx" }),
    ).toBeNull();

    fireEvent.click(screen.getByRole("button", { name: "project" }));
    fireEvent.click(screen.getByRole("button", { name: "Refresh" }));
    fireEvent.click(
      screen.getByRole("button", { name: "Open folder components" }),
    );
    fireEvent.click(screen.getByRole("button", { name: "Open file App.tsx" }));
    fireEvent.click(screen.getByRole("button", { name: "Copy" }));
    expect(onBreadcrumbSelect).toHaveBeenCalledWith(
      "artifact:folder-src",
      "root",
    );
    expect(onRefresh).toHaveBeenCalledWith("artifact:folder-src");
    expect(onNavigateFolder).toHaveBeenCalledWith(
      "artifact:folder-src",
      "folder-components",
    );
    expect(onConvertFile).toHaveBeenCalledWith(
      "artifact:folder-src",
      "file-app",
    );
    expect(onCopy).toHaveBeenCalledWith(
      "artifact:folder-src",
      "folder: components\nfile: App.tsx",
    );
  });

  it("renders empty, loading, and error states with accessible status", () => {
    const { rerender } = render(
      <FolderArtifact
        context="inline"
        model={{ ...READY_MODEL, entries: [], listingLimit: 20 }}
      />,
    );
    expect(screen.getByText("This folder is empty.")).toBeVisible();
    expect(screen.getByText("0 items")).toBeVisible();

    rerender(
      <FolderArtifact
        context="inline"
        model={{
          ...READY_MODEL,
          entries: [],
          listing: {
            message: "Reading bounded folder contents",
            phase: "loading",
          },
        }}
      />,
    );
    expect(screen.getByRole("status")).toHaveTextContent(
      "Reading bounded folder contents",
    );
    expect(screen.queryByText("This folder is empty.")).toBeNull();

    rerender(
      <FolderArtifact
        context="inline"
        model={{
          ...READY_MODEL,
          entries: [],
          listing: { message: "Folder grant expired", phase: "error" },
        }}
      />,
    );
    expect(screen.getByRole("alert")).toHaveTextContent("Folder grant expired");
  });

  it("keeps contextual Folder controls read-only while retaining Expand", () => {
    render(
      <FolderArtifact
        context="contextual"
        model={READY_MODEL}
        onBreadcrumbSelect={vi.fn()}
        onConvertFile={vi.fn()}
        onExpand={vi.fn()}
        onNavigateFolder={vi.fn()}
        onRefresh={vi.fn()}
      />,
    );

    expect(screen.getByRole("button", { name: "project" })).toBeDisabled();
    expect(screen.getByRole("button", { name: "Refresh" })).toBeDisabled();
    expect(screen.getByRole("button", { name: "Expand" })).toBeVisible();
  });

  it("captures the last focused Folder entry for Reply", () => {
    const onReply = vi.fn();
    render(
      <FolderArtifact
        context="inline"
        model={{ ...READY_MODEL, selectedEntryId: undefined }}
        onConvertFile={vi.fn()}
        onNavigateFolder={vi.fn()}
        onReply={onReply}
      />,
    );

    fireEvent.focus(
      screen.getByRole("button", { name: "Open folder components" }),
    );
    fireEvent.click(screen.getByRole("button", { name: "Reply" }));

    expect(onReply).toHaveBeenCalledWith("artifact:folder-src", {
      selectedEntryId: "folder-components",
    });
  });

  it("drops a focused entry that is absent after in-place navigation", () => {
    const onReply = vi.fn();
    const { rerender } = render(
      <FolderArtifact
        context="inline"
        model={{ ...READY_MODEL, selectedEntryId: undefined }}
        onConvertFile={vi.fn()}
        onNavigateFolder={vi.fn()}
        onReply={onReply}
      />,
    );
    fireEvent.focus(
      screen.getByRole("button", { name: "Open folder components" }),
    );

    rerender(
      <FolderArtifact
        context="inline"
        model={{
          ...READY_MODEL,
          breadcrumbs: [
            ...READY_MODEL.breadcrumbs.map((breadcrumb) => ({
              ...breadcrumb,
              isCurrent: false,
            })),
            { id: "src/components", isCurrent: true, label: "components" },
          ],
          entries: [
            {
              id: "file-button",
              kind: "file",
              name: "Button.tsx",
            },
          ],
          selectedEntryId: undefined,
        }}
        onConvertFile={vi.fn()}
        onNavigateFolder={vi.fn()}
        onReply={onReply}
      />,
    );
    fireEvent.click(screen.getByRole("button", { name: "Reply" }));

    expect(onReply).toHaveBeenCalledWith("artifact:folder-src");
  });

  it("preserves the native degraded lifecycle status", () => {
    render(
      <FolderArtifact
        context="inline"
        model={{
          ...READY_MODEL,
          status: { kind: "degraded", message: "Listing is partial" },
        }}
      />,
    );

    expect(screen.getByRole("status")).toHaveTextContent("Listing is partial");
  });
});
