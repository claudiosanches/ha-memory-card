<script setup>
const retroDark = 'https://ha-memory-card.pages.dev/images/themes/retro-dark.webp';
const pastelPlushLight = 'https://ha-memory-card.pages.dev/images/themes/pastel-plush-light.webp';
const cyberpunkDark = 'https://ha-memory-card.pages.dev/images/themes/cyberpunk-dark.webp';

const themes = [
  {
    name: 'Retro',
    mode: 'Dark',
    src: retroDark,
    alt: 'Retro theme in dark mode',
    href: '/docs/themes#retro',
  },
  {
    name: 'Pastel Plush',
    mode: 'Light',
    src: pastelPlushLight,
    alt: 'Pastel Plush theme in light mode',
    href: '/docs/themes#pastel-plush',
  },
  {
    name: 'Cyberpunk',
    mode: 'Dark',
    src: cyberpunkDark,
    alt: 'Cyberpunk theme in dark mode',
    href: '/docs/themes#cyberpunk',
  },
];
</script>

<template>
  <section class="mc-section">
    <div class="mc-section-header">
      <h2>12 built-in themes</h2>
      <p>Five quiet palettes and seven expressive themes, each in light and dark.</p>
    </div>

    <div class="mc-theme-trio">
      <figure v-for="theme in themes" :key="theme.name" class="mc-theme-card mc-zoomable">
        <img :src="theme.src" :alt="theme.alt" loading="lazy" />
        <figcaption class="mc-theme-card-label">
          <a :href="theme.href">{{ theme.name }}</a>
          <span>{{ theme.mode }}</span>
        </figcaption>
      </figure>
    </div>

    <p class="mc-section-link">
      <a href="/docs/themes">Browse all 12 themes in light &amp; dark &rarr;</a>
    </p>
  </section>
</template>
