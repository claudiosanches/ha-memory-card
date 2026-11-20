// Fails when a production dependency of the web app has a license that is not
// on the GPL-3.0 compatible allowlist (same list as deny.toml).
import { execFileSync } from 'node:child_process';

const ALLOWED = new Set([
  '0BSD',
  'Apache-2.0',
  'BSD-2-Clause',
  'BSD-3-Clause',
  'BSL-1.0',
  'CC0-1.0',
  'ISC',
  'MIT',
  'MPL-2.0',
  'Unicode-3.0',
  'Unlicense',
  'Zlib',
]);

/** Evaluates simple SPDX expressions: `A OR B`, `A AND B`, parentheses. */
function allowed(expression) {
  const expr = expression.trim().replace(/^\((.*)\)$/, '$1');
  if (/\sOR\s/.test(expr)) return expr.split(/\s+OR\s+/).some(allowed);
  if (/\sAND\s/.test(expr)) return expr.split(/\s+AND\s+/).every(allowed);
  return ALLOWED.has(expr);
}

const output = execFileSync('npm', ['query', '.workspace:attr([name=memory-card-web]) .prod'], {
  encoding: 'utf8',
  maxBuffer: 64 * 1024 * 1024,
});
const packages = JSON.parse(output);

const failures = packages.filter((pkg) => {
  const license = typeof pkg.license === 'string' ? pkg.license : pkg.license?.type;
  return !license || !allowed(license);
});

for (const pkg of failures) {
  console.error(`${pkg.name}@${pkg.version}: ${JSON.stringify(pkg.license ?? null)}`);
}
console.log(`${packages.length} production packages checked, ${failures.length} not allowed`);
process.exit(failures.length ? 1 : 0);
