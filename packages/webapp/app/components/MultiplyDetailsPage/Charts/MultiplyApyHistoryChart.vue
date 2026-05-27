<script lang="ts" setup>
import type { ECharts, EChartsOption } from 'echarts'
import type { ApiHistoryData } from '~/services'
import { bpsToNumber } from '@alula/client-sdk'
import * as echarts from 'echarts'
import { fetchPoolHistory } from '~/services'
import { normalizeChartDate } from '~/utils/chart'
import { calcMultiplyObligationNetApy, getApyRangeForMultiplier } from '~/utils/multiply'

const MY_APY_COLOR = '#22d3ee'
const MIN_APY_COLOR = '#8a8df4'
const MAX_APY_COLOR = '#f59e0b'
const MULTIPLIER_COLOR = '#47cd89'

const route = useRoute()

const multiplyStore = useMultiplyStore()
const selectedVault = computed(() => multiplyStore.selectedVault)
const position = computed(() => {
  if (!selectedVault.value) { return }
  return multiplyStore.positions.find(p => p.market === selectedVault.value!.market && p.pairKey === selectedVault.value!.pairKey)
})
const hasPosition = computed(() => !!position.value)

const chartFilter = useChartFilter()
const activeFilter = toRef(chartFilter, 'activeFilter')

const supplyHistory = ref<ApiHistoryData[]>([])
const borrowHistory = ref<ApiHistoryData[]>([])

const supplyData = computed(() => supplyHistory.value.slice(0, Number(activeFilter.value.value)))
const borrowData = computed(() => borrowHistory.value.slice(0, Number(activeFilter.value.value)))

// Live header metrics from on-chain vault data
const currentMyApy = computed(() => position.value?.currentApy ?? 0)
const currentMultiplier = computed(() => position.value?.currentMultiplier ?? 0)

const currentMinApy = computed(() => {
  if (!selectedVault.value) { return 0 }
  const maxMultiplier = selectedVault.value.maxMultiplier
  const supplyApy = bpsToNumber(Number(selectedVault.value.depositPoolData?.apy?.supply_bps ?? 0)) * 100
  const borrowApy = bpsToNumber(Number(selectedVault.value.borrowPoolData?.apy?.borrow_bps ?? 0)) * 100
  return getApyRangeForMultiplier({ supplyApy, borrowApy, maxMultiplier }).minApy
})
const currentMaxApy = computed(() => {
  if (!selectedVault.value) { return 0 }
  const maxMultiplier = selectedVault.value.maxMultiplier
  const supplyApy = bpsToNumber(Number(selectedVault.value.depositPoolData?.apy?.supply_bps ?? 0)) * 100
  const borrowApy = bpsToNumber(Number(selectedVault.value.borrowPoolData?.apy?.borrow_bps ?? 0)) * 100
  return getApyRangeForMultiplier({ supplyApy, borrowApy, maxMultiplier }).maxApy
})

const { width } = useWindowSize()
const isMobile = computed(() => width.value <= 650)

const chartData = computed(() => {
  const supply = supplyData.value
  const borrow = borrowData.value

  if (supply.length === 0 || borrow.length === 0) {
    return {
      minY: 0,
      maxY: 0,
      labels: [] as string[],
      rawDates: [] as string[],
      minApyValues: [] as number[],
      myApyValues: [] as number[],
      multiplierValues: [] as number[],
      maxApyValues: [] as number[],
    }
  }

  const len = Math.min(supply.length, borrow.length)
  const maxMult = selectedVault.value?.maxMultiplier ?? 1
  const pos = position.value
  const userMult = pos?.currentMultiplier ?? 1

  const labels: string[] = []
  const rawDates: string[] = []
  const minApyValues: number[] = []
  const maxApyValues: number[] = []
  const myApyValues: number[] = []
  const multiplierValues: number[] = []
  let minY = 0
  let maxY = 0

  for (let i = 0; i < len; i++) {
    const s = supply[i]!
    const b = borrow[i]!
    const supplyApy = s.supply_apy_bps / 100
    const borrowApy = b.borrow_apy_bps / 100

    const raw = String(s.start_time)
    rawDates.push(raw)
    labels.push(normalizeChartDate(raw, false))

    const { minApy: minApyVal, maxApy: maxApyVal } = getApyRangeForMultiplier({
      supplyApy,
      borrowApy,
      maxMultiplier: maxMult,
    })
    minApyValues.push(minApyVal)
    minY = Math.min(minY, minApyVal)
    maxY = Math.max(maxY, minApyVal)

    maxApyValues.push(maxApyVal)
    minY = Math.min(minY, maxApyVal)
    maxY = Math.max(maxY, maxApyVal)

    if (pos) {
      const myApyVal = calcMultiplyObligationNetApy({
        suppliedUsd: userMult,
        borrowedUsd: userMult - 1,
        supplyApy,
        borrowApy,
      })
      myApyValues.push(myApyVal)
      multiplierValues.push(userMult)
      minY = Math.min(minY, myApyVal, userMult)
      maxY = Math.max(maxY, myApyVal, userMult)
    }
  }

  return {
    minY,
    maxY,
    labels,
    rawDates,
    minApyValues,
    myApyValues,
    multiplierValues,
    maxApyValues,
  }
})

function formatTooltipValue(value: number, suffix = '%') {
  return `${Number.isFinite(value) ? value.toFixed(2) : '0.00'}${suffix}`
}

function tooltipMarker(color: string) {
  return `<span style="display:inline-block;margin-right:8px;border-radius:50%;width:10px;height:10px;background:${color};"></span>`
}

// ---- ECharts init ----
const el = ref<HTMLElement | null>(null)
let chart: ECharts | null = null

const option = computed<EChartsOption>(() => {
  const gridLine = 'rgba(120, 160, 200, 0.18)'
  const axisText = 'rgba(180, 200, 220, 0.55)'
  const { minY, maxY, labels, rawDates, minApyValues, myApyValues, multiplierValues, maxApyValues } = chartData.value

  const range = maxY - minY
  const padding = range === 0 ? 0.5 : Math.max(range * 0.12, 0.2)
  const yMin = Math.floor(minY - padding)
  const yMax = Math.ceil(maxY + padding)

  return {
    backgroundColor: 'transparent',
    animation: true,
    animationDuration: 500,
    animationDurationUpdate: 400,
    animationEasingUpdate: 'cubicOut',

    grid: {
      left: isMobile.value ? 42 : 56,
      right: isMobile.value ? 18 : 28,
      top: 14,
      bottom: isMobile.value ? 20 : 24,
      containLabel: false,
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
        const arr = Array.isArray(params) ? params : [params]
        const idx: number = arr[0]?.dataIndex ?? 0
        const raw = rawDates[idx]
        const title = raw ? normalizeChartDate(raw, true) : (arr[0]?.axisValue as string)
        const lines = hasPosition.value
          ? [
              `${tooltipMarker(MY_APY_COLOR)}My APY: <b>${formatTooltipValue(myApyValues[idx] ?? 0)}</b>`,
              `${tooltipMarker(MAX_APY_COLOR)}Max APY: <b>${formatTooltipValue(maxApyValues[idx] ?? 0)}</b>`,
              `${tooltipMarker(MULTIPLIER_COLOR)}Multiplier: <b>${formatTooltipValue(currentMultiplier.value, 'x')}</b>`,
            ]
          : [
              `${tooltipMarker(MIN_APY_COLOR)}Min APY: <b>${formatTooltipValue(minApyValues[idx] ?? 0)}</b>`,
              `${tooltipMarker(MAX_APY_COLOR)}Max APY: <b>${formatTooltipValue(maxApyValues[idx] ?? 0)}</b>`,
            ]
        return `<div style="margin-bottom:6px;">${title}</div>${lines.join('<br/>')}`
      },
    },

    xAxis: {
      type: 'category',
      data: labels,
      boundaryGap: false,
      axisLine: { show: false },
      axisTick: { show: false },
      splitLine: { show: false },
      axisLabel: {
        hideOverlap: true,
        color: axisText,
        fontSize: isMobile.value ? 10 : 12,
        margin: 14,
        formatter: (v: string) => v,
      },
    },

    yAxis: {
      type: 'value',
      min: yMin,
      max: yMin === yMax ? yMax + 1 : yMax,
      axisLine: { show: false },
      axisTick: { show: false },
      axisLabel: {
        color: axisText,
        fontSize: isMobile.value ? 10 : 12,
        margin: 18,
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
      ...(hasPosition.value
        ? [{
            name: 'My APY',
            type: 'line' as const,
            data: myApyValues,
            smooth: 0.45,
            showSymbol: false,
            lineStyle: { width: 2, color: MY_APY_COLOR, cap: 'round' as const, join: 'round' as const },
            itemStyle: { color: MY_APY_COLOR },
            areaStyle: {
              origin: 'start' as const,
              opacity: 1,
              color: new echarts.graphic.LinearGradient(0, 0, 0, 1, [
                { offset: 0, color: 'rgba(34, 211, 238, 0.4)' },
                { offset: 1, color: 'rgba(34, 211, 238, 0)' },
              ]),
            },
          }, {
            name: 'Multiplier',
            type: 'line' as const,
            data: multiplierValues,
            smooth: 0.45,
            showSymbol: false,
            lineStyle: { width: 2, color: MULTIPLIER_COLOR, cap: 'round' as const, join: 'round' as const },
            itemStyle: { color: MULTIPLIER_COLOR },
            areaStyle: {
              origin: 'start' as const,
              opacity: 1,
              color: new echarts.graphic.LinearGradient(0, 0, 0, 1, [
                { offset: 0, color: 'rgba(71, 205, 137, 0.4)' },
                { offset: 1, color: 'rgba(71, 205, 137, 0)' },
              ]),
            },
          }]
        : [{
            name: 'Min APY',
            type: 'line' as const,
            data: minApyValues,
            smooth: 0.45,
            showSymbol: false,
            lineStyle: { width: 2, color: MIN_APY_COLOR, cap: 'round' as const, join: 'round' as const },
            itemStyle: { color: MIN_APY_COLOR },
            areaStyle: {
              origin: 'start' as const,
              opacity: 1,
              color: new echarts.graphic.LinearGradient(0, 0, 0, 1, [
                { offset: 0, color: 'rgba(138, 141, 244, 0.4)' },
                { offset: 1, color: 'rgba(138, 141, 244, 0)' },
              ]),
            },
          }]),
      {
        name: 'Max APY',
        type: 'line',
        data: maxApyValues,
        smooth: 0.45,
        showSymbol: false,
        lineStyle: { width: 2, color: MAX_APY_COLOR, cap: 'round', join: 'round' },
        itemStyle: { color: MAX_APY_COLOR },
        areaStyle: {
          origin: 'start' as const,
          opacity: 1,
          color: new echarts.graphic.LinearGradient(0, 0, 0, 1, [
            { offset: 0, color: 'rgba(245, 158, 11, 0.4)' },
            { offset: 1, color: 'rgba(245, 158, 11, 0)' },
          ]),
        },
      },
    ],
  }
})

function render() {
  if (!chart) {
    return
  }
  chart.setOption(option.value, { notMerge: false, replaceMerge: ['series'], lazyUpdate: true })
}

watch(option, () => {
  render()
})

onBeforeUnmount(() => {
  if (import.meta.env.SSR) {
    return
  }
  chart?.dispose()
  chart = null
})

onMounted(async () => {
  if (import.meta.env.SSR) {
    return
  }
  await nextTick()
  if (!el.value) {
    return
  }

  chart = echarts.init(el.value)
  render()

  window.addEventListener('resize', () => chart?.resize())
})

watch(route, async (r) => {
  const market = r.params.market as string
  const pair = r.params.pair as string
  if (!market || !pair) {
    return
  }
  const [supplyPoolAdderss, borrowPoolAddress] = pair.split(':')
  if (!supplyPoolAdderss || !borrowPoolAddress) {
    return
  }
  const promises = [
    () => fetchPoolHistory(market, supplyPoolAdderss, '1d'),
    () => fetchPoolHistory(market, borrowPoolAddress, '1d'),
  ]
  const [supplyData, borrowData] = await Promise.all(promises.map(p => p()))
  console.group('%c[Leverage Pool History]', 'color: #1dc978; font-weight: bold;')

  console.log('%cSupply APY:', 'color: #4fc3f7', supplyData)
  console.log('%cBorrow APY:', 'color: #ffb74d', borrowData)

  console.groupEnd()
  if (!supplyData || !borrowData) {
    return
  }
  supplyHistory.value = supplyData
  borrowHistory.value = borrowData
}, { immediate: true })
</script>

<template>
  <section id="multiply-apy-history-chart">
    <div class="stat-card">
      <div class="stat-card__header">
        <h3 class="title">
          Net APY History
        </h3>

        <div class="current-metrics-data">
          <metric-indicator
            v-if="hasPosition"
            :color="MY_APY_COLOR"
            label="My APY"
            :value="`${formatPrice(currentMyApy, 1, 2)}%`"
          />

          <metric-indicator
            v-if="!hasPosition"
            :color="MIN_APY_COLOR"
            label="Min APY"
            :value="`${formatPrice(currentMinApy, 1, 2)}%`"
          />

          <metric-indicator
            v-if="hasPosition"
            :color="MULTIPLIER_COLOR"
            label="Multiplier"
            :value="`${formatPrice(currentMultiplier, 2, 2)}x`"
          />

          <metric-indicator
            :color="MAX_APY_COLOR"
            label="Max APY"
            :value="`${formatPrice(currentMaxApy, 1, 2)}%`"
          />
        </div>

        <chart-date-filter
          v-model="activeFilter"
          :filters="chartFilter.filters"
        />
      </div>

      <div class="stat-card__body">
        <client-only>
          <div class="history-chart">
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
section#multiply-apy-history-chart {
  h3 {
    margin: 0;
  }

  .current-metrics-data {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 12px;
    margin-left: 20px;

    @media (max-width: $breakpoint-xs) {
      margin: 6px auto;
      justify-content: center;
      row-gap: 4px;
    }
  }

  .chart-date-filters {
    margin-left: auto;

    @media (max-width: $breakpoint-xs) {
      margin: 0 auto;
    }
  }

  .stat-card__header {
    white-space: nowrap;
    flex-wrap: wrap;
    gap: 6px;
  }
}
</style>
