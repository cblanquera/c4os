export const requiredFixtures = ['shell', 'transcript', 'artifact', 'dialog', 'popover', 'editor', 'terminal', 'settings'];

export function sourceChecks({ html, css, script }) {
  return [
    ['all representative fixtures exist', requiredFixtures.every(name => html.includes(`data-fixture="${name}"`))],
    ['semantic surface tokens exist', ['--surface-window', '--surface-sidebar', '--surface-raised', '--surface-sunken', '--surface-overlay'].every(name => css.includes(name))],
    ['semantic state tokens exist', ['--focus-ring', '--danger', '--warning', '--success', '--selection'].every(name => css.includes(name))],
    ['reduced motion is handled', css.includes('prefers-reduced-motion: reduce')],
    ['live scheme changes are observed', script.includes("prefers-color-scheme: dark") && script.includes("addEventListener('change'")],
    ['runtime overflow checks exist', script.includes('scrollWidth') && script.includes('clientWidth')]
  ];
}
