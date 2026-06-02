<script setup lang="ts">
import { TermsWindow } from '~/features/terms'
</script>

<template>
  <app-header />
  <main class="container">
    <NuxtPage />
  </main>
  <app-footer />
  <client-only>
    <terms-window />
  </client-only>
</template>
