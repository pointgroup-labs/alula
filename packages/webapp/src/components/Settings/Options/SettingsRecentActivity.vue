<script lang="ts" setup>
import arrowRight from '~/assets/img/icons/arrow-right.svg?raw'
import stellarExpertLogo from '~/assets/img/stellar-expert-logo.webp'

const recentStore = useRecentActivityStore()
const records = computed(() => recentStore.state.records)

const subMenu = ref(false)
const { generateExplorerLink } = useExplorerLink()

function menuHandler() {
  subMenu.value = !subMenu.value
}

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
  <div
    class="setting-item history"
    @click="menuHandler"
  >
    <div class="history-label">
      <div class="setting-item__title">
        Recent activity
      </div>

      <i
        class="arrow-right"
        v-html="arrowRight"
      />
    </div>

    <div class="history-preview">
      <div
        v-if="records.length > 0"
        class="tx-history-list"
      >
        <div
          v-for="record in records[0] ? records.slice(0, 3) : []"
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
            @click.stop
          >
            <i-app-export-icon color="#111" />
          </a>
        </div>
      </div>
    </div>
  </div>

  <sidebar-sub-menu
    :is-sub-menu="subMenu"
    title="Recent activity"
    class="renect-activity-submenu"
    @close="menuHandler"
  >
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
  </sidebar-sub-menu>
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
    padding: $spacing-12 3px 0 0;
  }

  .arrow-right svg {
    width: 20px;
    height: 20px;

    path {
      stroke: $neutral-6;
    }
  }
}

.tx-history-list {
  display: flex;
  flex-direction: column;
  gap: $spacing-12;
  padding: $spacing-12 0 $spacing-20 0;

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
    color: $neutral-6;
  }

  &__action {
    margin-left: auto;
  }

  .stellar-expert-logo {
    width: 20px;
    height: 20px;
    object-fit: contain;
  }
}
.renect-activity-submenu {
  .no-recent-activity {
    text-align: center;
    padding: $spacing-20 0;
  }
}

.theme-dark {
  .tx-history-list__action svg {
    color: #fff;
  }
}
</style>
