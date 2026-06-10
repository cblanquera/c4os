//node
import { cp, mkdir, rm } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const scriptDir = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(scriptDir, '..');
const distDir = resolve(projectRoot, 'dist');

/**
 * Builds the static frontend bundle used by the Tauri shell.
 */
async function buildStaticFrontend() {
  await rm(distDir, { force: true, recursive: true });
  await mkdir(distDir, { recursive: true });
  await cp(resolve(projectRoot, 'index.html'), resolve(distDir, 'index.html'));
  await cp(resolve(projectRoot, 'src'), resolve(distDir, 'src'), {
    recursive: true,
  });
}

await buildStaticFrontend();
