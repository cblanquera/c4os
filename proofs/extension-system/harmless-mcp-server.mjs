#!/usr/bin/env node
import readline from 'node:readline';

const rl = readline.createInterface({
  input: process.stdin,
  output: process.stdout,
  terminal: false
});

const tools = [
  {
    name: 'echo_safe',
    description: 'Echoes a supplied message without filesystem, network, or shell access.',
    inputSchema: {
      type: 'object',
      properties: {
        message: { type: 'string' }
      },
      required: ['message'],
      additionalProperties: false
    }
  }
];

rl.on('line', (line) => {
  let request;
  try {
    request = JSON.parse(line);
  } catch {
    return;
  }

  if (request.method === 'initialize') {
    respond(request.id, {
      protocolVersion: '2025-03-26',
      serverInfo: { name: 'c4os-harmless-extension-poc', version: '0.0.0' },
      capabilities: { tools: {} }
    });
    return;
  }

  if (request.method === 'tools/list') {
    respond(request.id, { tools });
    return;
  }

  if (request.method === 'tools/call') {
    if (request.params?.name !== 'echo_safe') {
      respondError(request.id, -32602, 'Unknown tool');
      return;
    }
    respond(request.id, {
      content: [
        {
          type: 'text',
          text: `echo:${String(request.params?.arguments?.message || '')}`
        }
      ],
      isError: false
    });
    return;
  }

  respondError(request.id, -32601, 'Method not found');
});

function respond(id, result) {
  process.stdout.write(`${JSON.stringify({ jsonrpc: '2.0', id, result })}\n`);
}

function respondError(id, code, message) {
  process.stdout.write(`${JSON.stringify({ jsonrpc: '2.0', id, error: { code, message } })}\n`);
}
