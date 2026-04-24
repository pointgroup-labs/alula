<script lang="ts" setup>
const {
  position,
  selectedVault,
} = useLeveragePosition()

// Hard cap enforced on chain by the deposit pool's open_ltv (1 / (1 - open_ltv)).
// This is different from `selectedVault.maxMultiplier`, which applies a 0.8 SAFETY_MULTIPLIER
// discount used as the OPENING soft cap (so users leave headroom for slippage/fees).
// The live position can legitimately drift between the soft cap and the hard cap because
// of price movement after open, so we compare against the hard cap here to avoid showing
// "2.66x of 2.28x" when the position is actually still inside the contract limit.
const hardMaxMultiplier = computed(() => {
  if (!position.value) {
    return undefined
  }
  const openLtvRate = position.value.openLtv / 100
  if (!Number.isFinite(openLtvRate) || openLtvRate <= 0 || openLtvRate >= 1) {
    return undefined
  }
  return 1 / (1 - openLtvRate)
})
</script>

<template>
  <template v-if="position">
    <div class="pool-card position-panel">
      <div class="position-panel__eyebrow">
        Risk Metrics
      </div>

      <div class="metrics-list">
        <!-- multiplier -->
        <div class="metrics-list__row">
          <div class="metrics-list__row__label">
            Effective leverage
            <info-tooltip>
              Your live exposure ÷ equity right now — not the multiplier you picked at open.
              It drifts as the price of your collateral moves: collateral falling pushes it up,
              collateral rising pulls it down.
              <br>
              Hard ceiling enforced on chain: {{ hardMaxMultiplier !== undefined ? `${truncatePercent(hardMaxMultiplier, 2)}x` : '—' }}
              (from open LTV {{ truncatePercent(position.openLtv, 2) }}%).
              <br>
              Suggested maximum at open (with safety headroom for slippage and fees): {{ truncatePercent(selectedVault.maxMultiplier, 2) }}x.
            </info-tooltip>
          </div>
          <div class="metrics-list__row__value">
            <template v-if="Number.isFinite(position.currentMultiplier)">
              <strong>{{ truncatePercent(position.currentMultiplier, 2) }}x</strong>
            </template>
            <strong v-else>{{ '<0' }}</strong>
            of
            <strong>
              {{ hardMaxMultiplier !== undefined ? `${truncatePercent(hardMaxMultiplier, 2)}x` : `${selectedVault.maxMultiplier}x` }}
            </strong>
          </div>
        </div>

        <!-- Current LTV -->
        <div class="metrics-list__row">
          <div class="metrics-list__row__label">
            Current LTV
          </div>
          <div class="metrics-list__row__value">
            {{ truncatePercent(position.currentLtv, 2) }}%
          </div>
        </div>

        <!-- Liquidation LTV -->
        <div class="metrics-list__row">
          <div class="metrics-list__row__label">
            Liquidation LTV
          </div>
          <div class="metrics-list__row__value">
            {{ truncatePercent(position.closeLtv, 2) }}%
          </div>
        </div>

        <!-- liquidation price -->
        <div class="metrics-list__row">
          <div class="metrics-list__row__label">
            Liquidation Price
            <info-tooltip>
              Estimated {{ selectedVault?.asset.symbol }} price where this position reaches liquidation.
              <br>
              Based on Close LTV {{ truncatePercent(position.closeLtv, 2) }}% and liability factor {{ truncatePercent(position.liabilityFactor, 2) }}%.
            </info-tooltip>
          </div>
          <div class="metrics-list__row__value metrics-list__row__value--stacked">
            <strong>{{ position.liquidationPrice !== null ? `$${formatPrice(position.liquidationPrice, 2, position.liquidationPrice < 1 ? 6 : 4)}` : '—' }}</strong>
            <small v-if="position.distanceToLiquidationPercent !== null">
              {{ truncatePercent(position.distanceToLiquidationPercent, 2) }}% above liquidation
            </small>
          </div>
        </div>

        <!-- liquidation buffer -->
        <div class="metrics-list__row">
          <div class="metrics-list__row__label">
            Liquidation Buffer
            <info-tooltip>
              Estimated downside room before liquidation at current debt and collateral settings.
              <br>
              Current LTV: {{ truncatePercent(position.currentLtv, 2) }}%. Close LTV limit: {{ truncatePercent(position.closeLtv, 2) }}%.
            </info-tooltip>
          </div>
          <div class="metrics-list__row__value metrics-list__row__value--stacked">
            <strong>{{ formatCompactUSD(position.liquidationBufferUsd, 2, 2) }}</strong>
          </div>
        </div>
      </div>
    </div>

    <div class="pool-card position-panel">
      <div class="position-panel__eyebrow">
        Yield Breakdown
      </div>

      <div class="metrics-list">
        <div class="metrics-list__row">
          <div class="metrics-list__row__label">
            Supply APY
          </div>
          <div class="metrics-list__row__value text-cyan ">
            <strong>{{ truncatePercent(position.supplyApy, 2) }}%</strong>
          </div>
        </div>
        <div class="metrics-list__row">
          <div class="metrics-list__row__label">
            Borrow rate
          </div>
          <div class="metrics-list__row__value text-indigo">
              <strong>{{ truncatePercent(position.borrowApy, 2) }}%</strong>
          </div>
        </div>
        <div class="metrics-list__row">
          <div class="metrics-list__row__label">
            Estimated yearly result
          </div>
          <div class="metrics-list__row__value">
            <span :class="position.yearlyResultUsd >= 0 ? 'text-positive' : 'text-negative'">
              <strong> ≈ {{ position.yearlyResultUsd >= 0 ? '+' : '' }}{{ formatCompactUSD(position.yearlyResultUsd || 0, 2, 2) }}</strong>
            </span>
          </div>
        </div>
      </div>
    </div>
  </template>
</template>
