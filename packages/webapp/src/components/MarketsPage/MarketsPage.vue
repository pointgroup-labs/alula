<script lang="ts" setup>
const { width } = useWindowSize()
</script>

<template>
  <main class="page-container container">
    <not-valid-account-warning />
    <markets-info v-if="width > 1024" />
    <markets />
  </main>
</template>
