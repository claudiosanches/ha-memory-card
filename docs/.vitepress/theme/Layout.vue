<script setup>
import DefaultTheme from 'vitepress/theme';
import HomeHeroMedia from './HomeHeroMedia.vue';

const { Layout } = DefaultTheme;
</script>

<template>
  <Layout>
    <template #home-hero-info-before>
      <img class="mc-hero-logo" src="/favicon.svg" alt="" aria-hidden="true" />
    </template>
    <template #home-hero-after>
      <HomeHeroMedia />
    </template>
  </Layout>
</template>
