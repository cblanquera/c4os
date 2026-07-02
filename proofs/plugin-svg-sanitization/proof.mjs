export function sanitizePluginIcon() {
  const safeSvg = '<svg viewBox="0 0 16 16"><path d="M1 1h14v14H1z"/></svg>';
  const unsafeSvg = '<svg><script></script><a href="https://x.test"><path onclick="x()"/></a></svg>';

  return {
    safeIcon: sanitize(safeSvg),
    unsafeIcon: sanitize(unsafeSvg)
  };
}

function sanitize(svg) {
  const blockedReasons = [];
  if (/<script\b/i.test(svg)) blockedReasons.push('script element');
  if (/\son[a-z]+\s*=/i.test(svg)) blockedReasons.push('event handler attribute');
  if (/\s(?:href|xlink:href)=["']https?:/i.test(svg)) blockedReasons.push('external href');

  return blockedReasons.length === 0
    ? { accepted: true, fallback: undefined, blockedReasons }
    : { accepted: false, fallback: 'default-plugin-icon', blockedReasons };
}
