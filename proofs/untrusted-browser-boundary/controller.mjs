import { isAbsolute, relative, resolve } from 'node:path';

const allowedSchemes = new Set(['http:', 'https:']);
const allowedPermissions = new Set(['clipboard-read', 'camera', 'microphone', 'geolocation']);

export class UntrustedBrowserBoundary {
  constructor({ downloadRoot }) {
    this.downloadRoot = resolve(downloadRoot);
    this.generation = 1;
    this.profiles = new Map();
    this.grants = new Map();
    this.events = [];
    this.crashed = false;
  }

  createProfile(id, { persistent = false } = {}) {
    if (this.profiles.has(id)) throw new Error('Profile already exists');
    this.profiles.set(id, { id, persistent, cookies: new Map() });
  }

  setCookie(profileId, name, value) {
    const profile = this.profiles.get(profileId);
    if (!profile) throw new Error('Unknown profile');
    profile.cookies.set(name, value);
  }

  getCookie(profileId, name) {
    return this.profiles.get(profileId)?.cookies.get(name);
  }

  navigate(profileId, url) {
    if (!this.profiles.has(profileId)) throw new Error('Unknown profile');
    const parsed = new URL(url);
    if (!allowedSchemes.has(parsed.protocol)) {
      return this.record('navigation.blocked', { profileId, url, reason: 'blocked-scheme' });
    }
    return this.record('navigation.allowed', { profileId, url: parsed.href });
  }

  requestPopup(profileId, url) {
    return this.record('popup.blocked', { profileId, url, reason: 'new-window-requires-host-mediation' });
  }

  grantOnce({ profileId, origin, permission, userApproved }) {
    if (!this.profiles.has(profileId)) throw new Error('Unknown profile');
    if (!allowedPermissions.has(permission)) throw new Error('Unsupported permission');
    if (!userApproved) return false;
    const key = `${this.generation}:${profileId}:${origin}:${permission}`;
    this.grants.set(key, 1);
    return true;
  }

  consumePermission({ profileId, origin, permission }) {
    const key = `${this.generation}:${profileId}:${origin}:${permission}`;
    const remaining = this.grants.get(key) ?? 0;
    if (remaining < 1) return false;
    this.grants.delete(key);
    return true;
  }

  authorizeDownload({ profileId, sourceUrl, filename, policyDecision }) {
    if (!this.profiles.has(profileId)) throw new Error('Unknown profile');
    if (policyDecision !== 'allow') return { allowed: false, reason: 'policy' };
    const target = resolve(this.downloadRoot, filename);
    const rel = relative(this.downloadRoot, target);
    if (rel.startsWith('..') || isAbsolute(rel)) return { allowed: false, reason: 'outside-download-root' };
    const authorization = {
      allowed: true, generation: this.generation, profileId, sourceUrl,
      target, consumed: false,
    };
    return authorization;
  }

  consumeDownload(authorization, attempt) {
    if (!authorization.allowed || authorization.consumed) return false;
    if (authorization.generation !== this.generation) return false;
    for (const field of ['profileId', 'sourceUrl', 'target']) {
      if (authorization[field] !== attempt[field]) return false;
    }
    authorization.consumed = true;
    return true;
  }

  crash() {
    this.crashed = true;
    this.generation += 1;
    this.grants.clear();
    for (const profile of this.profiles.values()) {
      if (!profile.persistent) profile.cookies.clear();
    }
    return this.record('browser.crashed', { generation: this.generation });
  }

  recover() {
    if (!this.crashed) throw new Error('Browser has not crashed');
    this.crashed = false;
    return this.record('browser.recovered', { generation: this.generation });
  }

  ingest(event, generation) {
    if (generation !== this.generation) return { accepted: false, reason: 'stale-generation' };
    if (this.crashed) return { accepted: false, reason: 'crashed' };
    return { accepted: true, event: this.record(event.type, event) };
  }

  record(type, detail) {
    const event = { type, generation: this.generation, ...detail };
    this.events.push(event);
    return event;
  }
}
