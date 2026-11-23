export const APP_PORT = 18199;
export const PROXY_PORT = 18200;
export const PREFIX = '/api/hassio_ingress/e2e-token';
export const PROXY_ORIGIN = `http://127.0.0.1:${PROXY_PORT}`;
/** Home Assistant user IDs; requests without the `e2e_user` cookie come from the manager. */
export const MANAGER_ID = 'e2emanager';
export const VIEWER_ID = 'e2eviewer';
/** What the app's page sends with every change. */
export const WRITE_HEADERS = { 'X-Requested-With': 'memory-card' };
