/** Converts trusted lifecycle identifiers to concise visible labels. */
export function stateLabel(value: string): string {
  return value
    .replace(/([a-z])([A-Z])/gu, "$1 $2")
    .replace(/^./u, (letter) => letter.toLocaleUpperCase());
}
