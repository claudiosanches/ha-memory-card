<script setup>
import { useData } from 'vitepress';
import defaultLight from '../../images/themes/default-light.webp';
import defaultDark from '../../images/themes/default-dark.webp';

const { frontmatter } = useData();
</script>

<template>
  <div v-if="frontmatter.layout === 'home'" class="mc-hero-media">
    <div class="mc-shot-frame mc-zoomable">
      <img
        class="mc-light-only"
        :src="defaultLight"
        alt="The library in Memory Card, light theme"
      />
      <img
        class="mc-dark-only"
        :src="defaultDark"
        alt="The library in Memory Card, dark theme"
      />
    </div>
    <p class="mc-trademark">
      <em>Game titles and cover art belong to their respective owners.</em>
    </p>
  </div>
</template>
