<script lang="ts" setup>
import type { RewardsTableItem } from '~/types/table'

const { width } = useWindowSize()

const farmsStore = useFarmsStore()

const preparedRewards = computed(() => farmsStore.preparedRewards)
const totalRewardsUsd = computed(() => preparedRewards.value?.reduce((acc, reward) => acc + reward.amountUsd, 0))

const isLoading = computed(() => farmsStore.state.loadingRewards && totalRewardsUsd.value === 0)

const isClaiming = computed(() => farmsStore.state.claiming)

const fields = [
  { key: 'asset', label: 'Asset', align: 'left' },
  { key: 'market', label: 'Market', align: 'left' },
  { key: 'strategy', label: 'Strategy', align: 'left' },
  { key: 'pending', label: 'Pending', align: 'right' },
  { key: 'action', label: '', thClass: 'profile-action', tdClass: 'profile-action' },
]

const items: ComputedRef<RewardsTableItem[]> = computed(() => {
  return preparedRewards.value?.map((r) => {
    return {
      asset: r.asset,
      market: r.market,
      strategy: r.strategyType!,
      pending: {
        amount: r.amount,
        usd: r.amountUsd,
      },
      farmId: r.farmId,
      rewardIndex: r.rewardIndex,
    }
  }) ?? []
})

async function claim(data: RewardsTableItem) {
  await farmsStore.claim(data)
}

function isDisabled(farmId: string): boolean {
  const claimId = farmsStore.state.claimFarmId
  return !!(claimId && claimId !== farmId)
}
</script>

<template>
  <div class="portfolio-card rewards-card">
    <div class="portfolio-card__title">
      Rewards

      <metric-indicator
        v-if="totalRewardsUsd > 0"
        label="Total Rewards"
        :value="`${formatCompactUSD(totalRewardsUsd, 2, 2)}`"
        color="#17B26A"
      />
    </div>

    <div v-if="isLoading">
      <rewards-table-skeleton />
    </div>

    <div
      v-else
      class="table-wrapper"
    >
      <template v-if="items.length > 0">
        <BTable
          v-if="width >= 1024"
          borderless
          :fields="fields"
          :items="items"
          responsive
          class="portfolio-table market-table"
          :class="{ 'table-loading': farmsStore.state.loadingRewards }"
        >
          <template
            v-for="field in fields"
            :key="field.key"
            #[`head(${field.key})`]="data"
          >
            <span :style="{ '--align': field.align }">{{ data.label }}</span>
          </template>

          <template #cell(asset)="data">
            <div class="market-table__asset">
              <img
                :src="data.item.asset?.icon"
                alt=""
              >
              <div class="market-table__asset__info">
                <div class="market-table__asset__info__name">
                  {{ data.item.asset?.symbol }}
                </div>
              </div>
            </div>
          </template>

          <template #cell(market)="data">
            <div class="table-cell justify-content-start text-capitalize">
              {{ data.item.market }}
            </div>
          </template>

          <template #cell(strategy)="data">
            <div class="table-cell justify-content-start text-capitalize">
              {{ data.item.strategy }}
            </div>
          </template>

          <template #cell(pending)="data">
            <div class="table-cell justify-content-end with-price">
              {{ Number(data.item.pending.amount) > 1000 ? shortenNumber(Number(data.item.pending.amount)) : Number(data.item.pending.amount).toFixed(5) }}
              <span>{{ formatCompactUSD(data.item.pending.usd, 2, 2) }}</span>
            </div>
          </template>

          <template #cell(action)="data">
            <div class="table-cell justify-content-end">
              <j-btn
                variant="outlined-positive"
                size="sm"
                :loading="isClaiming && !isDisabled(data.item.farmId)"
                :disabled="isDisabled(data.item.farmId)"
                @click="claim(data.item)"
              >
                Claim
              </j-btn>
            </div>
          </template>
        </BTable>

        <portfolio-rewards-table-mobile
          v-else
          :items="items"
          @dialog-handler="(e) => claim(e.item)"
        />
      </template>

      <div
        v-else
        class="no-data"
      >
        <i-app-reward-icon style="width: 30px; height: 30px" />
        No rewards
      </div>
    </div>
  </div>
</template>

<style lang="scss">
.rewards-card {
  max-width: 50%;
  margin-top: $spacing-2xl;

  @media (max-width: $breakpoint-md) {
    max-width: 100%;
  }

  .profile-action {
    padding-left: $spacing-3xl;
  }
}
</style>
