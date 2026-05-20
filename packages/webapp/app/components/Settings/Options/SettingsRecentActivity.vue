<script lang="ts" setup>
import arrowRight from '~/assets/img/icons/arrow-right.svg?raw'
import stellarExpertLogo from '~/assets/img/stellar-expert-logo.webp'

const recentStore = useRecentActivityStore()
const records = computed(() => recentStore.state.records.slice(0, 100))

const { generateExplorerLink } = useExplorerLink()

function formatDate(iso: string) {
  const date = new Date(iso)
  return date.toLocaleString(undefined, {
    day: '2-digit',
    month: 'short',
    hour: '2-digit',
    minute: '2-digit',
  })
}
</script>

<template>
  <sidebar-panel title="Recent activity">
    <template #trigger>
      <div class="setting-item history">
        <div class="history-label">
          <div class="setting-item__title">
            Recent activity
          </div>
          <i
            class="arrow-right"
            v-html="arrowRight"
          />
        </div>
      </div>
    </template>

    <div
      v-if="records.length > 0"
      class="tx-history-list"
    >
      <div
        v-for="record in records"
        :key="record.transaction_hash"
        class="tx-history-list__item"
      >
        <!-- Explorer icon -->
        <img
          :src="stellarExpertLogo"
          alt="stellar expert"
          class="stellar-expert-logo"
        >

        <!-- Main content -->
        <div class="tx-history-list__content">
          <div class="tx-history-list__title">
            {{ getTxActionLabel(record) }}

            <!-- Status -->
            <i-app-success-circle v-if="record.transaction_successful" />
            <i-app-error-circle v-else />
          </div>
          <div class="tx-history-list__meta">
            {{ formatDate(record.created_at) }}
          </div>
        </div>

        <!-- External link -->
        <a
          :href="generateExplorerLink(record.transaction_hash)"
          target="_blank"
          class="tx-history-list__action"
          rel="noopener noreferrer nofollow"
        >
          <i-app-export-icon color="#111" />
        </a>
      </div>
    </div>

    <div
      v-else
      class="no-recent-activity"
    >
      No recent activity
    </div>
  </sidebar-panel>
</template>

<style lang="scss">
.setting-item.history {
  display: flex;
  flex-direction: column;
  cursor: pointer;
  user-select: none;

  .history {
    &-label {
      display: flex;
      justify-content: space-between;
      align-items: center;
    }
  }

  .tx-history-list {
    padding: $spacing-lg 3px 0 0;
  }

  .arrow-right svg {
    width: 14px;
    height: 14px;

    path {
      stroke: $text-primary;
    }
  }
}

.tx-history-list {
  display: flex;
  flex-direction: column;
  gap: 18px;
  padding: 0 0 $spacing-2xl 0;

  &__item {
    padding: 0;
    margin: 0;
    display: flex;
    justify-content: flex-start;
    align-items: center;
    gap: 12px;
  }

  &__content {
    font-size: 14px;
    line-height: normal;
    user-select: none;
  }

  &__title {
    color: $text-primary;
    font-weight: 500;
    display: flex;
    align-items: center;
    gap: 8px;

    svg {
      width: 16px;
      height: 16px;
    }
  }

  &__meta {
    font-size: 12px;
    color: $text-secondary;
    margin-top: 4px;
  }

  &__action {
    margin-left: auto;

    svg path {
      stroke: $text-primary;
    }
  }

  .stellar-expert-logo {
    width: 14px;
    height: 14px;
    object-fit: contain;
  }
}
.no-recent-activity {
  padding: $spacing-2xl 0;
  color: $navi-50;
  font-size: $text-xs;
  font-style: normal;
  font-weight: 400;
  line-height: 16px;
  text-align: center;
}
</style>
