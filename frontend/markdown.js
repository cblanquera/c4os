import { h } from "./dom.js";

export function renderMarkdown(text) {
  const blocks = String(text || "").split(/\n{2,}/).map((block) => block.trim()).filter(Boolean);
  if (blocks.length === 0) return [h("p", { text: "" })];
  return blocks.flatMap(renderBlock);
}

function renderBlock(block) {
  if (/^[-*]\s+/m.test(block)) {
    return [h("ul", {}, block.split("\n").filter(Boolean).map((line) => {
      return h("li", {}, renderInline(line.replace(/^[-*]\s+/, "")));
    }))];
  }

  if (/^\d+\.\s+/m.test(block)) {
    return [h("ol", {}, block.split("\n").filter(Boolean).map((line) => {
      return h("li", {}, renderInline(line.replace(/^\d+\.\s+/, "")));
    }))];
  }

  if (block.startsWith("### ")) return [h("h3", {}, renderInline(block.slice(4)))];
  if (block.startsWith("## ")) return [h("h2", {}, renderInline(block.slice(3)))];
  if (block.startsWith("# ")) return [h("h2", {}, renderInline(block.slice(2)))];

  return [h("p", {}, renderInline(block.replace(/\n/g, " ")))];
}

function renderInline(text) {
  const nodes = [];
  const pattern = /(\*\*[^*]+\*\*|`[^`]+`|\[[^\]]+\]\([^)]+\))/g;
  let cursor = 0;
  for (const match of text.matchAll(pattern)) {
    if (match.index > cursor) nodes.push(text.slice(cursor, match.index));
    nodes.push(renderInlineToken(match[0]));
    cursor = match.index + match[0].length;
  }
  if (cursor < text.length) nodes.push(text.slice(cursor));
  return nodes;
}

function renderInlineToken(token) {
  if (token.startsWith("**")) return h("strong", {}, [token.slice(2, -2)]);
  if (token.startsWith("`")) return h("code", {}, [token.slice(1, -1)]);
  const link = token.match(/^\[([^\]]+)\]\(([^)]+)\)$/);
  if (link) return h("a", { href: link[2], target: "_blank", rel: "noreferrer" }, [link[1]]);
  return token;
}
