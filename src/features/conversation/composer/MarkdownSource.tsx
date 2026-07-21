import type { ReactNode } from "react";

interface MarkdownSourceProps {
  readonly source: string;
}

const INLINE_SOURCE_PATTERN =
  /(\*\*[^*\n]+\*\*|_[^_\n]+_|\*[^*\n]+\*|`[^`\n]+`|\[[^\]\n]+\]\(https?:\/\/[^)\s]+\)|https?:\/\/[^\s]+)/g;

/** Renders safe, non-interactive source highlighting without hiding Markdown markers. */
export function MarkdownSource({ source }: MarkdownSourceProps) {
  const lines = source.split("\n");

  return (
    <>
      {lines.map((line, index) => (
        <span className={lineClassName(line)} key={`${index}-${line}`}>
          {highlightLine(line)}
          {index < lines.length - 1 ? "\n" : null}
        </span>
      ))}
    </>
  );
}

/** Preserves every source character while assigning light inline token classes. */
function highlightLine(line: string): readonly ReactNode[] {
  const parts: ReactNode[] = [];
  let sourceIndex = 0;

  for (const match of line.matchAll(INLINE_SOURCE_PATTERN)) {
    const matchIndex = match.index;
    const token = match[0];

    if (matchIndex > sourceIndex) {
      parts.push(line.slice(sourceIndex, matchIndex));
    }
    parts.push(
      <span className={tokenClassName(token)} key={`${matchIndex}-${token}`}>
        {token}
      </span>,
    );
    sourceIndex = matchIndex + token.length;
  }

  if (sourceIndex < line.length) {
    parts.push(line.slice(sourceIndex));
  }

  return parts;
}

/** Identifies line-level Markdown prefixes while leaving their source visible. */
function lineClassName(line: string): string {
  if (/^\s{0,3}#{1,6}\s/.test(line)) {
    return "conversation-composer__source-line is-heading";
  }
  if (/^\s{0,3}>\s?/.test(line)) {
    return "conversation-composer__source-line is-quote";
  }
  if (/^\s*(?:[-+*]|\d+\.)\s/.test(line)) {
    return "conversation-composer__source-line is-list";
  }
  return "conversation-composer__source-line";
}

/** Assigns the visual token family used by the source-preserving overlay. */
function tokenClassName(token: string): string {
  if (token.startsWith("`")) {
    return "conversation-composer__source-token is-code";
  }
  if (token.startsWith("[") || token.startsWith("http")) {
    return "conversation-composer__source-token is-link";
  }
  return "conversation-composer__source-token is-emphasis";
}
