# QA Notes

Date: 2026-06-19

Preview server:

- `http://127.0.0.1:4182/`

Route checks:

- `200 index.html`
- `200 app-start.html`
- `200 new-session.html`
- `200 chat-session.html`
- `200 providers-popover.html`
- `200 models-popover.html`
- `200 file-explorer.html`
- `200 file-editor.html`
- `200 terminal.html`
- `200 settings-providers.html`
- `200 settings-add-provider.html`
- `200 settings-models.html`
- `200 settings-configuration.html`

Static guideline checks:

- No root-relative generated asset links found.
- No `transition: all` found.
- No disabled browser zoom meta settings found.
- Provider form controls have explicit labels, ids, names, and autocomplete attributes.
- Focus-visible styles and reduced-motion handling are defined in `styles.css`.

Browser automation:

- Playwright package is available, but bundled Chromium is not installed.
- System Google Chrome is installed, but headless launch exited under automation in this environment.
- Rendered screenshots were not produced; browser visual verification remains pending.
