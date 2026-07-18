import { requiredFixtures } from './matrix.mjs';

const schemeMedia = matchMedia('(prefers-color-scheme: dark)');
const checksNode = document.querySelector('#runtime-checks');
const dialog = document.querySelector('#dialog');
const backdrop = document.querySelector('#backdrop');
const popover = document.querySelector('#popover');
const popoverTrigger = document.querySelector('#popover-trigger');
let previousFocus;

function scheme() { return schemeMedia.matches ? 'dark' : 'light'; }
function applyScheme() {
  document.documentElement.dataset.colorScheme = scheme();
  document.documentElement.style.colorScheme = scheme();
  document.querySelector('#scheme').textContent = `${scheme()} · macOS`;
}
function runChecks() {
  const results = [];
  results.push(['document overflow', document.documentElement.scrollWidth <= document.documentElement.clientWidth]);
  for (const name of requiredFixtures) {
    const node = document.querySelector(`[data-fixture="${name}"]`);
    const style = node ? getComputedStyle(node) : null;
    results.push([`${name} present`, Boolean(node)]);
    if (node && !['dialog', 'popover'].includes(name)) {
      results.push([`${name} contained`, node.scrollWidth <= node.clientWidth + 1 || getComputedStyle(node).overflowX === 'auto']);
      results.push([`${name} themed`, style.backgroundColor !== 'rgba(0, 0, 0, 0)']);
    }
  }
  results.push(['scheme synchronized', document.documentElement.dataset.colorScheme === scheme()]);
  results.push(['reduced motion observable', typeof matchMedia('(prefers-reduced-motion: reduce)').matches === 'boolean']);
  const passed = results.filter(([, value]) => value).length;
  checksNode.textContent = `${passed}/${results.length} runtime checks pass` + (passed === results.length ? '' : ` · ${results.filter(([, value]) => !value).map(([name]) => name).join(', ')}`);
  checksNode.classList.toggle('error', passed !== results.length);
  return results;
}
function openDialog() { previousFocus = document.activeElement; backdrop.hidden = false; dialog.hidden = false; dialog.querySelector('input').focus(); }
function closeDialog() { backdrop.hidden = true; dialog.hidden = true; previousFocus?.focus(); }

document.querySelector('#run-checks').addEventListener('click', runChecks);
document.querySelector('#open-dialog').addEventListener('click', openDialog);
document.querySelector('#cancel-dialog').addEventListener('click', closeDialog);
backdrop.addEventListener('click', closeDialog);
popoverTrigger.addEventListener('click', () => { popover.hidden = !popover.hidden; popoverTrigger.setAttribute('aria-expanded', String(!popover.hidden)); });
addEventListener('keydown', event => { if (event.key === 'Escape') { if (!dialog.hidden) closeDialog(); if (!popover.hidden) { popover.hidden = true; popoverTrigger.setAttribute('aria-expanded', 'false'); popoverTrigger.focus(); } } });
schemeMedia.addEventListener('change', () => { applyScheme(); runChecks(); });
applyScheme();
requestAnimationFrame(runChecks);
