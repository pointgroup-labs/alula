<script lang="ts" setup>
import logoDark from '~/assets/img/logo-dark.svg'
import logoLight from '~/assets/img/logo-light.svg'

const appLogo = computed(() => isDark.value ? logoDark : logoLight)
</script>

<template>
  <div class="market-info__bottom">
    <market-interest />
    <market-interest-chart />
    <div class="separator-vert hide-xs" />
    <div class="logo-with-text hide-xs">
      <img
        :src="appLogo"
        alt="app logo"
      >
      <span>
        Stellar Lending
      </span>
    </div>
  </div>
</template>

<style lang="scss">
.market-info-dialog {
  .market-info__bottom {
    display: flex;
    gap: $spacing-16;

    @media (max-width: $breakpoint-xs) {
      flex-direction: column;
      gap: $spacing-8;
    }

    .logo-with-text {
      flex: 1;
      display: flex;
      flex-direction: column;
      align-items: center;
      justify-content: center;
      font-size: 16px;
      font-style: normal;
      font-weight: 700;
      line-height: 20px;

      img {
        height: auto;
        width: 152px;
        object-fit: contain;
      }

      span {
        margin-top: -4px;
      }
    }
  }
}
</style>
