<script lang="ts" setup>
import type { MultiplyVaultItem } from '~/types/table'

const { vault } = defineProps<{
  vault?: MultiplyVaultItem
}>()

const userStore = useUserStore()

const tabs = computed(() => {
  const dynamicTabs = [{ label: 'Add', value: 'add-position' }]

  if (isUserHaveMultiply(vault)) {
    dynamicTabs.push({ label: 'Close', value: 'close-position' })
  }
  return dynamicTabs
})

const activeTab = ref(tabs.value[0])

watchDebounced(tabs, (t) => {
  if (t.length === 1) {
    activeTab.value = t[0]
  }
}, { debounce: 300 })

function isUserHaveMultiply(vault?: MultiplyVaultItem) {
  if (!vault) { return false }
  return checkIsHaveMultiply(userStore.state.multiplyObligations, [vault] as any, vault.depositPoolData.pool.pool_address, vault.market)
}
</script>

<template>
  <div class="supply-card multiply-supply-card">
    <div class="supply-card__body">
      <div
        v-if="tabs.length > 1"
        class="supply-card-tabs mb-3"
      >
        <div
          v-for="tab in tabs"
          :key="tab.value"
          class="nav-tab"
          :class="[`nav-tab--${tab.value}`, { active: tab.value === activeTab?.value }]"
          @click="activeTab = tab"
        >
          {{ tab.label }}
        </div>
      </div>

      <multiply-window
        v-if="activeTab?.value === 'add-position'"
        :vault="vault"
      />
      <withdraw-multiply-window
        v-if="activeTab?.value === 'close-position'"
        :vault="vault"
        opened
      />
    </div>
  </div>
</template>

<style lang="scss" scoped>
.multiply-supply-card {
  .nav-tab {
    &.nav-tab--add-position {
      &:hover {
        color: $success;
      }
      &.active {
        color: $success;
        background-color: rgba(23, 168, 100, 0.1);
      }
    }
    &.nav-tab--close-position {
      &:hover {
        color: $accent;
      }
      &.active {
        color: $accent;
        background-color: rgba(245, 159, 11, 0.1);
      }
    }
  }

  :deep(.summary-list__item) {
    padding-bottom: 4px;
    .label {
      display: flex;
      flex-direction: column;
      align-items: flex-start;

      .step-id {
        display: flex;
        align-items: center;
        justify-content: center;
        color: $navi-100;
        background-color: $navi-400;
        border-radius: 50%;
        width: 16px;
        height: 16px;
        font-size: 9px;
        font-weight: 700;
      }
    }
    .label-with-tip {
      display: flex;
      align-items: center;
      gap: 6px;
    }
    .sub-label {
      font-size: 11px;
      color: rgb(79, 96, 128);
      margin-left: 18px;

      .line-arrow-icon {
        width: 8px;
        height: 12px;
        margin-right: 4px;
      }
    }

    &:last-child {
      .sub-label {
        margin-left: 24px;
        svg {
          display: none;
        }
      }
    }
  }

  :deep(.select-asset-btn) {
    .asset-icons {
      position: relative;
      width: 24px;
      height: 24px;

      img {
        position: absolute;
        width: 18px;
        height: 18px;

        &:nth-child(1) {
          left: 0;
          top: 0;
        }
        &:nth-child(2) {
          right: -2px;
          bottom: -2px;
        }
      }
    }

    .swap-asset-label {
      white-space: nowrap;
      display: flex;
      align-items: center;
      gap: 4px;

      svg {
        width: 12px;
        height: 12px;
        color: #fff;
      }

      span {
        font-size: 12px;
        font-weight: 500;
      }
    }
  }
}
</style>
