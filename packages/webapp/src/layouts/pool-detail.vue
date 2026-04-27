<script setup lang="ts">
import { TermsWindow } from '~/features/terms'
</script>

<template>
  <app-header />
  <invalid-account-warning />
  <main class="market-detail-page container">
    <pool-detail-top />
    <NuxtPage />
  </main>
  <app-footer />
  <client-only>
    <terms-window />
  </client-only>
</template>
