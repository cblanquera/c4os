import { readdir, readFile } from "node:fs/promises";
import { resolve } from "node:path";
import process from "node:process";

const mode = process.argv[2];
if (!["production", "fixture"].includes(mode)) {
  throw new Error("usage: check-qa-bundle-boundary.mjs <production|fixture>");
}

const assetsRoot = resolve("dist/assets");
const assetNames = (await readdir(assetsRoot)).sort();
const assets = assetNames.filter(
  (name) => name.endsWith(".js") || name.endsWith(".map"),
);
const source = (
  await Promise.all(
    assets.map((name) => readFile(resolve(assetsRoot, name), "utf8")),
  )
).join("\n");

const markers = [
  "qa-fixture-only",
  "Deterministic QA fixture data",
  "VITE_C4OS_QA_FIXTURES",
];

if (mode === "production") {
  const sourceMaps = assetNames.filter((name) => name.endsWith(".map"));
  if (sourceMaps.length > 0) {
    throw new Error(
      `production bundle contains source maps: ${sourceMaps.join(", ")}`,
    );
  }
  const leaked = markers.filter((marker) => source.includes(marker));
  if (leaked.length > 0) {
    throw new Error(
      `production bundle contains QA authority: ${leaked.join(", ")}`,
    );
  }
  process.stdout.write(
    "production bundle excludes deterministic QA authority\n",
  );
} else {
  const missing = markers
    .slice(0, 2)
    .filter((marker) => !source.includes(marker));
  if (missing.length > 0) {
    throw new Error(
      `QA bundle is missing fixture authority: ${missing.join(", ")}`,
    );
  }
  process.stdout.write(
    "QA bundle contains explicit deterministic fixture authority\n",
  );
}
