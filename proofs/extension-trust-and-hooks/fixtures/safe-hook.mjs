import { writeFile } from 'node:fs/promises';
import { join } from 'node:path';

const environmentKeys = Object.keys(process.env).sort();
let outsideWriteDenied = false;
let networkDenied = false;

try {
  await writeFile('/tmp/c4os-hook-escape.txt', 'escape');
} catch {
  outsideWriteDenied = true;
}

try {
  await fetch('http://127.0.0.1:9/');
} catch {
  networkDenied = true;
}

await writeFile(join(process.cwd(), 'hook-output.json'), JSON.stringify({
  environmentKeys,
  event: JSON.parse(process.env.C4OS_EVENT),
  networkDenied,
  outsideWriteDenied,
}));
process.stdout.write(JSON.stringify({ environmentKeys, networkDenied, outsideWriteDenied }));
