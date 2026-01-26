<script lang="ts" setup>

</script>

<template>
  <div class="market-info-card">
    <slot />
  </div>
</template>

<style lang="scss">
.market-info-card {
  display: flex;
  flex-direction: column;
  text-wrap: nowrap;
  justify-content: center;
  align-items: center;
  padding: 16px 24px;
  border: 1px solid #212c421a;
  box-shadow: inset 0px 0px 7px 0px #212c421a;
  border-radius: 12px;
  font-weight: 500;
  height: fit-content;

  .info-label {
    font-size: 22px;
    line-height: 32px;

    &.positive {
      color: $success;
    }
    &.warning {
      color: $warning;
    }
  }

  .info-value {
    font-size: 14px;
    line-height: 20px;
    display: flex;
    align-items: center;
    gap: 4px;

    &.positive {
      color: $success;
    }
    &.warning {
      color: $warning;
    }
  }
}

.theme-dark {
  .market-info-card {
    border-color: $neutral-18;
    box-shadow: inset 0px 0px 7px 0px $neutral-18;
  }
}
</style>
