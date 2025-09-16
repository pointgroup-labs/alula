<script lang="ts" setup>
const { isSidebar } = defineProps<{
  isSidebar: boolean
}>()

const emit = defineEmits(['close'])

const { width } = useWindowSize()

const isShowMobileElements = computed(() => width.value < 1024)

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
    <settings-navigation
      v-if="isShowMobileElements"
      @close="close"
    />
    <market-info v-if="isShowMobileElements" />
    <settings-language />
    <settings-theme />
    <settings-network />

    <faucet-menu />

    <logo-with-text />
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

  .logo-with-text {
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
