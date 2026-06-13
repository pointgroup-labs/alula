<script lang="ts" setup>
import type { ECharts, EChartsOption } from 'echarts'
import * as echarts from 'echarts'

import { normalizeChartDate } from '~/utils/chart'

const statisticsStore = useMarketStatisticsStore()

const route = useRoute()
const router = useRouter()

const { width } = useWindowSize()
const isMobile = computed(() => width.value <= 650)

const chartFilter = useChartFilter()
const activeFilter = toRef(chartFilter, 'activeFilter')

const historyData = computed(() => {
  const length = activeFilter.value.value ?? 0
  const data = statisticsStore.historyMap.get(`${statisticsStore.marketAddress}-${statisticsStore.poolAddress}-1d`) ?? []
  return data.slice(0, Number(length))
})

const currentSupplyApy = computed(() => Number(historyData.value[0]?.supply_apy_bps ?? 0) / 100)
const currentBorrowApy = computed(() => Number(historyData.value[0]?.borrow_apy_bps ?? 0) / 100)

const maxY = ref(1)
const labels = ref<string[]>([])
const dates = ref<string[]>([])
const supplyValues = ref<number[]>([])
const borrowValues = ref<number[]>([])

watch(historyData, (data) => {
  dates.value = data.map(d => String(d.start_time))
  labels.value = data.map(d => normalizeChartDate(String(d.start_time), false))

  maxY.value = 1
  supplyValues.value = data.map((d) => {
    const v = Number(d.supply_apy_bps) / 100
    maxY.value = Math.max(maxY.value, v)
    return v
  })
  borrowValues.value = data.map((d) => {
    const v = Number(d.borrow_apy_bps) / 100
    maxY.value = Math.max(maxY.value, v)
    return v
  })
}, { immediate: true })

// ---- ECharts init ----
const el = ref<HTMLElement | null>(null)
let chart: ECharts | null = null

const option = computed<EChartsOption>(() => {
  const supply = '#22d3ee'
  const borrow = '#8a8df4'

  const gridLine = 'rgba(120, 160, 200, 0.18)'
  const axisText = 'rgba(180, 200, 220, 0.55)'

  return {
    backgroundColor: 'transparent',
    animation: true,
    animationDelay: 0,
    animationDuration: 500,
    animationDurationUpdate: 500,
    animationEasingUpdate: 'cubicOut',

    grid: { left: 0, right: 10, top: 0, bottom: 12 },

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
        const raw = dates.value[idx]
        let dateStr = arr[0]?.axisValue as string
        if (raw) {
          const d = new Date(raw)
          const y = d.getFullYear()
          const mo = String(d.getMonth() + 1).padStart(2, '0')
          const day = String(d.getDate()).padStart(2, '0')
          const h = String(d.getHours()).padStart(2, '0')
          const min = String(d.getMinutes()).padStart(2, '0')
          dateStr = `${y}/${mo}/${day} ${h}:${min}`
        }
        const lines = arr.map((p: any) => `${p.marker}${p.seriesName}: <b>${truncatePercent(p.data, 2)}%</b>`)
        return `<div style="margin-bottom:6px;color:#b4c8dc;">${dateStr}</div>${lines.join('<br/>')}`
      },
    },

    xAxis: {
      type: 'category',
      data: labels.value,
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
      min: 0,
      max: Math.ceil(maxY.value),
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
      {
        name: 'Supply APR',
        type: 'line',
        data: supplyValues.value,
        smooth: 0.45,
        showSymbol: false,
        lineStyle: { width: 2, color: supply, cap: 'round', join: 'round' },
        itemStyle: { color: supply },
        areaStyle: {
          opacity: 1,
          color: new echarts.graphic.LinearGradient(0, 0, 0, 1, [
            { offset: 0, color: 'rgba(34, 211, 238, 0.4)' },
            { offset: 1, color: 'rgba(34, 211, 238, 0)' },
          ]),
        },
      },
      {
        name: 'Borrow APR',
        type: 'line',
        data: borrowValues.value,
        smooth: 0.45,
        showSymbol: false,
        lineStyle: { width: 2, color: borrow, cap: 'round', join: 'round' },
        itemStyle: { color: borrow },
        areaStyle: {
          opacity: 1,
          color: new echarts.graphic.LinearGradient(0, 0, 0, 1, [
            { offset: 0, color: 'rgba(138, 142, 244, 0.4)' },
            { offset: 1, color: 'rgba(138, 142, 244, 0)' },
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
  chart.setOption(option.value, { notMerge: false, lazyUpdate: true })
}

function goToStatistics() {
  const marketAddress = route.params?.market as string
  const poolAddress = route.params?.pool as string
  router.push(`/statistics/${marketAddress}/${poolAddress}`)
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
</script>

<template>
  <section id="market-history-chart">
    <div class="stat-card">
      <div class="stat-card__header">
        <h3 class="title">
          Interest Rate History

          <j-tooltip>
            <i-app-export-icon @click="goToStatistics" />
            <template #content>
              Go to detail pool statistics
            </template>
          </j-tooltip>
        </h3>

        <div class="current-metrics-data">
          <metric-indicator
            color="#22d3ee"
            label="Supply"
            :value="`${formatPrice(currentSupplyApy, 1, 2)}%`"
          />

          <metric-indicator
            color="#8a8df4"
            label="Borrow"
            :value="`${formatPrice(currentBorrowApy, 1, 2)}%`"
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
section#market-history-chart {
  .title {
    display: flex;
    align-items: center;
    gap: 6px;

    [class*='tooltip'] {
      opacity: 0.7;
      cursor: pointer;

      &:hover {
        opacity: 1;
      }
    }
  }
  .current-metrics-data {
    display: flex;
    align-items: center;
    gap: 12px;
    margin-left: 20px;

    @media (max-width: $breakpoint-xs) {
      margin: 6px auto;
    }
  }

  .chart-date-filters {
    margin-left: auto;

    @media (max-width: $breakpoint-xs) {
      margin: 0 auto;
    }
  }
}
</style>
