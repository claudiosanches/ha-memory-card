/** Where the header row ends: the first line break outside quotes. */
function headerEnd(text: string): number {
  let quoted = false;
  for (let i = 0; i < text.length; i++) {
    const c = text[i];
    if (c === '"') quoted = !quoted;
    else if (!quoted && (c === '\n' || c === '\r')) return i;
  }
  return text.length;
}

function quote(value: string, delimiter: string): string {
  return /["\r\n]/.test(value) || value.includes(delimiter) ? `"${value.replaceAll('"', '""')}"` : value;
}

/**
 * The file with its header row replaced: a column name for each mapped header,
 * and a name the import ignores for the rest. The rows stay as they are.
 */
export function withColumns(text: string, columns: (string | null)[], delimiter: string): string {
  const body = text.replace(/^\uFEFF/, '');
  const header = columns.map((column, i) => quote(column ?? `ignored ${i + 1}`, delimiter)).join(delimiter);
  return header + body.slice(headerEnd(body));
}
