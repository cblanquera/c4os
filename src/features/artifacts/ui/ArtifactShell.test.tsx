import { fireEvent, render, screen, within } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { ArtifactShell } from "./ArtifactShell";
import { UnknownArtifact } from "./UnknownArtifact";

const IDENTITY = {
  accessibleLabel: "File response artifact: README.md",
  id: "artifact:file-one",
  title: "README.md",
  typeLabel: "File",
};

describe("ArtifactShell", () => {
  it("keeps static chrome outside the only scrolling body and reserves actions", () => {
    const onCopy = vi.fn();
    const onExpand = vi.fn();
    const onReply = vi.fn();
    const { container } = render(
      <ArtifactShell
        context="inline"
        focusSupported
        footerContent="Version 3"
        headerContent={<nav aria-label="File breadcrumbs">src / README.md</nav>}
        identity={IDENTITY}
        onCopy={onCopy}
        onExpand={onExpand}
        onReply={onReply}
      >
        <pre>Document body</pre>
      </ArtifactShell>,
    );

    const artifact = screen.getByRole("article", {
      name: "File response artifact: README.md",
    });
    const children = Array.from(artifact.children);
    expect(children[0]).toHaveAttribute("data-artifact-static", "header");
    expect(children[1]).toHaveAttribute("data-artifact-scroll-region", "body");
    expect(children[2]).toHaveAttribute("data-artifact-static", "footer");
    expect(
      container.querySelectorAll("[data-artifact-scroll-region]"),
    ).toHaveLength(1);
    expect(within(artifact).getByText("Document body")).toBeVisible();

    fireEvent.click(screen.getByRole("button", { name: "Copy" }));
    fireEvent.click(screen.getByRole("button", { name: "Reply" }));
    fireEvent.click(screen.getByRole("button", { name: "Expand" }));
    expect(onCopy).toHaveBeenCalledTimes(1);
    expect(onReply).toHaveBeenCalledTimes(1);
    expect(onExpand).toHaveBeenCalledTimes(1);
  });

  it("announces loading, error, degraded, and recovery without moving chrome", () => {
    const { rerender } = render(
      <ArtifactShell
        context="inline"
        focusSupported
        identity={IDENTITY}
        status={{ kind: "loading", message: "Reading README.md" }}
      >
        <p>Hidden while loading</p>
      </ArtifactShell>,
    );

    expect(screen.getByRole("article")).toHaveAttribute("aria-busy", "true");
    expect(screen.getByRole("status")).toHaveTextContent("Loading artifact");
    expect(screen.queryByText("Hidden while loading")).toBeNull();

    rerender(
      <ArtifactShell
        context="inline"
        focusSupported
        identity={IDENTITY}
        status={{ kind: "error", message: "Read failed safely" }}
      >
        <p>Hidden after error</p>
      </ArtifactShell>,
    );
    expect(screen.getByRole("alert")).toHaveTextContent("Read failed safely");

    for (const kind of ["degraded", "recovery"] as const) {
      rerender(
        <ArtifactShell
          context="inline"
          focusSupported
          identity={IDENTITY}
          status={{ kind, message: `${kind} detail` }}
        >
          <p>Bounded content remains visible</p>
        </ArtifactShell>,
      );
      expect(screen.getByRole("status")).toHaveTextContent(`${kind} detail`);
      expect(screen.getByText("Bounded content remains visible")).toBeVisible();
    }
  });

  it("uses Close in focus and prevents unsupported providers from focusing", () => {
    const onClose = vi.fn();
    const { rerender } = render(
      <ArtifactShell
        context="focused"
        focusSupported
        identity={IDENTITY}
        onClose={onClose}
        onExpand={vi.fn()}
      >
        Focused body
      </ArtifactShell>,
    );
    expect(screen.getByRole("article")).toHaveAttribute(
      "data-artifact-context",
      "focused",
    );
    expect(screen.queryByRole("button", { name: "Expand" })).toBeNull();
    fireEvent.click(screen.getByRole("button", { name: "Close" }));
    expect(onClose).toHaveBeenCalledTimes(1);

    rerender(
      <UnknownArtifact
        artifactId="artifact:future"
        context="focused"
        message="Artifact version 9 is not supported."
        requestedType="future-file"
        requestedVersion={9}
        title="Future document"
      />,
    );
    expect(screen.getByRole("article")).toHaveAttribute(
      "data-artifact-context",
      "inline",
    );
    expect(screen.queryByRole("button", { name: "Expand" })).toBeNull();
    expect(screen.queryByRole("button", { name: "Close" })).toBeNull();
    expect(screen.getByRole("status")).toHaveTextContent(
      "Artifact version 9 is not supported.",
    );
  });
});
