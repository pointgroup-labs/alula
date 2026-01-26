<script lang="ts" setup>
const {
  label = '',
  to = '/',
  withIcon = true,
} = defineProps<{
  label?: string
  to?: string
  withIcon?: boolean
}>()
</script>

<template>
  <nuxt-link
    style="width: fit-content;"
    :to="to"
  >
    <j-btn
      size="sm"
      class="back-btn"
      variant="secondary"
    >
      <i-app-line-arrow-right
        v-if="withIcon"
        style="transform: rotate(-180deg);"
      /> {{ label }}
    </j-btn>
  </nuxt-link>
</template>

<style scoped lang="scss">
.back-btn {
  padding: 0;
  width: 38px;
  height: 38px;

  :deep(svg) {
    width: 20px;
    height: 20px;
  }
}

.theme-dark .back-btn {
  background-color: $neutral-16;
  border-color: $neutral-16;
  color: #c4c5c7;
}
</style>
