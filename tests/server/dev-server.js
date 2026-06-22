import { startMockServer } from "./mock-server.js";

const server = await startMockServer({ port: Number(process.env.MOCK_PORT || 4310) });

console.log(`C4OS mock server available at ${server.origin}`);

const shutdown = async () => {
  await server.close();
  process.exit(0);
};

process.on("SIGINT", shutdown);
process.on("SIGTERM", shutdown);
