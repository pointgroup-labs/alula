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
    <div class="settings-sidebar__options">
      <settings-connect />
    </div>

    <div
      v-if="isShowMobileElements"
      class="settings-sidebar__options"
    >
      <div class="option-title">
        Navigations
      </div>
      <settings-navigation
        @close="close"
      />

      <markets-info />
    </div>

    <div class="settings-sidebar__options">
      <div class="option-title">
        Options
      </div>
      <settings-language />
      <settings-theme />
    </div>

    <div class="settings-sidebar__options">
      <div class="option-title">
        Network
      </div>
      <settings-network />
      <settings-recent-activity />
    </div>

    <faucet-menu />

  </sidebar>
</template>

<style lang="scss">
.settings-sidebar {
  .sidebar-panel-view {
    gap: 24px;
  }

  .sidebar-body {
    min-height: calc(100% - 36px);
    display: flex;
    flex-direction: column;
    gap: $spacing-24;
    padding-top: $spacing-24;
  }

  .setting-item__title {
    color: $text-primary;
    font-size: 16px;
    font-style: normal;
    font-weight: 500;
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

  .settings-sidebar__options {
    position: relative;
    display: flex;
    flex-direction: column;
    gap: 30px;

    &:not(:last-child) {
      &::after {
        content: '';
        width: calc(100% + 48px);
        margin-left: -24px;
        height: 1px;
        background-color: $surface-neutral-08;
        display: block;
      }
    }

    .option-title {
      color: rgba(216, 232, 238, 0.8);
      font-size: 22px;
      margin-bottom: -10px;
      font-weight: 700;
    }
  }
}
</style>
