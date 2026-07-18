export const fallbackTokens = {
  light: {
    surface: '#f7f7f8', text: '#161718', secondary: '#55585d', border: '#b8bbc0',
    accent: '#0067c0', accentText: '#ffffff', danger: '#b42318'
  },
  dark: {
    surface: '#1b1c1e', text: '#f5f5f6', secondary: '#b8bbc0', border: '#595c62',
    accent: '#4da3ff', accentText: '#06111d', danger: '#ff8a80'
  }
};

function rgb(hex) {
  const value = hex.replace('#', '');
  return [0, 2, 4].map(offset => parseInt(value.slice(offset, offset + 2), 16) / 255);
}
function luminance(hex) {
  return rgb(hex).map(value => value <= .04045 ? value / 12.92 : ((value + .055) / 1.055) ** 2.4)
    .reduce((sum, value, index) => sum + value * [.2126, .7152, .0722][index], 0);
}
export function contrastRatio(foreground, background) {
  const values = [luminance(foreground), luminance(background)].sort((a, b) => b - a);
  return (values[0] + .05) / (values[1] + .05);
}

export function tokenChecks(scheme) {
  const token = fallbackTokens[scheme];
  return [
    ['primary text', contrastRatio(token.text, token.surface), 4.5],
    ['secondary text', contrastRatio(token.secondary, token.surface), 4.5],
    ['accent text', contrastRatio(token.accentText, token.accent), 4.5],
    ['danger text', contrastRatio(token.danger, token.surface), 4.5]
  ];
}
