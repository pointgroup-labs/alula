<script lang="ts" setup>
const tabs = [
  { label: 'Supply', value: 'supply' },
  { label: 'Borrow', value: 'borrow' },
]

const activeTab = ref(tabs[0])

const dialog = ref(false)

function dialogHandler() {
  dialog.value = !dialog.value
}
</script>

<template>
  <div class="supply-card">
    <div class="supply-card__body">
      <div class="supply-card-tabs">
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

      <supply-window
        v-if="activeTab?.value === 'supply'"
        @dialog-handler="dialogHandler"
      />
      <borrow-window
        v-else-if="activeTab?.value === 'borrow'"
        @dialog-handler="dialogHandler"
      />

      <change-pool-dialog v-model="dialog" />
    </div>
  </div>
</template>

<style lang="scss">
.supply-card {
  max-width: 400px;
  width: 100%;
  height: fit-content;
  background-color: color-mix(in oklab, $card 50%, transparent);
  padding: 20px;
  border: 1px solid $border-color;
  border-radius: $radius-2xl;

  .info-card {
    background-color: color-mix(in oklab, $new-secondary 30%, transparent);
    border: 1px solid $border-color;
    border-radius: $radius-2xl;
    transition: border-color 0.2s ease;
    padding: 16px;
  }

  .supply-card-tabs {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 4px;
    border-radius: $radius-2xl;
    background-color: color-mix(in oklab, $new-secondary 60%, transparent);

    .nav-tab {
      font-size: 14px;
      padding: 10px 0;
      width: 100%;
      border-radius: $radius-2xl;
      color: $text-tertiary;
      text-align: center;
      cursor: pointer;
      transition: all 0.2s ease;

      &--supply {
        &:hover {
          color: $supply;
        }
      }
      &--borrow {
        &:hover {
          color: $purple;
        }
      }

      &.active {
        &.nav-tab--supply {
          color: $supply;
          background-color: rgba(0, 211, 238, 0.15);
        }

        &.nav-tab--borrow {
          color: $purple;
          background-color: rgba(99, 102, 241, 0.15);
        }
      }
    }
  }

  .select-pool-btn {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 14px;
    color: $text-primary;
    background-color: color-mix(in oklab, #1a2236 60%, transparent);
    padding: 8px 12px;
    border-radius: $radius-lg;
    cursor: pointer;

    &:hover {
      background-color: color-mix(in oklab, #1a2236 90%, transparent);
    }

    img {
      width: 24px;
      height: 24px;
      object-fit: contain;
      border-radius: 50%;
    }

    svg {
      width: 7px;
      color: $text-tertiary;
    }
  }

  .collateral {
    display: flex;
    align-items: center;
    justify-content: space-between;
    font-size: 14px;
    color: $text-tertiary;
  }

  .info-supply {
    background-color: var(--bg-color);
    border-color: var(--border-color);

    &__header {
      display: flex;
      align-items: center;
      justify-content: space-between;
      margin-bottom: 12px;

      .info-title {
        font-size: 12px;
        color: $text-tertiary;
        display: flex;
        align-items: center;
        gap: 8px;

        &::before {
          content: '';
          width: 8px;
          height: 8px;
          border-radius: 50%;
          background-color: var(--color);
          display: block;
        }
      }

      .info-apy {
        color: var(--color);
        font-family: $font-JetBrainsMono;
        font-weight: 700;
        font-size: 20px;
      }
    }

    &__body {
      display: flex;
      align-items: center;
      justify-content: space-between;
      gap: 16px;

      .info-detail {
        width: 100%;
        display: flex;
        flex-direction: column;
        align-items: flex-start;
        overflow: hidden;

        &__title {
          font-size: 10px;
          color: $text-tertiary;
          opacity: 0.7;
        }

        &__value {
          width: 100%;
          font-family: $font-JetBrainsMono;
          font-size: 14px;
          font-weight: 600;
          color: $text-primary;
          overflow: hidden;
          text-overflow: ellipsis;
        }
      }
    }
  }

  .info-summary {
    padding: 0;

    &__header {
      font-size: 11px;
      color: $text-tertiary;
      padding: 10px 16px;
      border-bottom: 1px solid $border-color;
      display: flex;
      align-items: center;
      justify-content: space-between;
    }

    .summary-list {
      padding: 16px;
      display: flex;
      flex-direction: column;
      gap: 12px;

      &__item {
        height: 16px;
        display: flex;
        align-items: center;
        justify-content: space-between;
        font-size: 12px;
        color: $text-primary;

        .label {
          color: $text-tertiary;
        }

        .value {
          font-family: $font-JetBrainsMono;
          opacity: 0.8;
        }
      }
    }
  }
}

.summary-slide-enter-active {
  animation: summary-in 0.3s cubic-bezier(0.16, 1, 0.3, 1);
}

.summary-slide-leave-active {
  animation: summary-in 0.2s cubic-bezier(0.16, 1, 0.3, 1) reverse;
}

@keyframes summary-in {
  from {
    opacity: 0;
    transform: translateY(-8px);
    max-height: 0;
  }
  to {
    opacity: 1;
    transform: translateY(0);
    max-height: 400px;
  }
}
</style>
