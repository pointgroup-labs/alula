<script lang="ts" setup>
import logoDark from '~/assets/img/logo-with-text-dark.svg'
import logoLight from '~/assets/img/logo-with-text-light.svg'

const { isSidebar } = defineProps<{
  isSidebar: boolean
}>()

const emit = defineEmits(['close'])

const { width } = useWindowSize()

const isShowMobileElements = computed(() => width.value < 1024)

const logoWithText = computed(() => isDark.value ? logoDark : logoLight)

function close() {
  emit('close')
}
</script>

<template>
  <sidebar
    :is-sidebar="isSidebar"
    :title="$t('common.settings')"
    class-name="settings-sidebar"
    @close="close"
  >
    <settings-connect />
    <settings-navigation v-if="isShowMobileElements" />
    <market-info v-if="isShowMobileElements" />
    <settings-language />
    <settings-theme />

    <client-only>
      <img
        :src="logoWithText"
        alt="JLend logo"
        class="sidebar-logo"
      >
    </client-only>
  </sidebar>
</template>

<style lang="scss">
.settings-sidebar {
  .sidebar-body {
    min-height: calc(100% - 36px);
    display: flex;
    flex-direction: column;
    gap: $spacing-24;
    padding-top: $spacing-24;
  }

  .setting-item__title {
    font-size: 16px;
    font-style: normal;
    font-weight: 700;
    line-height: 20px;
  }

  .sidebar-logo {
    width: 152px;
    height: 52px;
    object-fit: contain;
    margin: auto 0 0 auto;
  }

  .market-info {
    gap: $spacing-12;

    .market-size {
      display: none;
    }
    .total-card {
      padding: $spacing-16;
      width: 100%;
      height: auto;

      &:before {
        display: none;
      }

      &__title {
        font-size: 12px;
        line-height: 16px;
      }

      &__body {
        font-size: 20px;
        line-height: 20px;
      }

      &__icon {
        display: none;
      }
    }
  }
}

body.body--dark {
}
</style>
