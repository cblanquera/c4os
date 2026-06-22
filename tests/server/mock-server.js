import { createServer } from "node:http";
import { createMockWorkspace } from "./mock-data.js";

function wait(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

function readJson(request) {
  return new Promise((resolve, reject) => {
    let body = "";
    request.setEncoding("utf8");
    request.on("data", (chunk) => body += chunk);
    request.on("end", () => {
      if (!body) {
        resolve({});
        return;
      }
      try {
        resolve(JSON.parse(body));
      } catch (error) {
        reject(error);
      }
    });
    request.on("error", reject);
  });
}

function send(response, status, payload) {
  response.writeHead(status, {
    "access-control-allow-headers": "content-type",
    "access-control-allow-methods": "GET,POST,OPTIONS",
    "access-control-allow-origin": "*",
    "content-type": "application/json; charset=utf-8"
  });
  response.end(JSON.stringify(payload));
}

export async function startMockServer(options = {}) {
  const port = options.port || 0;
  const server = createServer(async (request, response) => {
    const url = new URL(request.url || "/", "http://127.0.0.1");
    const scenario = url.searchParams.get("scenario") || "success";
    const delay = Number(url.searchParams.get("delay") || "0");

    if (request.method === "OPTIONS") {
      send(response, 204, {});
      return;
    }

    try {
      if (delay > 0) await wait(delay);

      if (request.method === "GET" && url.pathname === "/api/workspace") {
        if (scenario === "boot-failure") {
          send(response, 503, { error: "Mock boot failure" });
          return;
        }
        send(response, 200, createMockWorkspace());
        return;
      }

      if (request.method === "POST" && url.pathname === "/api/runs") {
        const body = await readJson(request);
        if (scenario === "run-failure") {
          send(response, 500, {
            error: "Mock agent failed before producing output.",
            prompt: body.prompt || ""
          });
          return;
        }
        send(response, 200, {
          prompt: body.prompt || "",
          run: "Mock agent completed the requested transition.",
          agent: "Mock agent completed the requested transition."
        });
        return;
      }

      send(response, 404, { error: `Unknown mock route: ${request.method} ${url.pathname}` });
    } catch (error) {
      send(response, 500, { error: error.message });
    }
  });

  await new Promise((resolveListen, rejectListen) => {
    server.once("error", rejectListen);
    server.listen(port, "127.0.0.1", resolveListen);
  });

  const address = server.address();
  return {
    origin: `http://127.0.0.1:${address.port}`,
    close: () => new Promise((resolveClose, rejectClose) => {
      server.close((error) => error ? rejectClose(error) : resolveClose());
    })
  };
}
