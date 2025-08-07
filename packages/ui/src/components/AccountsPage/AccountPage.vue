<template>
  <main class="account-page container">
    <div class="account-cards card">
      <account-supplie-table />
      <div class="separator-vert" />
      <account-borrow-table />
    </div>
  </main>
</template>

<style lang="scss">
.account-page {
  .account-cards {
    display: grid;
    grid-template-columns: 1fr 1px 1fr;
    gap: $spacing-32;

    @media (max-width: $breakpoint-sm) {
      display: flex;
      flex-direction: column;
      gap: $spacing-16;
    }
  }
}
</style>
