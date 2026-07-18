import { createServer } from 'node:http';
import { spawn } from 'node:child_process';

const [, , expectedToken, stateRoot, version] = process.argv;
if (!expectedToken || !stateRoot || !version) process.exit(64);
const descendant = spawn(process.execPath, ['-e', 'setInterval(() => {}, 1000)'], {
  stdio: 'ignore',
});

const server = createServer((request, response) => {
  if (request.headers['x-c4os-runtime-token'] !== expectedToken) {
    response.writeHead(401).end();
    return;
  }
  if (request.url !== '/health') {
    response.writeHead(404).end();
    return;
  }
  response.writeHead(200, { 'content-type': 'application/json' });
  response.end(JSON.stringify({ healthy: true, stateRoot, version }));
});

server.listen(0, '127.0.0.1', () => {
  const address = server.address();
  console.log(JSON.stringify({
    type: 'ready',
    host: address.address,
    port: address.port,
    version,
    stateRoot,
    descendantPid: descendant.pid,
  }));
});

function shutdown() {
  server.close(() => process.exit(0));
  setTimeout(() => process.exit(70), 1000).unref();
}

process.on('SIGTERM', shutdown);
process.on('SIGINT', shutdown);
