<script lang="ts" setup>
import logoDark from '~/assets/img/logo-dark.svg'
import logoLight from '~/assets/img/logo-light.svg'

const logo = computed(() => isDark.value ? logoDark : logoLight)
</script>

<template>
  <client-only>
    <div class="logo-with-text">
      <img
        :src="logo"
        alt="JLend logo"
      >
      <span>Stellar Lending</span>
    </div>
  </client-only>
</template>

<style lang="scss">
  .logo-with-text {
  display: flex;
  flex-direction: column;
  align-items: center;
  font-size: 16px;
  font-style: normal;
  font-weight: 700;
  line-height: 20px;

  img {
    width: 152px;
    height: 52px;
    object-fit: contain;
  }

  span {
    margin-top: -4px;
  }
}
</style>
