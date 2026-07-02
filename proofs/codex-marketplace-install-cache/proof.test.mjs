import test from 'node:test';
import assert from 'node:assert/strict';
import { runMarketplaceCacheProof } from './proof.mjs';

test('marketplace plugin installs to cache, uninstalls cache copy, and reinstalls from source', () => {
  const result = runMarketplaceCacheProof();

  assert.equal(result.installedCachePath, '/user-cache/c4os/plugins/github/demo-plugin@abc123');
  assert.deepEqual(result.steps, ['metadata-read', 'cache-written', 'cache-removed', 'reinstalled-from-github-ref']);
  assert.equal(result.cacheExistsAfterUninstall, false);
  assert.equal(result.cacheExistsAfterReinstall, true);
});
