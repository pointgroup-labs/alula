<template>
  <main class="markets-page container">
    <market-filters />
    <market-info />
    <markets />
  </main>
</template>

<style lang="scss">
.markets-page {
  display: flex;
  flex-direction: column;
  gap: $spacing-24;
}
</style>
