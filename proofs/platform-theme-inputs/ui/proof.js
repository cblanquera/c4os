import { fallbackTokens, tokenChecks } from './tokens.mjs';

const queries = {
  'color scheme': '(prefers-color-scheme: dark)',
  'reduced motion': '(prefers-reduced-motion: reduce)',
  'more contrast': '(prefers-contrast: more)',
  'forced colors': '(forced-colors: active)'
};

function rows(values) {
  return Object.entries(values).map(([key, value]) => `<div><dt>${key}</dt><dd>${value ?? 'unavailable'}</dd></div>`).join('');
}
function rgb(value) { return value ? `rgb(${value.map(channel => Math.round(channel * 255)).join(', ')})` : null; }

async function refresh() {
  const native = await window.__TAURI__.core.invoke('native_inputs');
  const scheme = matchMedia(queries['color scheme']).matches ? 'dark' : 'light';
  const token = fallbackTokens[scheme];
  const probes = [...document.querySelectorAll('#system-probes > *')];
  const computed = probes.map(element => ({ tag: element.tagName.toLowerCase(), color: getComputedStyle(element).color, background: getComputedStyle(element).backgroundColor }));
  const fonts = ['system-ui', 'ui-monospace', '-apple-system', 'SF Mono', 'Menlo'].map(font => `${font}: ${document.fonts.check(`12px "${font}"`) ? 'available' : 'fallback'}`).join(' · ');

  document.querySelector('#source').textContent = `${native.source} · ${native.platform}/${native.architecture}`;
  document.querySelector('#native-values').innerHTML = rows({
    theme: native.nativeTheme, accent: rgb(native.accentRgb), 'increase contrast': native.increaseContrast,
    'reduce motion': native.reduceMotion, 'reduce transparency': native.reduceTransparency,
    'differentiate without color': native.differentiateWithoutColor, 'invert colors': native.invertColors
  });
  document.querySelector('#media-values').innerHTML = rows(Object.fromEntries(Object.entries(queries).map(([name, query]) => [name, matchMedia(query).matches])));
  document.querySelector('#web-values').innerHTML = rows({ fonts, 'user agent': navigator.userAgent, 'system colors': `${getComputedStyle(document.documentElement).color} on ${getComputedStyle(document.documentElement).backgroundColor}`, controls: computed.map(value => `${value.tag}:${value.color}/${value.background}`).join(' · ') });

  document.querySelector('#scheme-chip').textContent = `${scheme} fallback`;
  document.querySelector('#swatches').innerHTML = Object.entries(token).map(([name, value]) => `<div class="swatch" style="background:${value};color:${name.toLowerCase().includes('text') ? token.surface : token.text}">${name}<br>${value}</div>`).join('');
  const checks = tokenChecks(scheme);
  document.querySelector('#checks').innerHTML = checks.map(([name, ratio, required]) => `<tr><td>${name}</td><td>${ratio.toFixed(2)}:1</td><td>${required}:1</td><td class="${ratio >= required ? 'pass' : ''}">${ratio >= required ? 'PASS' : 'FAIL'}</td></tr>`).join('');
  document.querySelector('#summary').textContent = checks.every(([, ratio, required]) => ratio >= required) ? 'All fallback contrast checks pass' : 'Fallback contrast failure';
}

document.querySelector('#refresh').addEventListener('click', refresh);
refresh().catch(error => { document.querySelector('#summary').textContent = error.message; });
