<script setup>
import { useData } from 'vitepress';
import { useThemeCopy } from './i18n.mjs';

const libraryLight = 'https://ha-memory-card.pages.dev/images/library.webp';
const libraryDark = 'https://ha-memory-card.pages.dev/images/library-dark.webp';

const { frontmatter } = useData();
const copy = useThemeCopy();
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
        :alt="copy.hero.lightAlt"
      />
      <img
        class="mc-dark-only"
        width="1440"
        height="900"
        loading="lazy"
        :src="libraryDark"
        :alt="copy.hero.darkAlt"
      />
    </div>
    <p class="mc-trademark">
      <em>{{ copy.hero.trademark }}</em>
    </p>
  </div>
</template>
