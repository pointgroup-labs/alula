<script lang="ts" setup>
import type { ChartData, ChartOptions } from 'chart.js'
import type { MarketTableItem } from '~/types/table'
import { useWindowSize } from '@vueuse/core'
import { Chart } from 'chart.js'
import annotationPlugin from 'chartjs-plugin-annotation'
import { computed, inject, ref, watch } from 'vue'
import { truncatePercent } from '~/utils'

Chart.register(annotationPlugin)

const POINTS = 100
const { width } = useWindowSize()
const isMobile = computed(() => width.value <= 650)

const selectedMarketDetails = inject('selectedMarketDetails') as Ref<MarketTableItem>
const pool = computed(() => selectedMarketDetails.value?.raw)

const BPS = 10_000n

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
  if (U_bps <= kink1_ur_bps) {
    return lerpBps(U_bps, 0n, base_apr_bps, kink1_ur_bps, kink1_apr_bps)
  }
  if (U_bps <= kink2_ur_bps) {
    return lerpBps(U_bps, kink1_ur_bps, kink1_apr_bps, kink2_ur_bps, kink2_apr_bps)
  }
  return lerpBps(U_bps, kink2_ur_bps, kink2_apr_bps, BPS, max_apr_bps)
}

const bpsToPct = (x: bigint) => Number(x) / 100

const currentUtilizationPct = computed(() => {
  const b = BigInt(pool.value?.total_borrowed ?? 0)
  const a = BigInt(pool.value?.total_available ?? 0)
  return bpsToPct(utilizationBps(b, a))
})

const kinkModel = computed<KinkedModel | null>(() => {
  const v = pool.value?.interest_rate_model?.values?.[0]
  if (!v) {
    return null
  }
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
  kinkModel.value ? bpsToPct(kinkModel.value.kink2_ur_bps) : 80,
)

const curvePoints = computed(() => {
  const m = kinkModel.value
  if (!m) {
    return []
  }
  return Array.from({ length: POINTS }, (_, i) => {
    const u_pct = (i / (POINTS - 1)) * 100
    const u_bps = BigInt(Math.round(u_pct * 100))
    const apr_bps = borrowAprAtBps(m, u_bps)
    return { u_pct, apr_pct: Number(apr_bps) / 100 }
  })
})

const yMaxPct = computed(() => {
  const m = kinkModel.value
  if (!m) {
    return 100
  }
  const maxAprPct = Number(m.max_apr_bps) / 100
  return Math.min(Math.max(100, Math.ceil(maxAprPct / 50) * 50), 500)
})

const chartData = ref<ChartData<'line'>>({ labels: [], datasets: [] })

watch(curvePoints, (pts) => {
  chartData.value.labels = pts.map(p => `${p.u_pct.toFixed(2)}%`)
  chartData.value.datasets = [{
    type: 'line',
    label: 'Borrow APR',
    data: pts.map(p => p.apr_pct),
    borderColor: '#006CE4',
    pointBackgroundColor: '#006CE4',
    fill: false,
    pointRadius: 0,
  }]
}, { immediate: true })

const chartOptions = computed<ChartOptions<'line'>>(() => ({
  responsive: true,
  maintainAspectRatio: false,
  borderWidth: 2,
  pointRadius: 0,
  pointHoverRadius: 2,
  interaction: { intersect: false, mode: 'index' },
  elements: { line: { tension: 0.4, cubicInterpolationMode: 'monotone' } },
  scales: {
    x: {
      min: 0,
      max: 100,
      ticks: {
        autoSkip: false,
        callback: (value) => {
          const shown = [0, 17, 34, 51, 68, 85, 100]
          const v = Number(value)
          return shown.includes(v) ? `${v}%` : ''
        },
        font: { size: isMobile.value ? 8 : 12 },
      },
    },
    y: {
      min: 0,
      max: yMaxPct.value,
      ticks: {
        count: 3,
        callback: (val, idx) => {
          if (idx === 0) { return '0%' }
          if (idx === 1) { return '50%' }
          if (idx === 2) { return '100%' }
          return ''
        },
        font: { size: isMobile.value ? 8 : 12 },
      },
      grid: { color: '#DFE0E2', drawBorder: false },
      border: { display: false, dash: [2, 2] },
    },
  },
  plugins: {
    tooltip: {
      boxPadding: 4, usePointStyle: true, bodySpacing: 8,
      callbacks: {
        title(ctx: any) { return `Utilization: ${ctx[0].label}` },
        label(ctx: any) {
          const p = curvePoints.value[ctx.dataIndex]
          return `${ctx.dataset.label}: ${truncatePercent(p?.apr_pct ?? 0)}%`
        },
      },
    },
    legend: { display: false },
    annotation: {
      annotations: {
        current: {
          type: 'line',
          scaleID: 'x',
          value: currentUtilizationPct.value,
          borderColor: '#006CE4',
          borderDash: [2, 2],
          label: {
            display: true,
            content: `Current: ${currentUtilizationPct.value.toFixed(2)}%`,
            enabled: true,
            position: 'center',
            backgroundColor: '#006CE4',
            color: '#fff',
            borderRadius: 50,
            borderWidth: 2,
            borderColor: '#006CE4',
            font: { size: 10, weight: 'bold' },
            yAdjust: -25,
          },
        },
        optimal: {
          type: 'line',
          scaleID: 'x',
          value: optimalUtilizationPct.value,
          borderColor: '#FFD101',
          borderWidth: 2,
          borderDash: [2, 2],
          label: {
            display: true,
            content: `Optimal: ${optimalUtilizationPct.value}%`,
            enabled: true,
            position: 'center',
            backgroundColor: '#FFD101',
            color: '#111',
            borderRadius: 50,
            borderWidth: 2,
            borderColor: '#FFD101',
            font: { size: 10, weight: 'bold' },
            yAdjust: -25,
          },
        },
      },
    },
  },
}))
</script>

<template>
  <div
    :key="curvePoints.length"
    class="market-interest-chart"
  >
    <custom-mixed-chart
      :chart-data="chartData"
      :chart-options="chartOptions"
      :max-ticks-limit="6"
      chart-height="115px"
    />
  </div>
</template>

<style lang="scss">
.market-interest-chart {
  height: 115px;
  width: 424px;

  @media (max-width: $breakpoint-xs) {
    width: 100%;
  }
}
</style>
