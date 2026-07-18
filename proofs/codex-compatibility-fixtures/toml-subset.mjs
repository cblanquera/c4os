function parseValue(source) {
  const value = source.trim();
  if (value.startsWith('"') && value.endsWith('"')) return JSON.parse(value);
  if (value === 'true') return true;
  if (value === 'false') return false;
  if (/^-?\d+(?:\.\d+)?$/.test(value)) return Number(value);
  if (value.startsWith('[') && value.endsWith(']')) {
    const inner = value.slice(1, -1).trim();
    if (!inner) return [];
    return inner.split(',').map(parseValue);
  }
  throw new Error(`Unsupported TOML value: ${value}`);
}

function assign(root, path, key, value) {
  let cursor = root;
  for (const segment of path) cursor = cursor[segment] ??= {};
  cursor[key] = value;
}

export function parseTomlSubset(source) {
  const result = {};
  let table = [];
  for (const rawLine of source.split(/\r?\n/)) {
    const line = rawLine.trim();
    if (!line || line.startsWith('#')) continue;
    const tableMatch = line.match(/^\[([A-Za-z0-9_.-]+)\]$/);
    if (tableMatch) {
      table = tableMatch[1].split('.');
      continue;
    }
    const entry = line.match(/^([A-Za-z0-9_-]+)\s*=\s*(.+)$/);
    if (!entry) throw new Error(`Unsupported TOML line: ${line}`);
    assign(result, table, entry[1], parseValue(entry[2]));
  }
  return result;
}
