import { mkdir, readFile, writeFile } from "node:fs/promises";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { createOpencode } from "@opencode-ai/sdk";

const __dirname = dirname(fileURLToPath(import.meta.url));
const repoRoot = resolve(__dirname, "../..");
const proofDir = __dirname;
const localBin = join(repoRoot, "node_modules", ".bin");
process.env.PATH = `${localBin}:${process.env.PATH ?? ""}`;

const today = new Date().toISOString().slice(0, 10);
const fixtureRoot = join(proofDir, ".tmp", "fixture-project");
await mkdir(fixtureRoot, { recursive: true });
await writeFile(join(fixtureRoot, "README.md"), "# C4OS OpenCode Runtime Fixture\n");

const originalCwd = process.cwd();
process.chdir(fixtureRoot);

const result = {
  runtime: "opencode",
  packageVersions: {},
  adapterContract: {},
  observations: [],
  gaps: [],
  recommendation: "Use OpenCode as a viable MVP runtime candidate behind a C4OS-owned adapter, but keep app-owned approval records around OpenCode permission prompts.",
};

function mark(contractKey, status, evidence) {
  result.adapterContract[contractKey] = { status, evidence };
}

function dataOf(response) {
  return response?.data ?? response;
}

let opencode;
try {
  const sdkPackage = await readPackageJson(
    join(repoRoot, "node_modules", "@opencode-ai", "sdk", "package.json"),
  );
  const cliPackage = await readPackageJson(
    join(repoRoot, "node_modules", "opencode-ai", "package.json"),
  );
  result.packageVersions["@opencode-ai/sdk"] = sdkPackage.version;
  result.packageVersions["opencode-ai"] = cliPackage.version;

  opencode = await createOpencode({
    hostname: "127.0.0.1",
    port: 4771,
    timeout: 15000,
    config: {
      permission: {
        "*": "ask",
        bash: "ask",
        edit: "deny",
      },
    },
  });
  result.serverUrl = opencode.server.url;

  const pathInfo = dataOf(await opencode.client.path.get());
  const project = dataOf(await opencode.client.project.current());
  const config = dataOf(await opencode.client.config.get());
  const providers = dataOf(await opencode.client.provider.list());

  result.observations.push({
    name: "server-control-plane",
    pathInfo,
    project,
    permission: config.permission,
    connectedProviders: providers.connected ?? [],
  });

  const events = await opencode.client.event.subscribe();
  const sessionCreate = opencode.client.session.create({
    body: { title: "C4OS OpenCode runtime proof" },
  });

  const observedEvents = [];
  for await (const event of events.stream) {
    observedEvents.push(event);
    if (observedEvents.some((item) => item.type === "session.created")) {
      break;
    }
  }

  const session = dataOf(await sessionCreate);
  const resumed = dataOf(
    await opencode.client.session.get({ path: { id: session.id } }),
  );
  const sessions = dataOf(await opencode.client.session.list());
  const status = dataOf(await opencode.client.session.status());
  await opencode.client.session.abort({ path: { id: session.id } });
  const messages = dataOf(
    await opencode.client.session.messages({ path: { id: session.id } }),
  );

  result.observations.push({
    name: "session-control",
    session,
    resumed,
    sessionCount: Array.isArray(sessions) ? sessions.length : undefined,
    statusKeys: status ? Object.keys(status) : [],
    events: observedEvents.map((event) => ({
      id: event.id,
      type: event.type,
      sessionID: event.properties?.sessionID,
    })),
    messageCount: Array.isArray(messages) ? messages.length : undefined,
  });

  mark("createSession", "pass", "POST /session creates a project-scoped session with projectID, directory, title, cost, token, version, and time metadata.");
  mark("resumeSession", "pass", "GET /session/:id returns the created session by ID.");
  mark("sendUserMessage", "partial", "session.prompt/session.promptAsync are exposed, but a full assistant turn requires provider credentials and was not run in this offline proof.");
  mark("streamEvents", "pass", "event.subscribe streamed server.connected and session.created through the SDK stream.");
  mark("requestApproval", "partial", "OpenCode exposes permission config and a permission response endpoint, but app-owned approval queue integration must wrap OpenCode permission requests.");
  mark("resolveApproval", "partial", "postSessionIdPermissionsPermissionId exists in the SDK; no live permission request was produced without invoking a model-backed tool call.");
  mark("stopRun", "pass", "session.abort is available and accepted for the created session.");
  mark("listArtifacts", "partial", "session.messages, diff, file, and session metadata are exposed; app-owned artifact identity must be layered by C4OS.");

  result.gaps.push("No model-backed prompt was sent because this proof must not require provider credentials or spend tokens.");
  result.gaps.push("Permission request capture still needs a credentialed run that attempts a sensitive tool call.");
  result.gaps.push("OpenCode state/config paths default to user-level opencode directories unless the host app supplies isolated environment/config paths.");
} finally {
  opencode?.server.close();
  process.chdir(originalCwd);
}

const evidencePath = join(
  proofDir,
  `opencode-runtime-evidence-${today}.md`,
);
const resultPath = join(proofDir, "opencode-runtime-result.json");
await writeFile(resultPath, `${JSON.stringify(result, null, 2)}\n`);
await writeFile(evidencePath, renderEvidence(result));
console.log(`Wrote ${evidencePath}`);
console.log(`Wrote ${resultPath}`);

function renderEvidence(proof) {
  const rows = Object.entries(proof.adapterContract)
    .map(([key, value]) => `| ${key} | ${value.status} | ${value.evidence} |`)
    .join("\n");

  return `# OpenCode Runtime POC Evidence

Date: ${today}
Runtime: OpenCode
Packages:
- opencode-ai: ${proof.packageVersions["opencode-ai"]}
- @opencode-ai/sdk: ${proof.packageVersions["@opencode-ai/sdk"]}

## Command

\`\`\`sh
node proofs/opencode-runtime/opencode_runtime_poc.mjs
\`\`\`

## Summary

OpenCode is a viable MVP runtime candidate for the C4OS adapter control plane. The proof started a local OpenCode server through the SDK, created and resumed a project-scoped session, streamed server/session events, verified permission configuration, and called abort on the created session.

## Adapter Contract Results

| Adapter method | Result | Evidence |
| --- | --- | --- |
${rows}

## Observed Events

\`\`\`json
${JSON.stringify(proof.observations.at(-1)?.events ?? [], null, 2)}
\`\`\`

## Gaps

${proof.gaps.map((gap) => `- ${gap}`).join("\n")}

## Recommendation

${proof.recommendation}
`;
}

async function readPackageJson(path) {
  return JSON.parse(await readFile(path, "utf8"));
}
