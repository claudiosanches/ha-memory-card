/**
 * Where the header row starting at `from` ends: the next line break outside
 * quotes. As in the server's reader, a quote opens a field only at its start.
 */
function headerEnd(text: string, from: number, delimiter: string): number {
  let quoted = false;
  let fieldStart = true;
  for (let i = from; i < text.length; i++) {
    const c = text[i];
    if (quoted) {
      if (c === '"' && text[i + 1] === '"') i++;
      else if (c === '"') quoted = false;
    } else if (c === '\n' || c === '\r') {
      return i;
    } else if (c === '"' && fieldStart) {
      quoted = true;
    }
    fieldStart = !quoted && c === delimiter;
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
  return body.slice(0, start) + header + body.slice(headerEnd(body, start, delimiter));
}
