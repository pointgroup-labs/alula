<template>
  <main class="account-page container">
    <div class="account-cards card">
      <account-supply-table />
      <account-borrow-table />
    </div>

    <div class="account-multiply__cards card">
      <div class="account-card__title">
        Your Multiplies
      </div>
      <account-multiply-table only-multiplied />
    </div>
  </main>
</template>

<style lang="scss">
.account-page {
  .account-cards {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 16px;
    background-color: transparent;
    border: none;

    @media (max-width: $breakpoint-sm) {
      display: flex;
      flex-direction: column;
      gap: 16px;
    }
  }

  .account-multiply__cards {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }
}
</style>
