<script setup>
import { useData } from 'vitepress';
const libraryLight = 'https://ha-memory-card.pages.dev/images/library.webp';
const libraryDark = 'https://ha-memory-card.pages.dev/images/library-dark.webp';

const { frontmatter } = useData();
</script>

<template>
  <div v-if="frontmatter.layout === 'home'" class="mc-hero-media">
    <div class="mc-shot-frame mc-zoomable">
      <img
        class="mc-light-only"
        width="1440"
        height="900"
        loading="lazy"
        :src="libraryLight"
        alt="The library in Memory Card, light theme"
      />
      <img
        class="mc-dark-only"
        width="1440"
        height="900"
        loading="lazy"
        :src="libraryDark"
        alt="The library in Memory Card, dark theme"
      />
    </div>
    <p class="mc-trademark">
      <em>Game titles and cover art belong to their respective owners.</em>
    </p>
  </div>
</template>
