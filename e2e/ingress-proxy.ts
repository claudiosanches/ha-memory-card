// Stands in for the Supervisor: serves the app under an ingress prefix, adds
// X-Ingress-Path, and refuses anything outside the prefix so absolute URLs
// in the UI fail the tests.
import http from 'node:http';

const PREFIX = process.env.INGRESS_PREFIX ?? '/api/hassio_ingress/e2e-token';
const UPSTREAM = new URL(process.env.UPSTREAM ?? 'http://127.0.0.1:18099');
const PORT = Number(process.env.PORT ?? 18100);

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
