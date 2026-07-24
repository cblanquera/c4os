import { Buffer } from "node:buffer";
import http from "node:http";
import process from "node:process";
import { URL } from "node:url";

const requestedPort = Number.parseInt(process.argv[2] ?? "43110", 10);
if (
  !Number.isInteger(requestedPort) ||
  requestedPort < 1 ||
  requestedPort > 65_535
) {
  throw new Error("usage: node scripts/task-00010-browser-fixture.mjs [port]");
}

const pages = {
  "/": page(
    "Hostile Browser fixture A",
    `<main>
      <h1>Hostile Browser fixture A</h1>
      <p id="selected">SELECTED-CONTEXT-ONLY</p>
      <p>VISIBLE-CONTEXT-WITHOUT-SECRETS</p>
      <p id="bridge-status">Checking native bridge boundary…</p>
      <p id="storage-status">Storage value: unset</p>
      <nav>
        <a id="page-b" href="/page-b?credential=must-not-project#private-fragment">Open page B</a>
        <a id="redirect" href="/redirect">Follow sanitized redirect</a>
        <a id="popup" href="/popup" target="_blank">Request popup</a>
        <a id="download" download href="/download">Request download</a>
        <a id="binary" href="/binary">Request binary response</a>
      </nav>
      <form action="/posted?form-secret=must-not-project" method="post">
        <input name="password" value="FORM-VALUE-MUST-NOT-CAPTURE" />
        <button id="post" type="submit">Submit POST form</button>
      </form>
      <button id="store" type="button">Store profile marker</button>
      <button id="read" type="button">Read profile marker</button>
      <button id="media" type="button">Request camera</button>
      <script>
        (() => {
          const hasBridge = Boolean(
            window.__TAURI__ ||
              window.__TAURI_INTERNALS__ ||
              window.__C4OS__ ||
              window.webkit?.messageHandlers,
          );
          document.querySelector("#bridge-status").textContent = hasBridge
            ? "FAIL: page-accessible native bridge detected"
            : "PASS: no page-accessible native bridge";
          const showStorage = () => {
            const value = localStorage.getItem("task10-profile-marker") ?? "unset";
            document.querySelector("#storage-status").textContent = "Storage value: " + value;
          };
          document.querySelector("#store").addEventListener("click", () => {
            localStorage.setItem("task10-profile-marker", "CHAT-PERSISTED");
            showStorage();
          });
          document.querySelector("#read").addEventListener("click", showStorage);
          document.querySelector("#media").addEventListener("click", async () => {
            try {
              await navigator.mediaDevices.getUserMedia({ video: true });
              document.querySelector("#bridge-status").textContent = "Camera unexpectedly granted";
            } catch {
              document.querySelector("#bridge-status").textContent = "PASS: camera request denied or unavailable";
            }
          });
          showStorage();
        })();
      </script>
    </main>`,
  ),
  "/page-b": page(
    "Hostile Browser fixture B",
    `<main><h1>Hostile Browser fixture B</h1><p>History destination.</p><a href="/">Return to A</a></main>`,
  ),
  "/popup": page("Popup target", "<main><h1>Popup target</h1></main>"),
};

const server = http.createServer((request, response) => {
  const url = new URL(request.url ?? "/", "http://127.0.0.1");
  if (request.method === "POST") {
    request.resume();
    response.writeHead(405, { "Content-Type": "text/plain; charset=utf-8" });
    response.end(
      "POST reached fixture server and should have been blocked by C4OS",
    );
    return;
  }
  if (url.pathname === "/redirect") {
    response.writeHead(302, {
      Location: "/page-b?redirect-secret=must-not-project#redirect-private",
    });
    response.end();
    return;
  }
  if (url.pathname === "/download") {
    response.writeHead(200, {
      "Content-Disposition": 'attachment; filename="private-download-name.txt"',
      "Content-Type": "text/plain; charset=utf-8",
    });
    response.end("DOWNLOAD-BODY-MUST-NOT-PROJECT");
    return;
  }
  if (url.pathname === "/binary") {
    response.writeHead(200, { "Content-Type": "application/octet-stream" });
    response.end(Buffer.from([0, 1, 2, 3]));
    return;
  }
  const body = pages[url.pathname];
  if (body === undefined) {
    response.writeHead(404, { "Content-Type": "text/plain; charset=utf-8" });
    response.end("Not found");
    return;
  }
  response.writeHead(200, {
    "Cache-Control": "no-store",
    "Content-Security-Policy": "default-src 'self' 'unsafe-inline'",
    "Content-Type": "text/html; charset=utf-8",
    "X-Content-Type-Options": "nosniff",
  });
  response.end(body);
});

server.listen(requestedPort, "127.0.0.1", () => {
  process.stdout.write(
    `${JSON.stringify({ origin: `http://127.0.0.1:${requestedPort}`, pid: process.pid })}\n`,
  );
});

for (const signal of ["SIGINT", "SIGTERM"]) {
  process.on(signal, () => server.close(() => process.exit(0)));
}

function page(title, body) {
  return `<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>${title}</title>
    <style>
      :root { color-scheme: light dark; font: 16px/1.5 system-ui, sans-serif; }
      body { margin: 0; padding: 2rem; }
      main { max-width: 48rem; }
      nav, form { display: grid; gap: .75rem; margin-block: 1.5rem; }
      button, input { font: inherit; padding: .5rem .75rem; }
    </style>
  </head>
  <body>${body}</body>
</html>`;
}
