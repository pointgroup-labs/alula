<script lang="ts" setup>
import type { MultiplyAccountTableItem } from '~/types/table'
import { bpsToNumber } from '@alula/client-sdk'
import { calculateBorrow, calculateTotalStake } from '@alula/client-sdk/src/utils'
import { amountToUsdWithShort, calculateCurrentMultiplier, formatPrice, truncatePercent } from '~/utils'

type MultiplyAccountItemWithStats = MultiplyAccountTableItem & {
  healthFactor: number
  liquidationPrice: number
  softLiquidationPrice: number
  safetyBuffer: number
}

const marketsStore = useMarketsStore()
const userStore = useUserStore()
const market = useMarketActions()

const poolData = inject<any>('selectedPool')

const selectedPoolAddress = toRef(marketsStore, 'selectedPoolAddress')
const dialogLeverageWithdraw = toRef(marketsStore, 'dialogLeverageWithdraw')

const isLoading = computed(() => (marketsStore.state.loadingLeveragePools || marketsStore.state.loading) || userStore.loading)

const userPositions = computed<MultiplyAccountItemWithStats[]>(() => {
  const res = []
  const currentMarket = poolData?.value?.market

  if (!currentMarket) {
    return []
  }

  const state = marketsStore.state.markets[currentMarket]?.marketState
  const poolsData = state?.pools_data ?? []
  const oraclePriceDecimals = state?.oracle_price_decimals ?? 0
  const assetDecimals = state?.asset_decimals ?? 7
  const multiplyObligations = userStore.state.multiplyObligations[currentMarket]

  const depositPoolAddress = poolData?.value?.depositPoolData?.pool?.pool_address
  const borrowPoolAddress = poolData?.value?.borrowPoolData?.pool?.pool_address

  if (!depositPoolAddress || !borrowPoolAddress) {
    return []
  }

  const depositObligation = multiplyObligations?.deposits?.find(([address]) => address === depositPoolAddress)
  const borrowObligation = multiplyObligations?.borrows?.find(([address]) => address === borrowPoolAddress)
  const depositPoolData = poolsData.find(p => p.pool.pool_address === depositPoolAddress)
  const borrowPoolData = poolsData.find(p => p.pool.pool_address === borrowPoolAddress)

  if (!depositObligation || !borrowObligation || !depositPoolData || !borrowPoolData) {
    return []
  }

  const [, depOblData] = depositObligation
  const [, borrowOblData] = borrowObligation

  const supplyBPS = bpsToNumber(Number(depositPoolData?.apy.supply_bps || 0))
  const borrowBPS = bpsToNumber(Number(borrowPoolData?.apy.borrow_bps || 0))
  const ltv = Number(depositPoolData?.pool.config.health_config.open_ltv_bps) || 0
  const multiplier = calculateMaxMultiplierFromBps(ltv)
  const maxAPY = (supplyBPS * multiplier - borrowBPS * (multiplier - 1)) * 100

  const deposited = +calculateTotalStake(depOblData.j_tokens, {
    total_j_tokens: depositPoolData.pool.total_j_tokens,
    total_borrowed: depositPoolData.pool.total_borrowed,
    total_available: depositPoolData.total_available_adjusted,
  }) || 0

  const borrowed = +calculateBorrow(borrowOblData.d_tokens, {
    total_borrowed: borrowPoolData.pool.total_borrowed,
    total_d_tokens: borrowPoolData.pool.total_d_tokens,
  }, assetDecimals) || 0

  const currentPrice = Number(bigintToNumber(depositPoolData.oracle_asset_price, oraclePriceDecimals)) || 0
  const borrowPoolPrice = Number(bigintToNumber(borrowPoolData.oracle_asset_price, oraclePriceDecimals)) || 0

  const currentMultiplier = calculateCurrentMultiplier(deposited, currentPrice, borrowed, borrowPoolPrice) || 0

  const collateralValue = deposited * currentPrice
  const borrowValue = borrowed * borrowPoolPrice

  const liquidationLtv = bpsToNumber(Number(depositPoolData?.pool.config.health_config.close_ltv_bps || 0))
  const maxBorrowValueAtLiquidation = collateralValue * liquidationLtv
  const healthFactor = borrowValue > 0 ? (maxBorrowValueAtLiquidation / borrowValue) * 100 : 0

  const liquidationPrice = deposited > 0 ? (borrowed * borrowPoolPrice) / (deposited * liquidationLtv) : 0

  const softLiquidationPrice = liquidationPrice * 1.1

  const safetyBuffer = (currentPrice / liquidationPrice - 1) * 100

  const data = {
    market: currentMarket,
    depositPoolData,
    borrowPoolData,
    asset: getFullTokenData(depositPoolData?.pool.token_symbol),
    borrowAsset: getFullTokenData(borrowPoolData?.pool.token_symbol),
    deposited,
    borrowed,
    multiplier: currentMultiplier,
    maxAPY,
    price: currentPrice,
    borrowPoolPrice,
    pool_address: depositPoolData?.pool.pool_address || '',
    assetDecimals,
    healthFactor,
    liquidationPrice,
    softLiquidationPrice,
    safetyBuffer,
  }

  res.push(data)

  return res
})

const hasPosition = computed(() => userPositions.value.length > 0)

function getHealthColor(health: number) {
  if (health >= 150) {
    return 'safe'
  }
  if (health >= 120) {
    return 'warning'
  }
  return 'danger'
}

function openWithdrawDialog() {
  const item = userPositions.value[0]
  if (!item) {
    return
  }
  selectedPoolAddress.value = item.pool_address
  marketsStore.activeLeverageMarket = String(item.market)
  dialogLeverageWithdraw.value = true
}
</script>

<template>
  <section id="multiply-position">
    <div class="position-header">
      <h2>My Position</h2>
      <p class="position-subtitle">
        Manage your leveraged position
      </p>
    </div>

    <div
      v-if="isLoading"
      class="loading-state"
    >
      <j-loading-spinner>
        Loading your position...
      </j-loading-spinner>
    </div>

    <div
      v-else-if="!hasPosition"
      class="empty-state"
    >
      <div class="empty-state-card">
        <div class="empty-icon">
          <i-app-chart-square-icon />
        </div>
        <h3>No Active Position</h3>
        <p>You don't have any multiply position yet.</p>
        <p class="empty-hint">
          Use the form to create your first leveraged position and start multiplying your exposure.
        </p>
      </div>
    </div>

    <div
      v-else
      class="position-content"
    >
      <div class="position-cards">
        <div
          v-for="position in userPositions"
          :key="position.pool_address"
          class="position-card"
          :class="`health-${getHealthColor(position.healthFactor)}`"
        >
          <div class="position-card-header">
            <div class="position-pair">
              <div class="pair-icons">
                <img
                  :src="position.asset.icon"
                  :alt="position.asset.symbol"
                  class="icon-primary"
                >
                <img
                  :src="position.borrowAsset.icon"
                  :alt="position.borrowAsset.symbol"
                  class="icon-secondary"
                >
              </div>
              <div class="pair-info">
                <div class="pair-name">{{ position.asset.symbol }}/{{ position.borrowAsset.symbol }}</div>
                <div class="pair-label">{{ position.asset.name }}</div>
              </div>
            </div>

            <div
              class="health-badge"
              :class="`health-${getHealthColor(position.healthFactor)}`"
            >
              <div class="health-label">Health</div>
              <div class="health-value">{{ truncatePercent(position.healthFactor, 0) }}%</div>
            </div>
          </div>

          <div class="health-bar-container">
            <div class="health-bar-wrapper">
              <div class="health-bar">
                <div class="health-zone danger-zone" />
                <div class="health-zone warning-zone" />
                <div class="health-zone safe-zone" />

                <div
                  class="health-threshold"
                  style="left: 60%"
                >
                  <div class="threshold-line" />
                  <div class="threshold-label">120%</div>
                </div>
                <div
                  class="health-threshold"
                  style="left: 75%"
                >
                  <div class="threshold-line" />
                  <div class="threshold-label">150%</div>
                </div>

                <div
                  class="health-indicator"
                  :class="`health-${getHealthColor(position.healthFactor)}`"
                  :style="{ left: `${Math.min((position.healthFactor / 200) * 100, 100)}%` }"
                >
                  <div class="indicator-dot" />
                  <div class="indicator-value">{{ truncatePercent(position.healthFactor, 0) }}%</div>
                </div>
              </div>

              <div class="health-labels">
                <span class="label-start">0%</span>
                <span class="label-end">200%+</span>
              </div>
            </div>
          </div>

          <div class="position-card-body">
            <div class="position-metrics">
              <div class="metric-row">
                <div class="metric-item">
                  <div class="metric-label">
                    <span>Collateral</span>
                    <j-tooltip>
                      <i-app-info-circle class="info-icon" />
                      <template #content>
                        Your deposited collateral
                      </template>
                    </j-tooltip>
                  </div>
                  <div class="metric-value">
                    <div class="value-primary">{{ formatPrice(position.deposited, 2, 4) }} {{ position.asset.symbol }}</div>
                    <div class="value-secondary">${{ amountToUsdWithShort(position.deposited, position.price) }}</div>
                  </div>
                </div>

                <div class="metric-item">
                  <div class="metric-label">
                    <span>Debt</span>
                    <j-tooltip>
                      <i-app-info-circle class="info-icon" />
                      <template #content>
                        Your borrowed amount
                      </template>
                    </j-tooltip>
                  </div>
                  <div class="metric-value">
                    <div class="value-primary">{{ formatPrice(position.borrowed, 2, 4) }} {{ position.borrowAsset.symbol }}</div>
                    <div class="value-secondary">${{ amountToUsdWithShort(position.borrowed, position.borrowPoolPrice) }}</div>
                  </div>
                </div>
              </div>

              <div class="metric-row">
                <div class="metric-item highlight">
                  <div class="metric-label">Current Multiplier</div>
                  <div class="metric-value">
                    <j-pill-label
                      color="#111"
                      variant="success"
                      size="md"
                    >
                      {{ truncatePercent(position.multiplier, 2) }}x
                    </j-pill-label>
                  </div>
                </div>

                <div class="metric-item highlight">
                  <div class="metric-label">Net APY</div>
                  <div class="metric-value">
                    <span :class="position.maxAPY > 0 ? 'apy-positive' : 'apy-negative'">
                      {{ truncatePercent(position.maxAPY, 2) }}%
                    </span>
                  </div>
                </div>
              </div>

              <div class="metric-row">
                <div class="metric-item">
                  <div class="metric-label">Soft Liquidation Price</div>
                  <div class="metric-value">
                    <span class="liquidation-price">${{ formatPrice(position.softLiquidationPrice, 2, 4) }}</span>
                  </div>
                </div>

                <div class="metric-item">
                  <div class="metric-label">Safety Buffer</div>
                  <div class="metric-value">
                    <span
                      class="buffer-value"
                      :class="`buffer-${getHealthColor(position.safetyBuffer + 100)}`"
                    >{{ truncatePercent(position.safetyBuffer, 0) }}%</span>

                    <risk-warning :buffer="position.safetyBuffer" />
                  </div>
                </div>
              </div>

              <div class="metric-row">
                <div class="metric-item">
                  <div class="metric-label">Liquidation Price</div>
                  <div class="metric-value">
                    <span class="liquidation-price">${{ formatPrice(position.liquidationPrice, 2, 4) }}</span>
                  </div>
                </div>

                <div class="metric-item">
                  <div class="metric-label">Current Price</div>
                  <div class="metric-value">
                    <span class="value-primary">${{ formatPrice(position.price, 2, 4) }}</span>
                  </div>
                </div>
              </div>
            </div>
          </div>

          <div class="position-card-footer">
            <j-btn
              variant="accent"
              size="md"
              pill
              icon-right
              :disabled="market.isDisabled(position.pool_address, 'withdrawLeverage', position.market!)"
              :loading="market.isLoading(position.pool_address, 'withdrawLeverage', position.market!)"
              @click="openWithdrawDialog"
            >
              Manage Position
            </j-btn>
          </div>
        </div>
      </div>
    </div>

    <withdraw-leverage-dialog
      v-model="dialogLeverageWithdraw"
      :data="userPositions[0]"
    />
  </section>
</template>

<style lang="scss">
section#multiply-position {
  .position-header {
    margin-bottom: $spacing-4xl;

    h2 {
      font-size: 24px;
      font-weight: 600;
      margin: 0 0 $spacing-md 0;
      color: #262729;
      letter-spacing: -0.02em;
    }

    .position-subtitle {
      font-size: 14px;
      color: #8a8b8d;
      margin: 0;
    }
  }

  .loading-state {
    display: flex;
    justify-content: center;
    align-items: center;
    padding: 80px 20px;
  }

  .empty-state {
    display: flex;
    justify-content: center;
    padding: 40px 20px;

    .empty-state-card {
      max-width: 480px;
      text-align: center;
      padding: 48px 32px;
      background: linear-gradient(135deg, rgba(255, 255, 255, 0.98) 0%, rgba(249, 250, 251, 0.99) 100%);
      border: 1px solid rgba(33, 44, 66, 0.08);
      border-radius: $radius-3xl;
      box-shadow:
        0px 2px 8px rgba(33, 44, 66, 0.04),
        0px 8px 24px rgba(33, 44, 66, 0.06);

      .empty-icon {
        width: 80px;
        height: 80px;
        margin: 0 auto $spacing-3xl;
        background: linear-gradient(135deg, rgba(92, 108, 255, 0.1) 0%, rgba(92, 108, 255, 0.05) 100%);
        border-radius: 50%;
        display: flex;
        align-items: center;
        justify-content: center;

        svg {
          width: 40px;
          height: 40px;
          color: #5c6cff;
        }
      }

      h3 {
        font-size: 20px;
        font-weight: 600;
        margin: 0 0 $spacing-lg 0;
        color: #262729;
      }

      p {
        font-size: 14px;
        color: #8a8b8d;
        margin: 0 0 $spacing-md 0;
      }

      .empty-hint {
        font-size: 13px;
        color: #aaabad;
        margin-top: $spacing-xl;
      }
    }
  }

  .position-content {
    display: flex;
    flex-direction: column;
    gap: 24px;
  }

  .position-cards {
    display: flex;
    flex-direction: column;
    gap: 20px;
  }

  .position-card {
    background: linear-gradient(135deg, rgba(255, 255, 255, 0.98) 0%, rgba(249, 250, 251, 0.99) 100%);
    border: 2px solid rgba(33, 44, 66, 0.08);
    border-radius: $radius-3xl;
    padding: 0;
    overflow: hidden;
    box-shadow:
      0px 2px 8px rgba(33, 44, 66, 0.04),
      0px 8px 24px rgba(33, 44, 66, 0.06);
    transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);

    &.health-danger {
      border-color: rgba(251, 71, 71, 0.3);
      background: linear-gradient(135deg, rgba(251, 71, 71, 0.02) 0%, rgba(255, 255, 255, 0.99) 100%);
    }

    &.health-warning {
      border-color: rgba(255, 183, 38, 0.3);
      background: linear-gradient(135deg, rgba(255, 183, 38, 0.02) 0%, rgba(255, 255, 255, 0.99) 100%);
    }

    &.health-safe {
      border-color: rgba(8, 181, 118, 0.2);
    }
  }

  .position-card-header {
    padding: 24px 28px 20px;
    background: linear-gradient(135deg, rgba(92, 108, 255, 0.04) 0%, rgba(92, 108, 255, 0.01) 100%);
    border-bottom: 1px solid rgba(33, 44, 66, 0.06);
    display: flex;
    justify-content: space-between;
    align-items: center;

    .position-pair {
      display: flex;
      align-items: center;
      gap: 12px;

      .pair-icons {
        position: relative;
        width: 60px;
        height: 38px;

        .icon-primary,
        .icon-secondary {
          width: 38px;
          height: 38px;
          border-radius: 50%;
          border: 2px solid #fff;
          position: absolute;
        }

        .icon-primary {
          left: 0;
          z-index: 2;
        }

        .icon-secondary {
          right: 0;
          z-index: 1;
        }
      }

      .pair-info {
        .pair-name {
          font-size: 18px;
          font-weight: 600;
          color: #262729;
          letter-spacing: -0.01em;
        }

        .pair-label {
          font-size: 13px;
          color: #8a8b8d;
        }
      }
    }

    .health-badge {
      padding: $spacing-md $spacing-xl;
      border-radius: $radius-xl;
      text-align: center;
      min-width: 80px;

      &.health-safe {
        background: linear-gradient(135deg, rgba(8, 181, 118, 0.15) 0%, rgba(8, 181, 118, 0.08) 100%);
        border: 1px solid rgba(8, 181, 118, 0.3);
      }

      &.health-warning {
        background: linear-gradient(135deg, rgba(255, 183, 38, 0.15) 0%, rgba(255, 183, 38, 0.08) 100%);
        border: 1px solid rgba(255, 183, 38, 0.3);
      }

      &.health-danger {
        background: linear-gradient(135deg, rgba(251, 71, 71, 0.15) 0%, rgba(251, 71, 71, 0.08) 100%);
        border: 1px solid rgba(251, 71, 71, 0.3);
      }

      .health-label {
        font-size: 11px;
        font-weight: 600;
        text-transform: uppercase;
        letter-spacing: 0.5px;
        color: #4e4f51;
        margin-bottom: 2px;
      }

      .health-value {
        font-size: 16px;
        font-weight: 700;
        font-variant-numeric: tabular-nums;

        .health-safe & {
          color: $success;
        }

        .health-warning & {
          color: $warning;
        }

        .health-danger & {
          color: $danger;
        }
      }
    }
  }

  .health-bar-container {
    padding: 20px 28px 24px;
    background: rgba(249, 250, 251, 0.5);
    border-bottom: 1px solid rgba(33, 44, 66, 0.06);

    .health-bar-wrapper {
      position: relative;
    }

    .health-bar {
      width: 100%;
      height: 20px;
      background: transparent;
      border-radius: $radius-2xl;
      overflow: visible;
      position: relative;
      display: flex;
      border: 2px solid rgba(33, 44, 66, 0.1);
      box-shadow: inset 0 2px 4px rgba(0, 0, 0, 0.05);

      &:hover {
        .threshold-label {
          opacity: 1 !important;
        }
      }

      .health-zone {
        height: 100%;
        position: relative;

        &.danger-zone {
          width: 60%;
          background: linear-gradient(90deg, rgba(251, 71, 71, 0.25) 0%, rgba(251, 71, 71, 0.15) 100%);
          border-radius: $radius-2xl 0 0 $radius-2xl;
        }

        &.warning-zone {
          width: 15%;
          background: linear-gradient(90deg, rgba(255, 183, 38, 0.25) 0%, rgba(255, 183, 38, 0.15) 100%);
        }

        &.safe-zone {
          flex: 1;
          background: linear-gradient(90deg, rgba(8, 181, 118, 0.25) 0%, rgba(8, 181, 118, 0.15) 100%);
          border-radius: 0 $radius-2xl $radius-2xl 0;
        }
      }

      .health-threshold {
        position: absolute;
        top: 0;
        bottom: 0;
        transform: translateX(-50%);
        z-index: 3;

        .threshold-line {
          width: 2px;
          height: 100%;
          background: rgba(33, 44, 66, 0.153);
          position: relative;
        }

        .threshold-label {
          opacity: 0;
          position: absolute;
          top: -24px;
          left: 50%;
          transform: translateX(-50%);
          font-size: 11px;
          font-weight: 600;
          color: #4e4f51;
          white-space: nowrap;
          background: rgba(255, 255, 255, 0.95);
          padding: 2px 6px;
          border-radius: $radius-xs;
          border: 1px solid rgba(33, 44, 66, 0.1);
          transition: opacity 0.2s ease;
        }
      }

      .health-indicator {
        position: absolute;
        top: 50%;
        transform: translate(-50%, -50%);
        z-index: 5;
        transition: left 0.3s cubic-bezier(0.4, 0, 0.2, 1);

        .indicator-dot {
          width: 14px;
          height: 14px;
          border-radius: 50%;
          border: 3px solid #fff;
          box-shadow:
            0 2px 8px rgba(0, 0, 0, 0.15),
            0 0 0 2px rgba(33, 44, 66, 0.1);
          position: relative;
          transition: all 0.3s ease;
          cursor: pointer;

          &::after {
            content: '';
            position: absolute;
            inset: -4px;
            border-radius: 50%;
            animation: pulse 2s ease-in-out infinite;
          }
        }

        .indicator-value {
          position: absolute;
          top: -32px;
          left: 50%;
          transform: translateX(-50%);
          font-size: 12px;
          font-weight: 700;
          white-space: nowrap;
          background: #fff;
          padding: 4px 10px;
          border-radius: $radius-md;
          box-shadow:
            0 2px 8px rgba(0, 0, 0, 0.1),
            0 0 0 1px rgba(33, 44, 66, 0.1);
          font-variant-numeric: tabular-nums;
        }

        &.health-safe {
          .indicator-dot {
            background: linear-gradient(135deg, #08b576 0%, #0cd68a 100%);
          }

          .indicator-value {
            color: #08b576;
            border: 2px solid rgba(8, 181, 118, 0.3);
          }

          .indicator-dot::after {
            background: rgba(8, 181, 118, 0.2);
          }
        }

        &.health-warning {
          .indicator-dot {
            background: linear-gradient(135deg, #ffb726 0%, #ffc547 100%);
          }

          .indicator-value {
            color: #e49c0b;
            border: 2px solid rgba(255, 183, 38, 0.3);
          }

          .indicator-dot::after {
            background: rgba(255, 183, 38, 0.2);
          }
        }

        &.health-danger {
          .indicator-dot {
            background: linear-gradient(135deg, #fb4747 0%, #ff5e5e 100%);
          }

          .indicator-value {
            color: #fb4747;
            border: 2px solid rgba(251, 71, 71, 0.3);
          }

          .indicator-dot::after {
            background: rgba(251, 71, 71, 0.2);
          }
        }
      }
    }

    .health-labels {
      display: flex;
      justify-content: space-between;
      margin-top: 8px;
      padding: 0 4px;
      font-size: 11px;
      font-weight: 600;
      color: #8a8b8d;
    }

    @keyframes pulse {
      0%,
      100% {
        opacity: 0.6;
        transform: scale(1);
      }
      50% {
        opacity: 0;
        transform: scale(1.8);
      }
    }
  }

  .position-card-body {
    padding: 24px 28px;

    .position-metrics {
      display: flex;
      flex-direction: column;
      gap: 16px;
    }

    .metric-row {
      display: grid;
      grid-template-columns: repeat(2, 1fr);
      gap: 16px;
    }

    .metric-item {
      padding: 14px 16px;
      background: rgba(255, 255, 255, 0.6);
      border: 1px solid rgba(33, 44, 66, 0.06);
      border-radius: $radius-xl;
      transition: all 0.2s ease;

      &:hover {
        background: rgba(255, 255, 255, 0.9);
        border-color: rgba(92, 108, 255, 0.15);
      }

      &.highlight {
        background: linear-gradient(135deg, rgba(92, 108, 255, 0.06) 0%, rgba(92, 108, 255, 0.02) 100%);
        border-color: rgba(92, 108, 255, 0.2);
      }

      .metric-label {
        display: flex;
        align-items: center;
        gap: 4px;
        font-size: 12px;
        color: #8a8b8d;
        margin-bottom: 8px;
        font-weight: 500;

        .info-icon {
          width: 13px;
          height: 13px;
          opacity: 0.5;
          cursor: help;
        }
      }

      .metric-value {
        &:has(.warning-block) {
          display: flex;
          gap: 8px;
        }

        .value-primary {
          font-size: 15px;
          font-weight: 600;
          color: #262729;
          margin-bottom: 4px;
          font-variant-numeric: tabular-nums;
        }

        .value-secondary {
          font-size: 13px;
          color: #aaabad;
          font-variant-numeric: tabular-nums;
        }

        .apy-positive {
          font-size: 16px;
          font-weight: 700;
          color: $success;
        }

        .apy-negative {
          font-size: 16px;
          font-weight: 700;
          color: $danger;
        }

        .liquidation-price {
          font-size: 15px;
          font-weight: 600;
          color: $danger;
          font-variant-numeric: tabular-nums;
        }

        .buffer-value {
          font-size: 16px;
          font-weight: 600;
          font-variant-numeric: tabular-nums;

          &.buffer-safe {
            color: $success;
          }

          &.buffer-warning {
            color: $warning;
          }

          &.buffer-danger {
            color: $danger;
          }
        }
      }
    }
  }

  .position-card-footer {
    padding: 20px 28px 24px;
    border-top: 1px solid rgba(33, 44, 66, 0.06);
    background: linear-gradient(180deg, rgba(249, 250, 251, 0.3) 0%, rgba(255, 255, 255, 0.1) 100%);

    .j-btn {
      width: 100%;
      height: 48px;
      font-size: 15px;
      font-weight: 600;
    }
  }

  @media (max-width: $breakpoint-md) {
    .summary-card .summary-grid {
      grid-template-columns: repeat(2, 1fr);
    }

    .position-card-header {
      flex-direction: column;
      align-items: flex-start;
      gap: 16px;

      .health-badge {
        align-self: stretch;
      }
    }

    .metric-row {
      grid-template-columns: 1fr !important;
    }
  }
}
</style>
