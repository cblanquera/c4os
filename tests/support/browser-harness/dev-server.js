import { startFrontendServer } from "./server.js";

const server = await startFrontendServer({ port: Number(process.env.PORT || 3000) });

console.log(`C4OS frontend available at ${server.origin}`);

const shutdown = async () => {
  await server.close();
  process.exit(0);
};

process.on("SIGINT", shutdown);
process.on("SIGTERM", shutdown);
