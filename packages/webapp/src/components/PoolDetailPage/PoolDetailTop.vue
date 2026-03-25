<script lang="ts" setup>
import type { MarketTableItem } from '~/types/table'
import { bpsToNumber } from '@alula/client-sdk'

const { width } = useWindowSize()

const selectedPool = inject('selectedPool') as Ref<MarketTableItem>

const pool = computed(() => selectedPool.value?.raw?.pool)

const asset = computed(() => selectedPool?.value?.asset)

// const {
//   poolBorrowLimit,
// } = useBorrowDialog(selectedPool, toRef(false))

const price = computed(() => selectedPool.value?.price ?? 0)

const borrowCapacity = computed(() => {
  if (!pool.value) {
    return 0
  }
  const utilRatePercent = selectedPool.value?.utilization_rate_percent
  const utilLimit = bpsToNumber(Number(selectedPool.value?.raw?.pool?.config.health_config.utilization_ratio_limit_bps) || 0) * 100
  return utilRatePercent / utilLimit * 100
})

// const utilRateColor = computed(() => {
//   const currentUtil = selectedPool.value?.utilization_rate_percent ?? 0
//   const limitUtil
//     = bpsToNumber(
//       Number(selectedPool.value?.raw?.pool?.config.health_config.utilization_ratio_limit_bps) || 0,
//     ) * 100

//   if (!limitUtil) {
//     return '#e8edf5'
//   }
//   const capacityUsed = (currentUtil / limitUtil) * 100
//   if (capacityUsed >= 90) {
//     return '#f04438'
//   }
//   if (capacityUsed >= 70) {
//     return '#f79009'
//   }
//   return '#e8edf5'
// })

const detailCardsData = computed(() => {
  if (!pool.value) {
    return {
      depositApy: '0.00%',
      borrowAPY: '0.00%',
      supplied: '0.00',
      utilRate: '0.00%',
    }
  }
  const depositApy = selectedPool.value?.deposit_apy ?? '0.00%'
  const borrowAPY = selectedPool.value?.borrow_apy ?? '0.00%'
  return {
    depositApy,
    borrowAPY,
    supplied: selectedPool?.value?.total_supply ?? 0,
    utilRate: `${truncatePercent(borrowCapacity.value, 2)}%`,
  }
})
</script>

<template>
  <div class="market-details-top">
    <back-btn />

    <div
      v-if="asset"
      class="asset-data"
    >
      <img
        :src="asset?.icon"
        alt="asset icon"
      >
      <div class="asset-data__coin">
        <span data="name">{{ asset?.symbol }}</span>
        <span data="symbol">{{ asset?.name }}</span>
      </div>
    </div>

    <template v-if="selectedPool">
      <div class="market-pill">
        {{ selectedPool?.market ?? '' }} Market
      </div>

      <div
        v-if="width > 650"
        class="market-pill"
      >
        <div class="market-name">
          Price: <span class="text-num"> ${{ formatPrice(price, 2, 2) }}</span>
        </div>
      </div>
    </template>

    <div class="pool-metrics">
      <div class="pool-metrics__item">
        <span>Supply APY
          <info-tooltip>
            Estimated annual yield earned by supplying assets.
            <br>
            Rates adjust dynamically based on pool utilization.
          </info-tooltip>
        </span>
        <span class="supply">{{ detailCardsData.depositApy }}</span>
      </div>

      <div class="separator-vert" />

      <div class="pool-metrics__item">
        <span>Borrow APY
          <info-tooltip>
            Annual interest rate paid for borrowing assets.
            <br>
            Rates increase as pool utilization grows
          </info-tooltip>
        </span>
        <span class="borrow">{{ detailCardsData.borrowAPY }}</span>
      </div>

      <!-- <div class="separator-vert" /> -->

      <!-- <div class="pool-metrics__item">
        <span>Supplied
          <info-tooltip>
            Total amount of assets supplied to this pool by all users.
            <br>
            These assets can be borrowed and generate yield for suppliers.
          </info-tooltip>
        </span>
        <span>{{ shortenNumber(detailCardsData.supplied) }}</span>
      </div> -->

      <!-- <div class="separator-vert" /> -->

      <!-- <div class="pool-metrics__item">
        <span>Available Liquidity
          <info-tooltip>
            Amount of assets currently available to borrow from this pool.
            <br>
            Decreases as users borrow and increases when they repay.
          </info-tooltip>
        </span>
        <span>{{ shortenNumber(poolBorrowLimit) }}</span>
      </div> -->

      <!-- <div class="separator-vert" /> -->

      <!-- <div class="pool-metrics__item">
        <span>Borrow Capacity
          <info-tooltip>
            Percentage of the pool's borrow limit currently used.
            <br>
            At 100% no additional borrowing is possible
          </info-tooltip>
        </span>
        <span :style="{ color: utilRateColor }">{{ detailCardsData.utilRate }}</span>
      </div> -->
    </div>
  </div>
</template>
