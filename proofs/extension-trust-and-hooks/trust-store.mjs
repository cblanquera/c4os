import { createHash, verify } from 'node:crypto';

/**
 * Creates the canonical bytes signed by a marketplace authority.
 */
export function canonicalRelease(release) {
  return Buffer.from(JSON.stringify({
    digest: release.digest,
    id: release.id,
    origin: release.origin,
    version: release.version,
  }));
}

/**
 * Calculates a stable digest for the proof bundle payload.
 */
export function digestPayload(payload) {
  return createHash('sha256').update(payload).digest('hex');
}

/**
 * Models C4OS-owned marketplace origin, signature, quarantine, and revocation.
 */
export class ExtensionTrustStore {
  constructor(authorities) {
    this.authorities = new Map(authorities.map((authority) => [authority.id, authority]));
    this.installed = new Map();
    this.revokedDigests = new Set();
    this.revokedKeys = new Set();
  }

  /**
   * Installs a verified release disabled after checking every trust boundary.
   */
  install(release, payload) {
    const authority = this.authorities.get(release.keyId);
    if (!authority || authority.origin !== release.origin) {
      throw new Error('Untrusted marketplace origin or signing authority');
    }
    if (this.revokedKeys.has(release.keyId) || this.revokedDigests.has(release.digest)) {
      throw new Error('Release or signing authority is revoked');
    }
    if (digestPayload(payload) !== release.digest) throw new Error('Release digest mismatch');
    if (!verify(null, canonicalRelease(release), authority.publicKey, release.signature)) {
      throw new Error('Release signature verification failed');
    }
    const installed = {
      ...release,
      enabled: false,
      quarantined: true,
      payload,
    };
    this.installed.set(release.id, installed);
    return installed;
  }

  /**
   * Enables a verified release only after an explicit review decision.
   */
  enable(id, reviewed) {
    const installed = this.installed.get(id);
    if (!installed) throw new Error('Extension is not installed');
    if (!reviewed) throw new Error('Explicit review is required');
    if (this.revokedKeys.has(installed.keyId) || this.revokedDigests.has(installed.digest)) {
      throw new Error('Extension is revoked');
    }
    installed.enabled = true;
    installed.quarantined = false;
    return installed;
  }

  /**
   * Revokes a content digest and immediately disables matching installations.
   */
  revokeDigest(digest) {
    this.revokedDigests.add(digest);
    for (const installed of this.installed.values()) {
      if (installed.digest === digest) installed.enabled = false;
    }
  }

  /**
   * Revokes an authority and immediately disables everything it signed.
   */
  revokeKey(keyId) {
    this.revokedKeys.add(keyId);
    for (const installed of this.installed.values()) {
      if (installed.keyId === keyId) installed.enabled = false;
    }
  }
}
