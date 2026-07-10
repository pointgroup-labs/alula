<script lang="ts" setup>
import { isDark } from '~/composables/theme'
</script>

<template>
  <div
    class="setting-item theme"
    @click="toggleDark()"
  >
    <div class="setting-item__title">
      {{ $t('common.darkMode') }}
    </div>

    <j-toggle
      v-model="isDark"
      @click.stop
    />
  </div>
</template>

<style lang="scss">
.setting-item.theme {
  display: flex;
  align-items: center;
  justify-content: space-between;
  cursor: pointer;
  user-select: none;
}
</style>
