// Fails when a production dependency of the web app has a license that is not
// on the allowlist (same list as deny.toml).
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

/**
 * Evaluates an SPDX expression: AND binds tighter than OR and parentheses
 * group. A `WITH` exception, or an expression it cannot read, is not allowed.
 */
function allowed(expression) {
  const tokens = expression.match(/[()]|[^\s()]+/g) ?? [];
  let at = 0;
  const operator = (name) => tokens[at]?.toUpperCase() === name && ++at > 0;
  const fail = () => {
    throw new Error(expression);
  };
  // A license or exception name: no parenthesis or operator.
  const name = () => {
    const token = tokens[at++];
    if (token === undefined || token === '(' || token === ')' || ['AND', 'OR', 'WITH'].includes(token.toUpperCase())) fail();
    return token;
  };
  const term = () => {
    if (tokens[at] === '(') {
      at++;
      const ok = any();
      if (tokens[at++] !== ')') fail();
      return ok;
    }
    const license = name();
    if (operator('WITH')) {
      name();
      return false;
    }
    return ALLOWED.has(license);
  };
  const all = () => {
    let ok = term();
    while (operator('AND')) ok = term() && ok;
    return ok;
  };
  const any = () => {
    let ok = all();
    while (operator('OR')) ok = all() || ok;
    return ok;
  };
  try {
    const ok = any();
    return at === tokens.length && ok;
  } catch {
    return false;
  }
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
