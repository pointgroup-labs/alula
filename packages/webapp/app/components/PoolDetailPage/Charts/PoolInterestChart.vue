<script lang="ts" setup>
import type { ECharts, EChartsOption } from 'echarts'
import type { MarketTableItem } from '~/types/table'
import { useWindowSize } from '@vueuse/core'
import * as echarts from 'echarts'
import { computed, inject, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { truncatePercent } from '~/utils'

/* ------------------------------------------------ */
/* STATE */
/* ------------------------------------------------ */

const POINTS = 100
const { width } = useWindowSize()
const isMobile = computed(() => width.value <= 650)

const selectedPool = inject('selectedPool') as Ref<MarketTableItem>
const pool = computed(() => selectedPool.value?.raw.pool)

const BPS = 10_000n

/* ------------------------------------------------ */
/* INTEREST MODEL
/* ------------------------------------------------ */

type KinkedModel = {
  base_apr_bps: bigint
  kink1_apr_bps: bigint
  kink1_ur_bps: bigint
  kink2_apr_bps: bigint
  kink2_ur_bps: bigint
  max_apr_bps: bigint
}

function utilizationBps(totalBorrowed: bigint, totalAvailable: bigint): bigint {
  const denom = totalBorrowed + totalAvailable
  if (denom === 0n) { return 0n }
  return (totalBorrowed * BPS) / denom
}

function lerpBps(x: bigint, x0: bigint, y0: bigint, x1: bigint, y1: bigint): bigint {
  if (x1 === x0) { return y1 }
  return y0 + ((x - x0) * (y1 - y0)) / (x1 - x0)
}

function borrowAprAtBps(model: KinkedModel, U_bps: bigint): bigint {
  const { base_apr_bps, kink1_apr_bps, kink1_ur_bps, kink2_apr_bps, kink2_ur_bps, max_apr_bps } = model

  if (U_bps <= kink1_ur_bps) { return lerpBps(U_bps, 0n, base_apr_bps, kink1_ur_bps, kink1_apr_bps) }

  if (U_bps <= kink2_ur_bps) { return lerpBps(U_bps, kink1_ur_bps, kink1_apr_bps, kink2_ur_bps, kink2_apr_bps) }

  return lerpBps(U_bps, kink2_ur_bps, kink2_apr_bps, BPS, max_apr_bps)
}

const bpsToPct = (x: bigint) => Number(x) / 100

/* ------------------------------------------------ */

const currentUtilizationPct = computed(() => {
  const b = BigInt(pool.value?.total_borrowed ?? 0)
  const a = BigInt(pool.value?.total_available ?? 0)
  return bpsToPct(utilizationBps(b, a))
})

const kinkModel = computed<KinkedModel | null>(() => {
  const v = pool.value?.config.interest_rate_model?.values?.[0]
  if (!v) { return null }

  return {
    base_apr_bps: BigInt(v.base_apr_bps ?? 1n),
    kink1_apr_bps: BigInt(v.kink1_apr_bps ?? 3000n),
    kink1_ur_bps: BigInt(v.kink1_ur_bps ?? 7000n),
    kink2_apr_bps: BigInt(v.kink2_apr_bps ?? 6000n),
    kink2_ur_bps: BigInt(v.kink2_ur_bps ?? 8000n),
    max_apr_bps: BigInt(v.max_apr_bps ?? 40_000n),
  }
})

const optimalUtilizationPct = computed(() =>
  kinkModel.value ? bpsToPct(kinkModel.value.kink1_ur_bps) : 80,
)

const chartMaxUtilizationPct = computed(() =>
  kinkModel.value ? bpsToPct(kinkModel.value.kink2_ur_bps) : 80,
)

const maxUtilizationPct = computed(() => {
  const relevantMaxPct = Math.max(
    chartMaxUtilizationPct.value,
    currentUtilizationPct.value,
  )

  if (!Number.isFinite(relevantMaxPct) || relevantMaxPct <= 0) {
    return chartMaxUtilizationPct.value || 100
  }

  return Math.min(relevantMaxPct, 100)
})

const gradientStops = computed(() => {
  const maxPct = maxUtilizationPct.value

  if (!kinkModel.value || maxPct <= 0) {
    return { kink1: 0.7, kink2: 0.85 }
  }

  const toOffset = (pct: number) => Math.min(Math.max(pct / maxPct, 0), 1)

  return {
    kink1: toOffset(bpsToPct(kinkModel.value.kink1_ur_bps)),
    kink2: toOffset(bpsToPct(kinkModel.value.kink2_ur_bps)),
  }
})

const chartCurrentUtilizationPct = computed(() => currentUtilizationPct.value)

const chartOptimalUtilizationPct = computed(() => optimalUtilizationPct.value)

const chartCurrentUtilizationPctColor = computed(() => {
  if (!currentUtilizationPct.value || !chartOptimalUtilizationPct.value) {
    return '#c6ccd9'
  }
  const percent = currentUtilizationPct.value / chartOptimalUtilizationPct.value
  return percent < 0.8 ? '#17B26A' : (percent < 1 ? '#f0b100' : '#f04438')
})

/* ------------------------------------------------ */
/* CURVE */
/* ------------------------------------------------ */

const curvePoints = computed(() => {
  const m = kinkModel.value
  if (!m) { return [] }

  const maxPct = maxUtilizationPct.value

  return Array.from({ length: POINTS }, (_, i) => {
    const u_pct = (i / (POINTS - 1)) * maxPct
    const u_bps = BigInt(Math.round(u_pct * 100))
    const apr_bps = borrowAprAtBps(m, u_bps)

    return [u_pct, Number(apr_bps) / 100]
  })
})

const yMaxPct = computed(() => {
  if (!kinkModel.value) { return 100 }

  const visibleMaxAprPct = aprAtPct(maxUtilizationPct.value)
  const roundedMaxAprPct = Math.ceil(visibleMaxAprPct / 10) * 10
  const yAxisMaxPct = roundedMaxAprPct <= visibleMaxAprPct
    ? roundedMaxAprPct + 10
    : roundedMaxAprPct

  return Math.min(Math.max(10, yAxisMaxPct), 500)
})

function aprAtPct(pct: number) {
  if (!kinkModel.value) {
    return 0
  }

  const bps = BigInt(Math.round(pct * 100))
  return Number(borrowAprAtBps(kinkModel.value, bps)) / 100
}

/* ------------------------------------------------ */
/* ECHARTS */
/* ------------------------------------------------ */

const el = ref<HTMLElement | null>(null)
let chart: ECharts | null = null

const option = computed<EChartsOption>(() => {
  const gridLine = 'rgba(120, 160, 200, 0.18)'
  const axisText = 'rgba(180, 200, 220, 0.55)'

  return {
    animation: true,

    grid: {
      left: 10,
      right: 10,
      top: 10,
      bottom: 34,
    },

    tooltip: {
      trigger: 'axis',
      axisPointer: {
        type: 'line',
        lineStyle: { color: gridLine, type: 'dashed' },
      },
      padding: 10,
      backgroundColor: 'rgba(10,14,23,0.92)',
      borderColor: 'rgba(255,255,255,0.06)',
      borderWidth: 1,
      textStyle: { color: '#E8EDF5', fontSize: 12 },
      formatter: (params: any) => {
        const p = params[0]
        return `
        Utilization: ${p.axisValue.toFixed(2)}%<br/>
        Borrow APR: ${truncatePercent(p.data[1])}%
      `
      },
    },

    xAxis: {
      type: 'value',
      min: 0,
      max: maxUtilizationPct.value,
      splitNumber: isMobile.value ? 4 : 6,
      // name: 'Utilization Rate',
      nameLocation: 'middle',
      nameGap: isMobile.value ? 28 : 30,
      nameTextStyle: {
        color: axisText,
        fontSize: isMobile.value ? 10 : 12,
      },

      axisLabel: {
        color: axisText,
        fontSize: isMobile.value ? 8 : 12,
        formatter: (v: number) => `${truncatePercent(v, 0)}%`,
      },

      splitLine: { show: false },
    },

    yAxis: {
      type: 'value',
      min: 0,
      max: yMaxPct.value,

      axisLabel: {
        color: axisText,
        formatter: (v: number) => `${v}%`,
      },

      splitLine: {
        show: true,
        lineStyle: {
          color: gridLine,
          type: 'dashed',
          width: 1,
        },
      },
    },

    series: [
      {
        id: 'curve',
        type: 'line',
        smooth: 0.4,
        showSymbol: false,
        data: curvePoints.value,

        areaStyle: {
          color: new echarts.graphic.LinearGradient(0, 0, 1, 0, [
            { offset: 0, color: 'rgba(34,211,238,0.18)' },
            { offset: gradientStops.value.kink1, color: 'rgba(34,211,238,0.18)' },
            { offset: gradientStops.value.kink2, color: 'rgba(245,158,11,0.18)' },

            { offset: 1, color: 'rgba(255,77,109,0.18)' },
          ]),
        },

        lineStyle: {
          width: 3,
          color: new echarts.graphic.LinearGradient(0, 0, 1, 0, [
            { offset: 0, color: '#22d3ee' },
            { offset: gradientStops.value.kink1, color: '#22d3ee' },
            { offset: gradientStops.value.kink2, color: '#f59e0b' },
            { offset: 1, color: '#ef4444' },
          ]),
        },

        markLine: {
          symbol: 'none',
          label: { show: false },
          lineStyle: { type: 'dashed', width: 1, opacity: 0.4 },

          data: [
            {
              xAxis: chartCurrentUtilizationPct.value,
              // label: {
              //   show: true,
              //   formatter: 'Current',
              //   position: 'insideEndTop',
              //   color: '#f04438',
              //   fontSize: 12,
              // },
              lineStyle: { color: chartCurrentUtilizationPctColor.value ?? '#f04438' },
            },
            {
              xAxis: chartOptimalUtilizationPct.value,
              // label: {
              //   show: true,
              //   formatter: 'Optimal',
              //   position: 'insideEndTop',
              //   color: '#22d3ee',
              //   fontSize: 12,
              // },
              lineStyle: { color: '#22d3ee' },
            },
            {
              xAxis: chartMaxUtilizationPct.value,
              // label: {
              //   show: true,
              //   formatter: 'Utilization',
              //   position: 'insideEndTop',
              //   color: '#22d3ee',
              //   fontSize: 12,
              // },
              lineStyle: { color: '#f04438' },
            },
          ],
        },
      },

      {
        type: 'scatter',
        symbolSize: 10,
        data: [[
          chartCurrentUtilizationPct.value,
          aprAtPct(chartCurrentUtilizationPct.value),
        ]],
        itemStyle: { color: chartCurrentUtilizationPctColor.value ?? '#f04438' },
        z: 5,
      },

      {
        type: 'scatter',
        symbolSize: 10,
        data: [[
          chartOptimalUtilizationPct.value,
          aprAtPct(chartOptimalUtilizationPct.value),
        ]],
        itemStyle: { color: '#22d3ee' },
        z: 5,
      },

      {
        type: 'scatter',
        symbolSize: 10,
        data: [[
          chartMaxUtilizationPct.value,
          aprAtPct(chartMaxUtilizationPct.value),
        ]],
        itemStyle: { color: '#f04438' },
        z: 5,
      },
    ],
  }
})

/* ------------------------------------------------ */

const resize = () => chart?.resize()

onMounted(async () => {
  if (import.meta.env.SSR) {
    return
  }
  await nextTick()

  chart = echarts.init(el.value!)
  chart.setOption(option.value)

  window.addEventListener('resize', resize)
})

watch(option, () => {
  chart?.setOption(option.value)
})

onBeforeUnmount(() => {
  window.removeEventListener('resize', resize)
  chart?.dispose()
})
</script>

<template>
  <section id="market-interest-chart">
    <div class="stat-card">
      <div class="stat-card__header">
        <h3 class="title">
          Utilization & interest rate model
        </h3>
      </div>

      <div class="stat-card__body">
        <div class="interest-stats">
          <div
            class="stat-card"
            :style="{ '--color': '#22d3ee' }"
          >
            <div class="stat-card__label">
              Optimal <br> utilization
            </div>
            <div class="stat-card__value">
              {{ truncatePercent(optimalUtilizationPct, 2) }}%
            </div>
          </div>

          <div
            class="stat-card"
            :style="{ '--color': chartCurrentUtilizationPctColor }"
          >
            <div class="stat-card__label">
              Current <br> utilization
            </div>
            <div class="stat-card__value">
              {{ truncatePercent(currentUtilizationPct, 2) }}%
            </div>
          </div>

          <div
            class="stat-card"
            :style="{ '--color': '#f04438' }"
          >
            <div class="stat-card__label">
              Max <br> utilization
            </div>
            <div class="stat-card__value">
              {{ truncatePercent(maxUtilizationPct, 2) }}%
            </div>
          </div>
        </div>
        <client-only>
          <div class="interest-chart">
            <div
              ref="el"
              style="height: 200px; width: 100%;"
            />
          </div>
        </client-only>
      </div>
    </div>

  </section>
</template>

<style lang="scss">
section#market-interest-chart {
  .stat-card__body {
    display: flex;
    gap: 24px;

    @media (max-width: $breakpoint-xs) {
      flex-direction: column;
    }
  }
  .interest-chart {
    flex: 1;
  }
  .interest-stats {
    display: flex;
    flex-direction: column;
    gap: 6px;

    .stat-card {
      padding: 10px;
      display: flex;
      justify-content: space-between;
      align-items: center;
      gap: 24px;
      border-radius: $radius-md;
      background: $bg-tertiary;

      &__label {
        width: fit-content;
        color: $text-tertiary;
        font-size: $text-xs;
        font-style: normal;
        font-weight: 400;
        line-height: 18px;
        display: flex;
        align-items: center;
        gap: 6px;

        &:before {
          content: '';
          display: block;
          min-width: 6px;
          width: 6px;
          height: 6px;
          border-radius: 50%;
          background-color: var(--color, $text-primary);
        }
      }

      &__value {
        color: var(--color, $text-primary);
        font-family: $font-JetBrainsMono;
        font-size: $text-sm;
        font-style: normal;
        font-weight: 700;
        line-height: normal;
      }
    }
  }
}
</style>
