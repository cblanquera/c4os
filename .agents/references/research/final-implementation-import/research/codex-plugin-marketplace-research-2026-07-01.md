# Codex Plugin Marketplace Research

Date: 2026-07-01
Status: research-input

## Sources Checked

- OpenAI Codex manual fetched from `https://developers.openai.com/codex/codex-manual.md`
- OpenAI developer docs: `https://developers.openai.com/codex/plugins`
- OpenAI developer docs: `https://developers.openai.com/codex/plugins/build`
- C4OS context: `.agents/references/context/technical-specs/extension-loading.md`
- C4OS UI handoff: `.agents/references/context/ui-handoff/chunk-003.md`
- C4OS technical context: `.agents/context/technical-specs.md`
- C4OS product context: `.agents/context/product-specs.md`

## Codex Marketplace Findings

- A Codex plugin marketplace is a JSON catalog of plugin entries that Codex can
  read and install.
- Codex marketplace files can be repo-scoped at
  `$REPO_ROOT/.agents/plugins/marketplace.json`, personal at
  `~/.agents/plugins/marketplace.json`, legacy-compatible at
  `$REPO_ROOT/.claude-plugin/marketplace.json`, or curated by OpenAI.
- CLI marketplace sources can be added with `codex plugin marketplace add`.
  Sources can be GitHub shorthand, HTTP/HTTPS Git URLs, SSH Git URLs, or local
  marketplace root directories. Git-backed sources can be pinned by `--ref` and
  narrowed with repeated `--sparse PATH`.
- Marketplace entries point to plugin roots through `source`. Supported source
  shapes include local paths, Git repository roots, and Git subdirectories.
- Codex resolves a marketplace entry's local `source.path` relative to the
  marketplace root, not relative to the `.agents/plugins/` folder.
- Codex installs marketplace plugins into
  `~/.codex/plugins/cache/$MARKETPLACE_NAME/$PLUGIN_NAME/$VERSION/`. Local
  plugins use `local` as the installed version and load from the installed cache
  copy, not directly from the marketplace entry.
- Each plugin has a required `.codex-plugin/plugin.json`. A plugin root can also
  include `skills/`, `hooks/`, `.app.json`, `.mcp.json`, and `assets/`.
- Published plugin manifests commonly include interface metadata for install
  surfaces, including display names, descriptions, capability metadata, legal
  links, starter prompts, brand color, icons/logos, and screenshots.
- Codex stores plugin enabled/disabled state in `~/.codex/config.toml`; install
  and enablement are separate concepts.
- Uninstall removes the plugin bundle from Codex. Bundled apps can remain
  installed in ChatGPT and are managed separately.
- The Codex plugin directory groups plugins by marketplace/source in the CLI
  and by curated/shared/created categories in the app.

## C4OS Context Findings

- C4OS already models Settings > Plugins as a catalog with search, a C4OS-built
  filter, plugin cards, an Add Marketplace action, and a marketplace dialog with
  Source, Git ref, and Sparse paths.
- C4OS extension loading context says discovery should read metadata first and
  defer full skill loading, hook execution, MCP launch, and runtime access until
  explicit enablement through app-owned records.
- C4OS product context requires plugin, skill, and MCP install/connect flows,
  but extension runtime impact remains disabled until explicit enablement.
- C4OS app plugins need an additional `agents/c4os.yaml` shell contribution
  descriptor and `[plugin root]/c4os` application code, but those should layer
  on top of Codex plugin package/marketplace compatibility rather than replace
  it.

## C4OS Implications

- C4OS should treat marketplace source management as a first-class Settings >
  Plugins concern, not just as arbitrary filesystem plugin roots.
- C4OS should likely mirror Codex marketplace source types: repo marketplace,
  personal marketplace, local marketplace root, GitHub shorthand, Git URL, ref,
  and sparse paths.
- C4OS should distinguish these lifecycle states:
  discovered in marketplace, installed in cache, enabled, disabled, uninstalled,
  dependency-blocked, incompatible, unavailable, and update-available.
- C4OS should install/load marketplace plugins from an app-owned cache copy
  rather than directly from the marketplace entry, because Codex does this and
  it stabilizes plugin execution against source drift.
- C4OS should keep install, enablement, app/plugin configuration, and runtime
  authority as separate records.
- Built-in C4OS plugins can be represented as a bundled/default marketplace
  source, which may align better with Codex compatibility than treating built-in
  plugin roots as a special ad hoc source.
- A full compatibility proof should parse a real Codex-compatible marketplace,
  resolve local and Git-subdir plugin entries, install/cache a plugin copy, read
  `.codex-plugin/plugin.json`, then layer `agents/c4os.yaml` into a normalized
  C4OS plugin record without activating runtime behavior.

## Open Decisions For Grill

- Should C4OS model built-in plugins as a bundled marketplace source?
- Which marketplace source types should C4OS expose in Settings?
- Should C4OS use a Codex-style cache path, C4OS-owned cache path, or both?
- After uninstall, should reinstall come from the marketplace source/cache model
  rather than a special built-in restore path?
- Should plugin enable/disable require restart, follow Codex restart behavior,
  or be live where C4OS can safely unload shell surfaces?
