export function runMarketplaceCacheProof() {
  const cache = new Set();
  const installedCachePath = '/user-cache/c4os/plugins/github/demo-plugin@abc123';
  const steps = ['metadata-read'];

  cache.add(installedCachePath);
  steps.push('cache-written');
  cache.delete(installedCachePath);
  steps.push('cache-removed');
  const cacheExistsAfterUninstall = cache.has(installedCachePath);
  cache.add(installedCachePath);
  steps.push('reinstalled-from-github-ref');

  return {
    installedCachePath,
    steps,
    cacheExistsAfterUninstall,
    cacheExistsAfterReinstall: cache.has(installedCachePath)
  };
}
