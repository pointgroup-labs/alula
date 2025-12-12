<script lang="ts" setup>
const { width } = useWindowSize()
</script>

<template>
  <main class="markets-page container">
    <not-valid-account-warning />
    <h2 class="markets-page__title">
      Stellar Lending
    </h2>
    <market-info v-if="width > 1024" />
    <markets />
  </main>
</template>
