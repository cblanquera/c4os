const media = matchMedia('(prefers-color-scheme: dark)');
const events = document.querySelector('#events');
let nativeTheme = null;
let changes = 0;
let counter = 7;

function webviewTheme() { return media.matches ? 'dark' : 'light'; }
function log(kind, detail, at = Date.now()) {
  const item = document.createElement('li');
  item.innerHTML = `<strong>${kind}</strong> · ${new Date(Number(at)).toISOString()} · ${detail}`;
  events.prepend(item);
}
function render() {
  const web = webviewTheme();
  document.documentElement.dataset.colorScheme = web;
  document.documentElement.style.colorScheme = web;
  document.querySelector('#native-theme').textContent = nativeTheme ?? 'unavailable';
  document.querySelector('#webview-theme').textContent = web;
  document.querySelector('#root-theme').textContent = document.documentElement.dataset.colorScheme;
  document.querySelector('#change-count').textContent = String(changes);
  const agreement = document.querySelector('#agreement');
  const agrees = nativeTheme === null || nativeTheme === web;
  agreement.textContent = nativeTheme === null ? 'Fallback active' : agrees ? 'Signals agree' : 'Mismatch';
  agreement.classList.toggle('pass', agrees);
}
async function capture(kind = 'manual capture') {
  const snapshot = await window.__TAURI__.core.invoke('theme_snapshot');
  nativeTheme = snapshot.nativeTheme;
  document.querySelector('#platform').textContent = snapshot.platform;
  document.querySelector('#architecture').textContent = snapshot.architecture;
  log(kind, `native=${nativeTheme ?? 'null'}, webview=${webviewTheme()}`, snapshot.capturedAtMs);
  render();
}

media.addEventListener('change', () => {
  changes += 1;
  log('webview change', `prefers-color-scheme=${webviewTheme()}`);
  capture('post-webview snapshot');
});

window.__TAURI__.event.listen('native-theme-changed', ({ payload }) => {
  const value = typeof payload === 'string' ? JSON.parse(payload) : payload;
  nativeTheme = value.theme;
  changes += 1;
  log('native change', `theme=${value.theme}`, value.capturedAtMs);
  render();
});

document.querySelector('#refresh').addEventListener('click', () => capture());
document.querySelector('#increment').addEventListener('click', () => { counter += 1; document.querySelector('#counter').value = counter; });
document.querySelector('#decrement').addEventListener('click', () => { counter -= 1; document.querySelector('#counter').value = counter; });

const boot = window.__C4OS_NATIVE_BOOT__;
if (boot) log('native boot', `theme=${boot.theme ?? 'null'}`, boot.capturedAtMs);
log('document ready', `root=${document.documentElement.dataset.colorScheme}`);
capture('initial snapshot').catch(error => log('error', error.message));
