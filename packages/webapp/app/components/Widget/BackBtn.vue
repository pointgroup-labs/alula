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
      size="xs"
      class="back-btn"
      variant="ghost"
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
  width: 30px;
  height: 30px;
  border-radius: $radius-sm;
  background-color: transparent;

  :deep(svg) {
    width: 16px;
    height: 16px;
  }

  &:hover {
    color: $text-primary;
  }
}
</style>
