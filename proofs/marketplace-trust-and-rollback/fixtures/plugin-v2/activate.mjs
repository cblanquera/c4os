import { writeFile } from 'node:fs/promises';
await writeFile(new URL('../ACTIVATED', import.meta.url), 'unexpected');
