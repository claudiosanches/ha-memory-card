// Stands in for the Supervisor: serves the app under an ingress prefix, adds
// X-Ingress-Path, and refuses anything outside the prefix so absolute URLs
// in the UI fail the tests. Like the Supervisor, it replaces any
// X-Remote-User-* headers: the `e2e_user` cookie names the user (empty for
// none), and without the cookie the user is DEFAULT_USER, if set.
import http from 'node:http';

const PREFIX = process.env.INGRESS_PREFIX ?? '/api/hassio_ingress/e2e-token';
const UPSTREAM = new URL(process.env.UPSTREAM ?? 'http://127.0.0.1:18099');
const PORT = Number(process.env.PORT ?? 18100);
const DEFAULT_USER = process.env.DEFAULT_USER ?? '';

function cookie(req: http.IncomingMessage, name: string): string | undefined {
  for (const part of (req.headers.cookie ?? '').split(';')) {
    const [key, ...value] = part.trim().split('=');
    if (key === name) return decodeURIComponent(value.join('='));
  }
  return undefined;
}

const server = http.createServer((req, res) => {
  const url = req.url ?? '/';
  if (url !== PREFIX && !url.startsWith(`${PREFIX}/`) && !url.startsWith(`${PREFIX}?`)) {
    res.writeHead(404, { 'Content-Type': 'text/plain' }).end(`outside ingress prefix: ${url}`);
    return;
  }

  const headers: http.OutgoingHttpHeaders = {
    ...req.headers,
    host: UPSTREAM.host,
    'x-ingress-path': PREFIX,
  };
  for (const name of Object.keys(headers)) {
    if (name.startsWith('x-remote-user-')) delete headers[name];
  }
  const user = cookie(req, 'e2e_user') ?? DEFAULT_USER;
  if (user) {
    headers['x-remote-user-id'] = user;
    headers['x-remote-user-name'] = user;
    headers['x-remote-user-display-name'] = `Player ${user}`;
  }

  const upstream = http.request(
    {
      hostname: UPSTREAM.hostname,
      port: UPSTREAM.port,
      method: req.method,
      path: url.slice(PREFIX.length) || '/',
      headers,
    },
    (upstreamRes) => {
      res.writeHead(upstreamRes.statusCode ?? 502, upstreamRes.headers);
      upstreamRes.pipe(res);
    },
  );
  upstream.on('error', (err) => {
    res.writeHead(502, { 'Content-Type': 'text/plain' }).end(String(err));
  });
  req.pipe(upstream);
});

server.listen(PORT, '127.0.0.1', () => {
  console.log(`ingress proxy on http://127.0.0.1:${PORT}${PREFIX}/ -> ${UPSTREAM.href}`);
});
