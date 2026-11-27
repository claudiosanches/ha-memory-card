<script setup>
import { ref } from 'vue';
import { useThemeCopy } from './i18n.mjs';

const REPO_URL = 'https://github.com/claudiosanches/ha-memory-card';
const MY_HA_URL =
  'https://my.home-assistant.io/redirect/supervisor_add_addon_repository/?repository_url=https%3A%2F%2Fgithub.com%2Fclaudiosanches%2Fha-memory-card';

const copy = useThemeCopy();
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
          <h2>{{ copy.install.heading }}</h2>
          <span class="mc-req-badge">{{ copy.install.requires }}</span>
        </div>
        <a :href="MY_HA_URL" target="_blank" rel="noreferrer" class="mc-ha-badge">
          <img
            src="https://my.home-assistant.io/badges/supervisor_add_addon_repository.svg"
            :alt="copy.install.badgeAlt"
          />
        </a>
      </div>

      <div class="mc-steps-grid">
        <div class="mc-step">
          <span class="mc-step-num" aria-hidden="true">1</span>
          <h3>{{ copy.install.step1Title }}</h3>
          <p>
            {{ copy.install.step1Before }}<strong>{{ copy.install.step1Path }}</strong>{{ copy.install.step1Middle }}<strong>{{ copy.install.step1Repos }}</strong>{{ copy.install.step1After }}
          </p>
          <div class="mc-copy-box">
            <code>{{ REPO_URL }}</code>
            <button type="button" class="mc-copy-btn" aria-live="polite" @click="copyRepoUrl">
              {{ copied ? copy.install.copied : copy.install.copy }}
            </button>
          </div>
        </div>

        <div class="mc-step">
          <span class="mc-step-num" aria-hidden="true">2</span>
          <h3>{{ copy.install.step2Title }}</h3>
          <p>
            {{ copy.install.step2Before }}<strong>{{ copy.install.step2App }}</strong>{{ copy.install.step2After }}
          </p>
          <p class="mc-step-link">
            <a :href="`${copy.prefix}/docs/getting-started`">{{ copy.install.gettingStarted }} &rarr;</a>
          </p>
        </div>
      </div>
    </div>

    <div class="mc-support-card">
      <div>
        <h3>{{ copy.install.supportHeading }}</h3>
        <p>{{ copy.install.supportText }}</p>
      </div>
      <div class="mc-btn-row">
        <a
          class="mc-pill-btn primary"
          href="https://github.com/sponsors/claudiosanches"
          target="_blank"
          rel="noreferrer"
        >
          {{ copy.install.githubSponsors }}
        </a>
        <a
          class="mc-pill-btn"
          href="https://ko-fi.com/claudiosanches"
          target="_blank"
          rel="noreferrer"
        >
          {{ copy.install.kofi }}
        </a>
      </div>
    </div>
  </section>
</template>
