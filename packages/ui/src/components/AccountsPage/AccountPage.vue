<template>
  <main class="account-page container">
    <div class="account-cards card">
      <account-card-supplies />
      <div class="separator-vert" />
      <account-card-borrows />
    </div>
  </main>
</template>

<style lang="scss">
.account-page {
  .account-cards {
    display: grid;
    grid-template-columns: 1fr 1px 1fr;
    gap: $spacing-32;
  }
}
</style>
