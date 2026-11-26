<script setup>
import { ref } from 'vue';

const REPO_URL = 'https://github.com/claudiosanches/ha-memory-card';
const MY_HA_URL =
  'https://my.home-assistant.io/redirect/supervisor_add_addon_repository/?repository_url=https%3A%2F%2Fgithub.com%2Fclaudiosanches%2Fha-memory-card';

const copied = ref(false);
let resetTimer;

async function copyRepoUrl() {
  try {
    await navigator.clipboard.writeText(REPO_URL);
    copied.value = true;
    clearTimeout(resetTimer);
    resetTimer = setTimeout(() => {
      copied.value = false;
    }, 2000);
  } catch {
    copied.value = false;
  }
}
</script>

<template>
  <section id="install" class="mc-section">
    <div class="mc-install-card">
      <div class="mc-install-top">
        <div>
          <h2>Install in Home Assistant</h2>
          <span class="mc-req-badge">Requires Home Assistant 2026.11 or newer</span>
        </div>
        <a :href="MY_HA_URL" target="_blank" rel="noreferrer" class="mc-ha-badge">
          <img
            src="https://my.home-assistant.io/badges/supervisor_add_addon_repository.svg"
            alt="Open your Home Assistant instance and show the add app repository dialog with a specific repository URL pre-filled."
          />
        </a>
      </div>

      <div class="mc-steps-grid">
        <div class="mc-step">
          <span class="mc-step-num" aria-hidden="true">1</span>
          <h3>Add the repository</h3>
          <p>
            Select the button above, or go to <strong>Settings &gt; Apps &gt; Install app</strong>,
            open the &#8942; menu, choose <strong>Repositories</strong> and paste:
          </p>
          <div class="mc-copy-box">
            <code>{{ REPO_URL }}</code>
            <button type="button" class="mc-copy-btn" aria-live="polite" @click="copyRepoUrl">
              {{ copied ? 'Copied' : 'Copy' }}
            </button>
          </div>
        </div>

        <div class="mc-step">
          <span class="mc-step-num" aria-hidden="true">2</span>
          <h3>Install and start</h3>
          <p>
            Install <strong>Memory Card</strong> and start it. It appears in your Home Assistant
            sidebar with no separate login required.
          </p>
          <p class="mc-step-link">
            <a href="/docs/getting-started">Follow the getting started guide &rarr;</a>
          </p>
        </div>
      </div>
    </div>

    <div class="mc-support-card">
      <div>
        <h3>Free and open source</h3>
        <p>If Memory Card is useful to your household, you can support ongoing development.</p>
      </div>
      <div class="mc-btn-row">
        <a
          class="mc-pill-btn primary"
          href="https://github.com/sponsors/claudiosanches"
          target="_blank"
          rel="noreferrer"
        >
          GitHub Sponsors
        </a>
        <a
          class="mc-pill-btn"
          href="https://ko-fi.com/claudiosanches"
          target="_blank"
          rel="noreferrer"
        >
          Support on Ko-fi
        </a>
      </div>
    </div>
  </section>
</template>
