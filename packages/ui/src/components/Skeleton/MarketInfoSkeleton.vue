<script lang="ts" setup>
const { width } = useWindowSize()

const sizes = computed(() =>
  width.value > 650
    ? {
        width: '337',
        height: '168' }
    : {
        width: '165',
        height: '76',
      })
</script>

<template>
  <j-skeleton
    :width="sizes.width"
    :height="sizes.height"
    style="border-radius: 16px;"
  />
</template>
