const dialog = document.querySelector('#settings');
const backdrop = document.querySelector('#backdrop');
const closeButton = document.querySelector('#close-settings');
let previousFocus;

function openSettings(source) {
  previousFocus = document.activeElement;
  document.querySelector('#last-action').textContent = `Settings opened from ${source}`;
  backdrop.hidden = false;
  dialog.hidden = false;
  closeButton.focus();
}
function closeSettings() {
  backdrop.hidden = true;
  dialog.hidden = true;
  previousFocus?.focus();
}

window.__C4OS_OPEN_SETTINGS__ = openSettings;
closeButton.addEventListener('click', closeSettings);
backdrop.addEventListener('click', closeSettings);
addEventListener('keydown', event => { if (event.key === 'Escape' && !dialog.hidden) closeSettings(); });
