import { readFileSync, readdirSync } from "node:fs";
import { extname, join, relative } from "node:path";

import { describe, expect, it } from "vitest";

const SOURCE_ROOT = join(process.cwd(), "src");

function sourceFiles(directory = SOURCE_ROOT): readonly string[] {
  return readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
    const path = join(directory, entry.name);
    if (entry.isDirectory()) return sourceFiles(path);
    if (![".ts", ".tsx"].includes(extname(entry.name))) return [];
    if (entry.name.endsWith(".test.ts") || entry.name.endsWith(".test.tsx")) {
      return [];
    }
    return [path];
  });
}

describe("renderer authority boundary", () => {
  it("keeps generic native invocation behind one allowlisted adapter", () => {
    const imports = sourceFiles()
      .map((path) => ({
        path: relative(SOURCE_ROOT, path),
        source: readFileSync(path, "utf8"),
      }))
      .filter(({ source }) => source.includes('"@tauri-apps/api/core"'));

    expect(imports.map(({ path }) => path)).toEqual([
      "platform/native-transport.ts",
    ]);
    expect(imports[0]?.source).toContain("export type NativeCommand =");
  });

  it("contains no renderer persistence or unsafe HTML authority", () => {
    const violations = sourceFiles().flatMap((path) => {
      const source = readFileSync(path, "utf8");
      const reasons = [
        /\b(?:window\.|globalThis\.)?localStorage\s*(?:\[|\.(?:getItem|setItem|removeItem|clear|key|length))/.test(
          source,
        )
          ? "localStorage access"
          : null,
        /\b(?:window\.|globalThis\.)?sessionStorage\s*(?:\[|\.(?:getItem|setItem|removeItem|clear|key|length))/.test(
          source,
        )
          ? "sessionStorage access"
          : null,
        /dangerouslySetInnerHTML/.test(source) ? "unsafe HTML" : null,
      ].filter((reason): reason is string => reason !== null);
      return reasons.map(
        (reason) => `${relative(SOURCE_ROOT, path)}: ${reason}`,
      );
    });

    expect(violations).toEqual([]);
  });
});
