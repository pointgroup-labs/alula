<script setup lang="ts">
import { TermsWindow } from '~/features/terms'
</script>

<template>
  <app-header />
  <NuxtPage />
  <app-footer />
  <client-only>
    <terms-window />
  </client-only>
</template>
