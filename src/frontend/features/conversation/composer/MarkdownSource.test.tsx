import { render } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { MarkdownSource } from "./MarkdownSource";

describe("MarkdownSource", () => {
  it("preserves canonical source while assigning light syntax families", () => {
    const source = [
      "# **Build** notes",
      "> Read [the guide](https://example.com)",
      "- Run `npm test`",
    ].join("\n");
    const { container } = render(<MarkdownSource source={source} />);

    expect(container.textContent).toBe(source);
    expect(container.querySelector(".is-heading")).toHaveTextContent(
      "# **Build** notes",
    );
    expect(container.querySelector(".is-quote")).toHaveTextContent(
      "> Read [the guide](https://example.com)",
    );
    expect(container.querySelector(".is-list")).toHaveTextContent(
      "- Run `npm test`",
    );
    expect(container.querySelector(".is-emphasis")).toHaveTextContent(
      "**Build**",
    );
    expect(container.querySelector(".is-link")).toHaveTextContent(
      "[the guide](https://example.com)",
    );
    expect(container.querySelector(".is-code")).toHaveTextContent("`npm test`");
  });
});
