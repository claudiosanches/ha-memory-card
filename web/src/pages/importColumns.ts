/** Where the header row starting at `from` ends: the next line break outside quotes. */
function headerEnd(text: string, from: number): number {
  let quoted = false;
  for (let i = from; i < text.length; i++) {
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
  // The server skips blank lines before the header; keeping them keeps its line numbers.
  const start = body.length - body.replace(/^[\r\n]+/, '').length;
  return body.slice(0, start) + header + body.slice(headerEnd(body, start));
}
