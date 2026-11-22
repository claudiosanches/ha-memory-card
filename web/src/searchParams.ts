/**
 * Search params as plain `key=value` pairs with lists joined by commas, so
 * library links read `?platform=switch,ps5`. Parsed values are strings; each
 * route's `validateSearch` turns them into its own types.
 */
export function stringifySearch(search: Record<string, unknown>): string {
  const pairs: string[] = [];
  for (const [key, value] of Object.entries(search)) {
    const items = (Array.isArray(value) ? value : [value]).filter(
      (item) => item !== undefined && item !== null && item !== '',
    );
    // Items are slugs, keys and codes, which never hold a comma.
    if (items.length) pairs.push(`${encodeURIComponent(key)}=${items.map((item) => encodeURIComponent(String(item))).join(',')}`);
  }
  return pairs.length ? `?${pairs.join('&')}` : '';
}

export function parseSearch(text: string): Record<string, string> {
  const search: Record<string, string> = {};
  for (const pair of text.replace(/^\?/, '').split('&')) {
    if (!pair) continue;
    const at = pair.indexOf('=');
    search[decode(at < 0 ? pair : pair.slice(0, at))] = at < 0 ? '' : decode(pair.slice(at + 1));
  }
  return search;
}

function decode(text: string): string {
  try {
    return decodeURIComponent(text.replace(/\+/g, ' '));
  } catch {
    return text;
  }
}

/** A list param: an array when set by a link, comma-separated text when read from the address. */
export function listParam(value: unknown): string[] {
  const items = Array.isArray(value) ? value : typeof value === 'string' ? value.split(',') : [];
  return items.filter((item): item is string => typeof item === 'string').map((item) => item.trim()).filter(Boolean);
}
