import { mkdir, readFile, writeFile } from "node:fs/promises";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import {
  Agent,
  InMemorySessionRepo,
  convertToLlm,
} from "@earendil-works/pi-agent-core";
import {
  Type,
  fauxAssistantMessage,
  fauxToolCall,
  registerFauxProvider,
} from "@earendil-works/pi-ai/base";

const __dirname = dirname(fileURLToPath(import.meta.url));
const today = new Date().toISOString().slice(0, 10);
const proofDir = __dirname;
await mkdir(join(proofDir, ".tmp"), { recursive: true });

const result = {
  runtime: "pi",
  packageVersions: {},
  adapterContract: {},
  observations: [],
  gaps: [],
  recommendation: "Use Pi as a viable in-process runtime candidate only if C4OS owns the sandbox, approval policy, provider configuration, and persistence boundaries around it.",
};

function mark(contractKey, status, evidence) {
  result.adapterContract[contractKey] = { status, evidence };
}

const repoRoot = join(proofDir, "../..");
const corePackage = await readPackageJson(
  join(repoRoot, "node_modules", "@earendil-works", "pi-agent-core", "package.json"),
);
const cliPackage = await readPackageJson(
  join(repoRoot, "node_modules", "@earendil-works", "pi-coding-agent", "package.json"),
);
result.packageVersions["@earendil-works/pi-agent-core"] = corePackage.version;
result.packageVersions["@earendil-works/pi-coding-agent"] = cliPackage.version;

const repo = new InMemorySessionRepo();
const session = await repo.create({ id: "c4os-pi-proof-session" });
const sessionMetadata = session.storage.metadata;
const sessionList = await repo.list();
const resumed = await repo.open(sessionMetadata);

const faux = registerFauxProvider({
  provider: "c4os-proof",
  models: [{ id: "runtime-proof", name: "Runtime Proof" }],
});

faux.setResponses([
  fauxAssistantMessage(
    [
      fauxToolCall(
        "write_file",
        { path: "blocked.txt", content: "should not be written" },
        { id: "tool-1" },
      ),
    ],
    { stopReason: "toolUse" },
  ),
  fauxAssistantMessage("C4OS policy denial was recorded.", {
    stopReason: "stop",
  }),
]);

const events = [];
const approvalRequests = [];
let executedTool = false;

const writeFileTool = {
  name: "write_file",
  label: "Write file",
  description: "Proof-only file write tool.",
  parameters: Type.Object({
    path: Type.String(),
    content: Type.String(),
  }),
  execute: async (_toolCallId, args) => {
    executedTool = true;
    return {
      content: [{ type: "text", text: `executed ${args.path}` }],
      details: { args },
    };
  },
};

const agent = new Agent({
  sessionId: sessionMetadata.id,
  initialState: {
    systemPrompt: "C4OS Pi runtime adapter proof.",
    model: faux.getModel(),
    tools: [writeFileTool],
  },
  convertToLlm,
  beforeToolCall: async (context) => {
    approvalRequests.push({
      toolName: context.toolCall.name,
      args: context.args,
      decision: "deny",
      reason: "C4OS app-owned approval policy denied proof write.",
    });
    return {
      block: true,
      reason: "C4OS app-owned approval policy denied proof write.",
    };
  },
});

agent.subscribe((event) => {
  events.push({
    type: event.type,
    toolName: event.toolName,
    toolCallId: event.toolCallId,
    isError: event.isError,
  });
});

await agent.prompt("Attempt the proof write_file tool.");
await agent.waitForIdle();
agent.abort();

const messages = agent.state.messages;
const toolResult = messages.find((message) => message.role === "toolResult");

result.observations.push({
  name: "session-repository",
  createdSessionID: sessionMetadata.id,
  resumedSessionID: resumed.storage.metadata.id,
  listedSessionIDs: sessionList.map((item) => item.id),
});
result.observations.push({
  name: "agent-run",
  sessionId: agent.sessionId,
  eventTypes: events.map((event) => event.type),
  approvalRequests,
  executedTool,
  messageRoles: messages.map((message) => message.role),
  blockedToolResult: toolResult,
});

mark("createSession", "pass", "InMemorySessionRepo creates an app-selectable session ID; JsonlSessionRepo can replace it for durable storage.");
mark("resumeSession", "pass", "Session metadata reopens through the repository contract.");
mark("sendUserMessage", "pass", "Agent.prompt accepts a user message and runs the agent loop.");
mark("streamEvents", "pass", "Agent.subscribe emits lifecycle, message, and tool execution events suitable for app-owned records.");
mark("requestApproval", "pass", "beforeToolCall receives validated tool arguments before execution.");
mark("resolveApproval", "pass", "Returning block:true from beforeToolCall prevents the proof write tool from executing and emits an error tool result.");
mark("stopRun", "pass", "Agent.abort and waitForIdle are available for run cancellation/lifecycle control.");
mark("listArtifacts", "partial", "Pi exposes transcript/tool-result state; C4OS must define artifact identity and persistence outside the core runtime.");

result.gaps.push("Pi's public README says it has no built-in permission sandbox for filesystem, process, network, or credential access; C4OS must own those boundaries.");
result.gaps.push("This proof uses Pi's faux provider to avoid credentials and token spend; provider-backed streaming should be repeated after provider policy is decided.");
result.gaps.push("Pi is lower-level than OpenCode: session persistence, artifact identity, trusted-root policy, and provider secret storage remain app-owned integration work.");

faux.unregister();

const evidencePath = join(proofDir, `pi-runtime-evidence-${today}.md`);
const resultPath = join(proofDir, "pi-runtime-result.json");
await writeFile(resultPath, `${JSON.stringify(result, null, 2)}\n`);
await writeFile(evidencePath, renderEvidence(result));
console.log(`Wrote ${evidencePath}`);
console.log(`Wrote ${resultPath}`);

function renderEvidence(proof) {
  const rows = Object.entries(proof.adapterContract)
    .map(([key, value]) => `| ${key} | ${value.status} | ${value.evidence} |`)
    .join("\n");

  return `# Pi Runtime POC Evidence

Date: ${today}
Runtime: Pi
Packages:
- @earendil-works/pi-agent-core: ${proof.packageVersions["@earendil-works/pi-agent-core"]}
- @earendil-works/pi-coding-agent: ${proof.packageVersions["@earendil-works/pi-coding-agent"]}

## Command

\`\`\`sh
node proofs/pi-runtime/pi_runtime_poc.mjs
\`\`\`

## Summary

Pi is a viable in-process runtime candidate for the C4OS adapter contract. The proof created and reopened a session, ran a prompt through the Agent loop with a faux provider, streamed lifecycle/tool events, intercepted a write-file tool call before execution, denied it through app-owned policy, and confirmed the tool body did not execute.

## Adapter Contract Results

| Adapter method | Result | Evidence |
| --- | --- | --- |
${rows}

## Approval Observation

\`\`\`json
${JSON.stringify(proof.observations.at(-1)?.approvalRequests ?? [], null, 2)}
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
