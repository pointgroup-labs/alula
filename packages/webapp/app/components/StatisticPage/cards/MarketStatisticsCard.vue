<script lang="ts" setup>
import type { MarketTableItem } from '~/types/table'

const { pool } = defineProps<{
  pool: MarketTableItem
}>()

const route = useRoute()
const router = useRouter()

const isShowDetails = ref(false)

const totalSupplyUSD = computed(() => pool.total_supply * pool.price)
const totalBorrowUSD = computed(() => pool.total_borrowed * pool.price)

function handleStats() {
  router.push(`${route.path}/${pool.pool_address}`)
}
</script>

<template>
  <div
    class="market-statistic-card"
  >
    <div class="market-statistic-card__top">
      <div class="asset-data">
        <img
          :src="pool.asset?.icon"
          alt="asset icon"
        >
        {{ pool.asset?.symbol }}
      </div>

      <div class="pool-tvl">
        <span>TVL</span> ${{ shortenNumber(totalSupplyUSD, 0, 0) }}
      </div>
    </div>

    <div class="market-statistic-card__body">
      <div
        class="pool-amount-info"
        @click="isShowDetails = !isShowDetails"
      >
        <div class="amount-info">
          <div class="amount-info__title supply-title">
            Supplied
          </div>
          <div class="amount-info__amount">
            ${{ shortenNumber(totalSupplyUSD, 0, 0) }}
          </div>
        </div>

        <div class="amount-info">
          <div class="amount-info__title borrow-title">
            Borrowed
          </div>
          <div class="amount-info__amount">
            ${{ shortenNumber(totalBorrowUSD, 0, 0) }}
          </div>
        </div>

        <i-app-arrow-up
          class="arrow-icon"
          :class="{ 'arrow-icon--active': isShowDetails }"
        />
      </div>

      <statistic-util-rate-chart
        v-if="isShowDetails"
        :pool="pool"
      />

    </div>

    <div class="market-statistic-card__footer">
      <div class="detail-item">
        <div class="detail-item__title">
          Supply APY
        </div>
        <div class="detail-item__value">
          {{ pool.deposit_apy }}
        </div>
      </div>

      <div class="detail-item">
        <div class="detail-item__title">
          Borrow APY
        </div>
        <div class="detail-item__value">
          {{ pool.borrow_apy }}
        </div>
      </div>

      <div class="detail-item">
        <div class="detail-item__title">
          Utilization
        </div>
        <div class="detail-item__value">
          {{ pool.utilization_rate }}
        </div>
      </div>

      <div class="detail-item">
        <div class="detail-item__title">
          Available
        </div>
        <div class="detail-item__value">
          ${{ shortenNumber(pool.available * pool.price, 0, 0) }}
        </div>
      </div>
    </div>

    <div class="market-statistic-card__action">
      <j-btn
        size="xs"
        variant="outlined-brand"
        @click="handleStats"
      >
        All stats <i-app-export-icon />
      </j-btn>
    </div>
  </div>
</template>
