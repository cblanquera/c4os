import { Fragment, type ReactNode } from "react";

export interface SafeMarkdownProps {
  readonly className?: string;
  readonly onLinkActivate?: (href: string) => void;
  readonly source: string;
}

const INLINE_TOKEN =
  /(`[^`\n]+`|\[[^\]\n]+\]\([^\s)\n]+\)|\*\*[^*\n]+\*\*|__[^_\n]+__|\*[^*\n]+\*|https?:\/\/[^\s<]+)/g;

/** Accepts only ordinary web URLs and never grants navigation on its own. */
function safeWebUrl(value: string): string | null {
  try {
    const url = new URL(value);
    return url.protocol === "http:" || url.protocol === "https:"
      ? url.href
      : null;
  } catch {
    return null;
  }
}

/** Renders a compact, source-derived inline subset without injecting HTML. */
function renderInline(
  source: string,
  onLinkActivate: SafeMarkdownProps["onLinkActivate"],
  keyPrefix: string,
): ReactNode[] {
  const nodes: ReactNode[] = [];
  let cursor = 0;

  for (const [index, match] of Array.from(
    source.matchAll(INLINE_TOKEN),
  ).entries()) {
    const token = match[0];
    const start = match.index;
    if (start > cursor) nodes.push(source.slice(cursor, start));
    const key = `${keyPrefix}-${index}`;

    if (token.startsWith("`")) {
      nodes.push(<code key={key}>{token.slice(1, -1)}</code>);
    } else if (token.startsWith("[")) {
      const linkMatch = /^\[([^\]]+)\]\(([^)]+)\)$/.exec(token);
      const href = linkMatch?.[2] ? safeWebUrl(linkMatch[2]) : null;
      nodes.push(
        href && linkMatch?.[1] ? (
          <a
            key={key}
            href={href}
            rel="noreferrer noopener"
            onClick={(event) => {
              event.preventDefault();
              onLinkActivate?.(href);
            }}
          >
            {linkMatch[1]}
          </a>
        ) : (
          <Fragment key={key}>{token}</Fragment>
        ),
      );
    } else if (token.startsWith("**") || token.startsWith("__")) {
      nodes.push(
        <strong key={key}>
          {renderInline(token.slice(2, -2), onLinkActivate, `${key}-strong`)}
        </strong>,
      );
    } else if (token.startsWith("*")) {
      nodes.push(
        <em key={key}>
          {renderInline(token.slice(1, -1), onLinkActivate, `${key}-em`)}
        </em>,
      );
    } else {
      const punctuation = /[),.;:!?]+$/.exec(token)?.[0] ?? "";
      const candidate = token.slice(0, token.length - punctuation.length);
      const href = safeWebUrl(candidate);
      nodes.push(
        href ? (
          <Fragment key={key}>
            <a
              href={href}
              rel="noreferrer noopener"
              onClick={(event) => {
                event.preventDefault();
                onLinkActivate?.(href);
              }}
            >
              {candidate}
            </a>
            {punctuation}
          </Fragment>
        ) : (
          <Fragment key={key}>{token}</Fragment>
        ),
      );
    }

    cursor = start + token.length;
  }

  if (cursor < source.length) nodes.push(source.slice(cursor));
  return nodes;
}

/** Renders submitted Markdown as safe React text and a narrow semantic subset. */
export function SafeMarkdown({
  className,
  onLinkActivate,
  source,
}: SafeMarkdownProps) {
  const lines = source.split("\n");
  const blocks: ReactNode[] = [];
  let lineIndex = 0;

  while (lineIndex < lines.length) {
    const line = lines[lineIndex] ?? "";
    const key = `markdown-${lineIndex}`;

    if (line.startsWith("```")) {
      const code: string[] = [];
      lineIndex += 1;
      while (lineIndex < lines.length && !lines[lineIndex]?.startsWith("```")) {
        code.push(lines[lineIndex] ?? "");
        lineIndex += 1;
      }
      if (lineIndex < lines.length) lineIndex += 1;
      blocks.push(
        <pre key={key}>
          <code>{code.join("\n")}</code>
        </pre>,
      );
      continue;
    }

    if (line.length === 0) {
      lineIndex += 1;
      continue;
    }

    const heading = /^(#{1,6})\s+(.+)$/.exec(line);
    if (heading?.[1] && heading[2]) {
      const content = renderInline(heading[2], onLinkActivate, key);
      const level = heading[1].length;
      blocks.push(
        level === 1 ? (
          <h1 key={key}>{content}</h1>
        ) : level === 2 ? (
          <h2 key={key}>{content}</h2>
        ) : level === 3 ? (
          <h3 key={key}>{content}</h3>
        ) : level === 4 ? (
          <h4 key={key}>{content}</h4>
        ) : level === 5 ? (
          <h5 key={key}>{content}</h5>
        ) : (
          <h6 key={key}>{content}</h6>
        ),
      );
      lineIndex += 1;
      continue;
    }

    if (line.startsWith("> ")) {
      blocks.push(
        <blockquote key={key}>
          {renderInline(line.slice(2), onLinkActivate, key)}
        </blockquote>,
      );
      lineIndex += 1;
      continue;
    }

    const unordered = /^\s*[-+*]\s+(.+)$/.exec(line);
    if (unordered?.[1]) {
      blocks.push(
        <ul key={key}>
          <li>{renderInline(unordered[1], onLinkActivate, key)}</li>
        </ul>,
      );
      lineIndex += 1;
      continue;
    }

    blocks.push(<p key={key}>{renderInline(line, onLinkActivate, key)}</p>);
    lineIndex += 1;
  }

  return (
    <div
      className={["conversation-markdown", className].filter(Boolean).join(" ")}
      data-safe-markdown="react-text"
    >
      {blocks}
    </div>
  );
}
