<script setup>
import { computed } from 'vue';
import { useThemeCopy } from './i18n.mjs';

const retroDark = 'https://ha-memory-card.pages.dev/images/themes/retro-dark.webp';
const pastelPlushLight = 'https://ha-memory-card.pages.dev/images/themes/pastel-plush-light.webp';
const cyberpunkDark = 'https://ha-memory-card.pages.dev/images/themes/cyberpunk-dark.webp';

const copy = useThemeCopy();

const themes = computed(() => [
  {
    key: 'retro',
    name: copy.value.themes.items.retro.name,
    mode: copy.value.themes.dark,
    src: retroDark,
    alt: copy.value.themes.items.retro.alt,
    href: `${copy.value.prefix}/docs/themes#retro`,
  },
  {
    key: 'pastelPlush',
    name: copy.value.themes.items.pastelPlush.name,
    mode: copy.value.themes.light,
    src: pastelPlushLight,
    alt: copy.value.themes.items.pastelPlush.alt,
    href: `${copy.value.prefix}/docs/themes#pastel-plush`,
  },
  {
    key: 'cyberpunk',
    name: copy.value.themes.items.cyberpunk.name,
    mode: copy.value.themes.dark,
    src: cyberpunkDark,
    alt: copy.value.themes.items.cyberpunk.alt,
    href: `${copy.value.prefix}/docs/themes#cyberpunk`,
  },
]);
</script>

<template>
  <section class="mc-section">
    <div class="mc-section-header">
      <h2>{{ copy.themes.heading }}</h2>
      <p>{{ copy.themes.subtitle }}</p>
    </div>

    <div class="mc-theme-trio">
      <figure v-for="theme in themes" :key="theme.key" class="mc-theme-card mc-zoomable">
        <img :src="theme.src" :alt="theme.alt" loading="lazy" />
        <figcaption class="mc-theme-card-label">
          <a :href="theme.href">{{ theme.name }}</a>
          <span>{{ theme.mode }}</span>
        </figcaption>
      </figure>
    </div>

    <p class="mc-section-link">
      <a :href="`${copy.prefix}/docs/themes`">{{ copy.themes.browseAll }} &rarr;</a>
    </p>
  </section>
</template>
