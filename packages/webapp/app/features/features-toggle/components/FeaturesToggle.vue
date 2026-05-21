<script setup lang="ts">
import { useFeatureToggle } from '../index'

defineProps<{
  feature: string
  fallback?: boolean
}>()

const { isEnabled, isProd } = useFeatureToggle()
</script>

<template>
  <slot v-if="isEnabled(feature) && !isProd" />
  <slot
    v-else
    name="disabled"
  />
</template>
