<template>
  <main class="multiply-page container">
    <section class="multiply-page__hero">
      <div>
        <div class="multiply-page__eyebrow">
          Atomic leverage
        </div>
        <h1 class="multiply-page__title">
          Build multiply positions through one market batch.
        </h1>
        <p class="multiply-page__copy">
          The new flow borrows margin, swaps into collateral through Soroswap, deposits the received asset, and closes the flash leg with a terminal borrow in a single transaction.
        </p>
      </div>
    </section>

    <div class="multiply-page__content">
      <multiply-table />
    </div>
  </main>
</template>

<style lang="scss">
.multiply-page {
  display: flex;
  flex-direction: column;
  gap: 32px;
  padding-bottom: 72px;

  &__hero {
    padding: $spacing-4xl;
    border-radius: 32px;
    background:
      radial-gradient(circle at top left, rgba(45, 212, 191, 0.18), transparent 30%),
      linear-gradient(135deg, #0f172a 0%, #12323a 55%, #f3f8f9 180%);
    color: white;

    @media (max-width: $breakpoint-xs) {
    padding: $spacing-2xl;
    }
  }

  &__eyebrow {
    font-size: 12px;
    font-weight: 700;
    letter-spacing: 0.1em;
    text-transform: uppercase;
    color: rgba(255, 255, 255, 0.72);
  }

  &__title {
    margin: 12px 0 10px;
    max-width: 720px;
    font-size: clamp(32px, 5vw, 52px);
    line-height: 0.98;
  }

  &__copy {
    margin: 0;
    max-width: 700px;
    color: rgba(255, 255, 255, 0.84);
    font-size: 16px;
    line-height: 1.7;
  }
}
</style>
