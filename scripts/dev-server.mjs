//node
import { createReadStream } from 'node:fs';
import { stat } from 'node:fs/promises';
import { createServer } from 'node:http';
import { dirname, extname, normalize, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const host = '127.0.0.1';
const port = Number.parseInt(process.env.PORT ?? '5173', 10);
const scriptDir = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(scriptDir, '..');

const contentTypes = new Map([
  ['.css', 'text/css; charset=utf-8'],
  ['.html', 'text/html; charset=utf-8'],
  ['.js', 'text/javascript; charset=utf-8'],
  ['.json', 'application/json; charset=utf-8'],
]);

/**
 * Resolves a request URL to a file under the project root.
 */
export function resolveRequestPath(url) {
  const rawPath = url.split('?')[0];
  const decodedPath = decodeURIComponent(rawPath);

  if (decodedPath.split('/').includes('..')) {
    return null;
  }

  const requestedPath = new URL(url, `http://${host}:${port}`).pathname;
  const relativePath = requestedPath === '/' ? '/index.html' : requestedPath;
  const absolutePath = resolve(projectRoot, `.${normalize(relativePath)}`);

  if (!absolutePath.startsWith(projectRoot)) {
    return null;
  }

  return absolutePath;
}

/**
 * Serves the static scaffold for local Tauri development.
 */
export async function serveRequest(request, response) {
  const filePath = resolveRequestPath(request.url);

  if (!filePath) {
    response.writeHead(403);
    response.end('Forbidden');
    return;
  }

  try {
    const fileStat = await stat(filePath);

    if (!fileStat.isFile()) {
      response.writeHead(404);
      response.end('Not found');
      return;
    }

    response.writeHead(200, {
      'Content-Type':
        contentTypes.get(extname(filePath)) ?? 'application/octet-stream',
    });
    createReadStream(filePath).pipe(response);
  } catch {
    response.writeHead(404);
    response.end('Not found');
  }
}

export function startDevServer() {
  return createServer(serveRequest).listen(port, host, () => {
    console.log(`C4OS dev server running at http://${host}:${port}`);
  });
}

if (import.meta.url === pathToFileURL(process.argv[1]).href) {
  startDevServer();
}
