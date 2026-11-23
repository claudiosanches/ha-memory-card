// Sets up the Home Assistant that scripts/ha-demo.sh runs: finishes
// onboarding, adds a non-admin user, installs Mosquitto and the MQTT
// integration, installs or updates the app from the staged folder, and the
// first time fills it with test data and adds a dashboard for its sensors.
// Logins and a token are kept in the state file.
// Usage: node scripts/ha-demo.mjs <url> <state file>
import { randomBytes } from 'node:crypto';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { seed } from './seed.mjs';

const [base, stateFile] = process.argv.slice(2);
if (!base || !stateFile) {
  console.error('usage: node scripts/ha-demo.mjs <url> <state file>');
  process.exit(1);
}
const CLIENT_ID = `${base}/`;
const APP = 'local_memory_card';
const MOSQUITTO = 'core_mosquitto';
const LONG = 1800;

const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));
const newPassword = () => randomBytes(12).toString('base64url');

async function request(method, url, { json, form, token } = {}) {
  const headers = {};
  let body;
  if (json !== undefined) {
    headers['content-type'] = 'application/json';
    body = JSON.stringify(json);
  } else if (form) {
    body = new URLSearchParams(form);
  }
  if (token) headers.authorization = `Bearer ${token}`;
  const response = await fetch(`${base}${url}`, { method, headers, body });
  const text = await response.text();
  if (!response.ok) throw new Error(`${method} ${url}: ${response.status} ${text}`);
  return text ? JSON.parse(text) : null;
}

async function json(url) {
  try {
    const response = await fetch(`${base}${url}`);
    return { status: response.status, body: response.ok ? await response.json() : null };
  } catch {
    return { status: 0, body: null };
  }
}

/** Waits until Home Assistant itself answers, not the page shown while it installs. */
async function waitForCore() {
  const started = Date.now();
  let said = started;
  for (;;) {
    const onboarding = await json('/api/onboarding');
    if (Array.isArray(onboarding.body)) return onboarding.body;
    if (onboarding.status === 404 && (await json('/auth/providers')).body) return null;
    if (Date.now() - said > 60_000) {
      const minutes = Math.floor((Date.now() - started) / 60_000);
      console.log(`Waiting for Home Assistant (${minutes} min). The first start downloads it.`);
      said = Date.now();
    }
    if (Date.now() - started > 45 * 60_000) throw new Error('Home Assistant did not start');
    await sleep(3000);
  }
}

async function onboard(steps) {
  if (steps.some((step) => step.done)) {
    throw new Error('Onboarding was started by hand. Run make ha-destroy and make ha-up.');
  }
  console.log('Onboarding Home Assistant');
  const owner = { name: 'Owner', username: 'owner', password: newPassword() };
  const { auth_code: code } = await request('POST', '/api/onboarding/users', {
    json: { ...owner, client_id: CLIENT_ID, language: 'en' },
  });
  const { access_token: token } = await request('POST', '/auth/token', {
    form: { grant_type: 'authorization_code', code, client_id: CLIENT_ID },
  });
  await request('POST', '/api/onboarding/core_config', { json: {}, token });
  await request('POST', '/api/onboarding/analytics', { json: {}, token });
  await request('POST', '/api/onboarding/integration', {
    json: { client_id: CLIENT_ID, redirect_uri: `${base}/?auth_callback=1` },
    token,
  });

  const ha = await connect(token);
  owner.id = (await ha.call('auth/current_user')).id;
  const longLived = await ha.call('auth/long_lived_access_token', {
    client_name: 'make ha-up',
    lifespan: 3650,
  });
  ha.close();
  return { owner, token: longLived };
}

async function connect(token, url = base) {
  const socket = new WebSocket(`${url.replace(/^http/, 'ws')}/api/websocket`);
  const pending = new Map();
  let next = 0;
  await new Promise((resolve, reject) => {
    socket.addEventListener('message', (event) => {
      const message = JSON.parse(event.data);
      if (message.type === 'auth_required') {
        socket.send(JSON.stringify({ type: 'auth', access_token: token }));
      } else if (message.type === 'auth_ok') {
        resolve();
      } else if (message.type === 'auth_invalid') {
        reject(new Error('Home Assistant refused the saved token. Run make ha-destroy and make ha-up.'));
      } else if (message.type === 'result' && pending.has(message.id)) {
        const call = pending.get(message.id);
        pending.delete(message.id);
        if (message.success) call.resolve(message.result);
        else call.reject(new Error(`${call.what}: ${message.error.message}`));
      }
    });
    socket.addEventListener('error', () => reject(new Error('Could not reach the Home Assistant websocket')));
    socket.addEventListener('close', () => {
      for (const call of pending.values()) call.reject(new Error(`${call.what}: connection closed`));
      pending.clear();
    });
  });
  return {
    call(type, data = {}) {
      const id = ++next;
      socket.send(JSON.stringify({ id, type, ...data }));
      return new Promise((resolve, reject) => pending.set(id, { resolve, reject, what: type }));
    },
    supervisor(method, endpoint, data, timeout = 60) {
      return this.call('supervisor/api', { method, endpoint, data, timeout }).catch((error) => {
        throw new Error(`${method.toUpperCase()} ${endpoint}: ${error.message}`);
      });
    },
    close() {
      socket.close();
    },
  };
}

async function addViewer(ha) {
  const viewer = { name: 'Viewer', username: 'viewer', password: newPassword() };
  const { user } = await ha.call('config/auth/create', {
    name: viewer.name,
    group_ids: ['system-users'],
    local_only: false,
  });
  await ha.call('config/auth_provider/homeassistant/create', {
    user_id: user.id,
    username: viewer.username,
    password: viewer.password,
  });
  return { ...viewer, id: user.id };
}

/** Starts Mosquitto and accepts the MQTT integration it offers. */
async function mosquitto(ha, token) {
  const store = await ha.supervisor('get', `/store/addons/${MOSQUITTO}`);
  if (!store.installed) {
    console.log('Installing Mosquitto');
    await ha.supervisor('post', `/store/addons/${MOSQUITTO}/install`, undefined, LONG);
  }
  const info = await ha.supervisor('get', `/addons/${MOSQUITTO}/info`);
  if (info.state !== 'started') await ha.supervisor('post', `/addons/${MOSQUITTO}/start`, undefined, 300);

  if ((await ha.call('config_entries/get', { domain: 'mqtt' })).length > 0) return;
  for (let tries = 0; tries < 60; tries++) {
    const flows = await ha.call('config_entries/flow/progress');
    const flow = flows.find((f) => f.handler === 'mqtt');
    if (flow) {
      await request('POST', `/api/config/config_entries/flow/${flow.flow_id}`, { json: {}, token });
      return;
    }
    await sleep(2000);
  }
  console.warn('The MQTT integration was not offered. Add it in Settings > Devices & services.');
}

/** Values never printed, even inside an error from the Supervisor. */
const secrets = [];

function redact(text) {
  return (
    secrets
      .reduce((out, secret) => out.replaceAll(secret, '<hidden>'), text)
      // Supervisor errors repeat the app's options, including saved ones.
      .replace(/(['"]\w*(?:secret|password|token)\w*['"]\s*:\s*)(['"])[^'"]*\2/gi, '$1$2<hidden>$2')
  );
}

/** IGDB credentials from `make run`, so covers and search work here too. */
async function igdbOptions() {
  try {
    const options = JSON.parse(await readFile('.data/options.json', 'utf8'));
    if (options.igdb_client_id && options.igdb_client_secret) {
      secrets.push(options.igdb_client_id, options.igdb_client_secret);
      return { igdb_client_id: options.igdb_client_id, igdb_client_secret: options.igdb_client_secret };
    }
  } catch {
    // No local options.
  }
  return {};
}

async function app(ha, state, igdb) {
  await ha.supervisor('post', '/store/reload', undefined, 300);
  const store = await ha.supervisor('get', `/store/addons/${APP}`);
  if (!store.installed) {
    console.log('Building and installing Memory Card');
    await ha.supervisor('post', `/store/addons/${APP}/install`, undefined, LONG);
  } else {
    console.log('Rebuilding Memory Card');
    // A running app restarts after the update, which fails on options its
    // new schema refuses. The update itself is in by then.
    await ha.supervisor('post', `/store/addons/${APP}/update`, { backup: false }, LONG).catch((error) => {
      if (!error.message.includes('has invalid options')) throw error;
    });
    const { valid } = await ha.supervisor('post', `/addons/${APP}/options/validate`, {});
    if (!valid) {
      console.log('The saved options do not fit this version, so they start over from the defaults');
      await ha.supervisor('post', `/addons/${APP}/options`, { options: null });
      state.configured = false;
    }
  }
  // Managers and the sensors are set once; IGDB follows .data/options.json.
  const { options } = await ha.supervisor('get', `/addons/${APP}/info`);
  const igdbChanged = Object.entries(igdb).some(([key, value]) => options[key] !== value);
  if (!state.configured || igdbChanged) {
    // The Supervisor replaces all options at once.
    const first = state.configured ? {} : { managers: [state.owner.id], sensors: 'mqtt' };
    await ha.supervisor('post', `/addons/${APP}/options`, {
      options: { ...options, ...first, ...igdb },
      ingress_panel: true,
    });
    state.configured = true;
    await saveState(state);
  }
  const info = await ha.supervisor('get', `/addons/${APP}/info`);
  if (info.state !== 'started') await ha.supervisor('post', `/addons/${APP}/start`, undefined, 300);
  else if (igdbChanged) await ha.supervisor('post', `/addons/${APP}/restart`, undefined, 300);
}

/** Calls the app's API through ingress, as the owner. */
async function ingress(ha) {
  const { ingress_entry: entry } = await ha.supervisor('get', `/addons/${APP}/info`);
  const { session } = await ha.call('supervisor/ingress/session', { slug: APP });
  const api = async (method, url, body, type = 'application/json') => {
    const headers = { cookie: `ingress_session=${session}`, 'x-requested-with': 'memory-card' };
    if (body !== undefined) headers['content-type'] = type;
    const response = await fetch(`${base}${entry}${url}`, {
      method,
      headers,
      body: body === undefined || typeof body === 'string' ? body : JSON.stringify(body),
    });
    const text = await response.text();
    if (!response.ok) throw new Error(`${method} ${url}: ${response.status} ${text}`);
    return text ? JSON.parse(text) : null;
  };
  for (let tries = 0; ; tries++) {
    try {
      await api('GET', '/api/status');
      return api;
    } catch (error) {
      if (tries > 60) throw error;
      await sleep(1000);
    }
  }
}

const DASHBOARD = 'memory-card-games';

/** A dashboard with every Memory Card entity, once its sensors exist. */
async function dashboard(ha) {
  const dashboards = await ha.call('lovelace/dashboards/list');
  if (dashboards.some((d) => d.url_path === DASHBOARD)) return;
  for (let tries = 0; ; tries++) {
    const states = await ha.call('get_states');
    if (states.some((s) => s.entity_id === 'button.memory_card_roll_pick')) break;
    if (tries > 30) {
      console.warn('The Memory Card sensors did not appear, so no dashboard was added');
      return;
    }
    await sleep(2000);
  }
  await ha.call('lovelace/dashboards/create', {
    url_path: DASHBOARD,
    title: 'Games',
    icon: 'mdi:gamepad-variant-outline',
    show_in_sidebar: true,
    require_admin: false,
    mode: 'storage',
  });
  const tile = (entity, name, wide = false) => ({ type: 'tile', entity, name, ...(wide && { grid_options: { columns: 12 } }) });
  const roll = { action: 'perform-action', perform_action: 'button.press', target: { entity_id: 'button.memory_card_roll_pick' } };
  await ha.call('lovelace/config/save', {
    url_path: DASHBOARD,
    config: {
      views: [
        {
          title: 'Games',
          path: 'games',
          type: 'sections',
          max_columns: 3,
          sections: [
            {
              type: 'grid',
              cards: [
                { type: 'heading', heading: 'Collection' },
                tile('sensor.memory_card_games', 'Games'),
                tile('sensor.memory_card_copies', 'Copies'),
                tile('sensor.memory_card_backlog', 'Backlog'),
                tile('sensor.memory_card_wishlist', 'Wishlist'),
                tile('sensor.memory_card_last_added', 'Last added', true),
              ],
            },
            {
              type: 'grid',
              cards: [
                { type: 'heading', heading: 'Playing' },
                tile('sensor.memory_card_last_played', 'Last played', true),
                {
                  type: 'markdown',
                  content:
                    "{% for game in state_attr('sensor.memory_card_playing', 'games') or [] %}- {{ game }}\n{% else %}Nothing right now.{% endfor %}",
                  grid_options: { columns: 12 },
                },
              ],
            },
            {
              type: 'grid',
              cards: [
                { type: 'heading', heading: 'Pick for me' },
                tile('sensor.memory_card_pick', 'Current pick', true),
                { ...tile('button.memory_card_roll_pick', 'Roll a pick', true), hide_state: true, tap_action: roll, icon_tap_action: roll },
              ],
            },
            {
              type: 'grid',
              column_span: 3,
              cards: [
                { type: 'heading', heading: 'Over time' },
                {
                  type: 'history-graph',
                  hours_to_show: 168,
                  entities: ['sensor.memory_card_games', 'sensor.memory_card_copies', 'sensor.memory_card_backlog', 'sensor.memory_card_wishlist'],
                  grid_options: { columns: 'full' },
                },
              ],
            },
          ],
        },
      ],
    },
  });
}

const CERTIFICATE = { ssl_certificate: '/ssl/memory-card-demo.pem', ssl_key: '/ssl/memory-card-demo.key' };

/**
 * Makes Home Assistant answer at HA_URL, serving HTTPS when that is an
 * https URL, through its HTTP settings. Returns that URL.
 */
async function httpSettings(ha, token) {
  const url = process.env.HA_URL ?? base;
  const https = url.startsWith('https:');
  const { stable, pending } = await ha.call('http/config');
  const settings = { ...stable };
  for (const key of ['created_at', 'error', 'error_message', ...Object.keys(CERTIFICATE)]) delete settings[key];

  if (!pending && Boolean(stable.ssl_certificate) === https) {
    if (!(https && process.env.HA_NEW_CERTIFICATE)) return url;
    console.log('Restarting Home Assistant for its new certificate');
    await ha.call('call_service', { domain: 'homeassistant', service: 'restart' });
    await restarted(url);
    return url;
  }
  console.log(`Switching Home Assistant to ${https ? 'HTTPS' : 'HTTP'}`);
  const { restart } = await ha.call('http/config/configure', { config: https ? { ...settings, ...CERTIFICATE } : settings });
  if (restart) await restarted(url);
  // Home Assistant goes back to the previous settings unless these are confirmed.
  const confirm = await connect(token, url);
  await confirm.call('http/config/promote');
  confirm.close();
  return url;
}

async function answers(url) {
  try {
    return (await fetch(`${url}/auth/providers`)).ok;
  } catch {
    return false;
  }
}

/** Waits for Home Assistant to stop answering at url, then to answer again. */
async function restarted(url) {
  for (let tries = 0; tries < 30 && (await answers(url)); tries++) await sleep(1000);
  for (let tries = 0; !(await answers(url)); tries++) {
    if (tries > 100) throw new Error(`Home Assistant did not come back at ${url}`);
    await sleep(3000);
  }
}

/** Home Assistant answers before the Supervisor takes app operations. */
async function waitForSupervisor(ha) {
  for (let tries = 0; ; tries++) {
    try {
      if ((await ha.supervisor('get', '/info')).state === 'running') return;
    } catch (error) {
      if (tries > 100) throw error;
    }
    if (tries > 100) throw new Error('The Supervisor did not finish starting');
    await sleep(3000);
  }
}

async function main() {
  const igdb = await igdbOptions();
  const steps = await waitForCore();
  let state;
  if (steps && !steps.every((step) => step.done)) {
    state = await onboard(steps);
    await saveState(state);
  } else {
    try {
      state = JSON.parse(await readFile(stateFile, 'utf8'));
    } catch {
      throw new Error(`${stateFile} is missing. Run make ha-destroy and make ha-up.`);
    }
  }

  const ha = await connect(state.token);
  let url = base;
  try {
    await waitForSupervisor(ha);
    if (!state.viewer) {
      state.viewer = await addViewer(ha);
      await saveState(state);
    }
    await mosquitto(ha, state.token);
    await app(ha, state, igdb);
    if (!state.seeded) {
      await seed(await ingress(ha));
      state.seeded = true;
      await saveState(state);
    }
    if (!state.dashboard) {
      await dashboard(ha);
      state.dashboard = true;
      await saveState(state);
    }
    url = await httpSettings(ha, state.token);
  } finally {
    ha.close();
  }

  console.log(`
Home Assistant is at ${url}
  owner / ${state.owner.password}   administrator, made a Memory Card manager at install
  viewer / ${state.viewer.password}   regular user
These logins are kept in ${stateFile}. The app's sensors are on the Games dashboard.`);
}

async function saveState(state) {
  await mkdir(path.dirname(stateFile), { recursive: true });
  await writeFile(stateFile, `${JSON.stringify(state, null, 2)}\n`, { mode: 0o600 });
}

main().catch((error) => {
  console.error(redact(error.message));
  process.exit(1);
});
