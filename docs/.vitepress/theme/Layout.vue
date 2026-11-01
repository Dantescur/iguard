<script setup lang="ts">
import DefaultTheme from 'vitepress/theme'
import { useData } from 'vitepress'
import AuthPanel from './components/AuthPanel.vue'
import InstallCommand from './components/InstallCommand.vue'

const { Layout } = DefaultTheme
const { page } = useData()
</script>

<template>
  <Layout>
    <template #home-hero-info-before>
      <p class="iguard-eyebrow">
        <span class="iguard-eyebrow__dot" aria-hidden="true" />
        Polkit session agent
        <span class="iguard-eyebrow__sep" aria-hidden="true">/</span>
        Rust + Iced
      </p>
    </template>

    <template #home-hero-image>
      <AuthPanel />
    </template>

    <template #home-hero-actions-after>
      <InstallCommand />
    </template>

    <template #home-features-before>
      <div class="iguard-section-label">
        <span>Why iguard</span>
        <span class="iguard-section-label__line" aria-hidden="true" />
      </div>
    </template>

    <template #doc-before>
      <p class="iguard-doc-eyebrow">
        <span class="iguard-doc-eyebrow__mark" aria-hidden="true" />
        iguard / {{ page.title || 'guide' }}
      </p>
    </template>
  </Layout>
</template>
