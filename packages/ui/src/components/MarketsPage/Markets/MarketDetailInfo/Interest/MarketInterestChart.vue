<script lang="ts" setup>
import type { ChartData, ChartOptions } from 'chart.js'
import { Chart } from 'chart.js'
import annotationPlugin from 'chartjs-plugin-annotation'
import { truncatePercent } from '~/utils'

Chart.register(annotationPlugin)

const POINTS = 201

const { width } = useWindowSize()

const isMobile = computed(() => width.value <= 650)

const marketStore = useMarketsStore()
const pool = computed(() => marketStore.selectedMarketInfo)
const poolConfig = computed(() => pool.value?.raw.config)

// APR for chart
const modelParams = {
  // Minimum borrow interest rate when utilization is 0%
  baseRate: 0.0002,
  // Utilization rate threshold below which the interest rate increases slowly (linear slope)
  optimalUtil: 0.8,
  // Utilization rate at which the "jump" in interest rate slope begins
  jumpUtil: 0.91,
  // Maximum possible borrow interest rate (when utilization reaches 100%)
  maxRate: 0.99,
  // Portion of the borrow interest kept by the protocol (not given to suppliers)
  reserveFactor: 0.2,
}

function getBorrowRateForChart(util: number): number {
  const { optimalUtil, jumpUtil, maxRate } = modelParams

  if (util === 0) {
    return 0
  }

  if (util <= optimalUtil) {
    // without base rate like in Camino
    return (util / optimalUtil) * 0.0413
  }

  if (util <= jumpUtil) {
    const slope = (0.0993 - 0.0413) / (jumpUtil - optimalUtil)
    return 0.0413 + (util - optimalUtil) * slope
  }

  const finalSlope = (maxRate - 0.0993) / (1 - jumpUtil)
  return 0.0993 + (util - jumpUtil) * finalSlope
}

const dataPointsChart = computed(() => {
  return Array.from({ length: POINTS }, (_, i) => {
    const util = i * 0.005
    return {
      utilization: util * 100,
      borrowAPY: getBorrowRateForChart(util) * 100,
    }
  })
})

// APR data
const currentUtilization = computed(() => {
  if (!pool.value) {
    return 0
  }
  const borrowed = Number(pool.value.total_borrowed)
  const available = Number(pool.value.available)
  const sup = borrowed + available
  return sup === 0 ? 0 : borrowed / sup
})

const params = computed(() => {
  const c = poolConfig.value
  if (!c) {
    return null
  }
  return {
    baseRate: Number(c.base_rate_per_second),
    slope1: Number(c.slope1),
    slope2: Number(c.slope2),
    optimalUtil: Number(c.optimal_utilization_ratio_bps) / 10_000,
  }
})

const SECONDS_PER_YEAR = 31_536_000 // 365 * 24 * 60 * 60

function getBorrowAPRPercent(utilRate: number): number {
  const p = params.value
  if (!p) {
    return 0
  }
  const { baseRate, slope1, slope2, optimalUtil } = p

  const irPerSec = utilRate < optimalUtil
    ? baseRate + slope1 * utilRate
    : baseRate
      + slope1 * optimalUtil
      + (utilRate - optimalUtil) * slope2

  const aprDecimal = (irPerSec / 1_000_000_000_000) * SECONDS_PER_YEAR
  const apr = aprDecimal / (31 / 100) * 1000
  return apr
}

const dataPoints = computed(() => {
  if (!params.value) {
    return []
  }
  return Array.from({ length: POINTS }, (_, i) => {
    const u = i / (POINTS - 1)
    return {
      utilization: +(u * 100),
      borrowAPY: +getBorrowAPRPercent(u),
    }
  })
})

const chartData = ref<ChartData<'line'>>({
  labels: [],
  datasets: [],
})

watch(dataPoints, (d) => {
  if (!d) {
    return
  }

  chartData.value.labels = dataPoints.value.map(d => `${d.utilization.toFixed(2)}%`)
  chartData.value.datasets = [
    {
      type: 'line',
      borderColor: '#006CE4',
      pointBackgroundColor: '#006CE4',
      fill: false,
      label: 'Borrow APR',
      data: dataPointsChart.value.map(d => d.borrowAPY),
      pointRadius: 0,
    },
  ]
}, { immediate: true })

const chartOptions = computed<ChartOptions<'line'>>(() => {
  return {
    responsive: true,
    maintainAspectRatio: false,
    borderWidth: 2,
    pointRadius: 0,
    pointHoverRadius: 2,
    interaction: {
      intersect: false,
      mode: 'index',
    },
    elements: {
      line: {
        tension: 0.4,
        cubicInterpolationMode: 'monotone',
      },
    },
    scales: {
      x: {
        min: 0,
        max: 200,
        ticks: {
          autoSkip: false,
          maxTicksLimit: 10,
          callback: (value, _index, _ticks) => {
            const shownLabels = [0, 17, 34, 51, 68, 85, 100]
            const val = Number(value) / 2
            return shownLabels.includes(val) ? `${val}%` : ''
          },
          font: {
            size: isMobile.value ? 8 : 12,
          },
        },
      },
      y: {
        min: 0,
        max: 100,
        ticks: {
          autoSkip: false,
          stepSize: 50,
          color: isDark.value ? '#5B5B5B' : '#4E4E4E',
          callback: (value, _index, _ticks) => `${value}%`,
          font: {
            size: isMobile.value ? 8 : 12,
          },
        },
        grid: {
          color: isDark.value ? '#2F2F2F' : '#DFE0E2',
          drawBorder: false,
        },
        border: {
          display: false,
          dash: [2, 2],
        },
      },
    },
    plugins: {
      tooltip: {
        boxPadding: 4,
        usePointStyle: true,
        bodySpacing: 8,
        callbacks: {
          title(context: any) {
            return `Utilization: ${context[0].label}`
          },
          label(context: any) {
            const data = dataPoints.value[context.dataIndex]
            return `${context?.dataset?.label}: ${truncatePercent(Number(data?.borrowAPY) || 0)}%`
          },
        },
      },
      legend: {
        display: false,
      },
      annotation: {
        annotations: {
          current: {
            type: 'line',
            scaleID: 'x',
            value: (Number(currentUtilization.value) || 0) * 100 * 2,
            borderColor: '#006CE4',
            borderDash: [2, 2],
            label: {
              display: true,
              content: `Current: ${(currentUtilization.value * 100).toFixed(2)}%`,
              enabled: true,
              position: 'center',
              backgroundColor: '#006CE4',
              color: '#fff',
              borderRadius: 50,
              borderWidth: 2,
              borderColor: '#006CE4',
              font: {
                size: 10,
                weight: 'bold',
              },
              yAdjust: -25,
            },
          },
          optimal: {
            type: 'line',
            scaleID: 'x',
            value: (Number(params.value?.optimalUtil) || 0) * 100 * 2, // annotation x position
            borderColor: '#FFD101',
            borderWidth: 2,
            borderDash: [2, 2],
            label: {
              display: true,
              content: `Optimal: ${(Number(params.value?.optimalUtil) || 0) * 100}%`,
              enabled: true,
              position: 'center',
              backgroundColor: '#FFD101',
              color: '#111',
              borderRadius: 50,
              borderWidth: 2,
              borderColor: '#FFD101',
              font: {
                size: 10,
                weight: 'bold',
              },
              yAdjust: -25,
            },
          },
        },
      },
    },
  }
})
</script>

<template>
  <div
    :key="dataPoints.length"
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
}
</style>
