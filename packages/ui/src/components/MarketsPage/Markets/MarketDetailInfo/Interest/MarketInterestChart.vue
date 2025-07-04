<script lang="ts" setup>
import type { ChartData, ChartOptions } from 'chart.js'
import { Chart } from 'chart.js'
import annotationPlugin from 'chartjs-plugin-annotation'
import { truncatePercent } from '~/utils'

Chart.register(annotationPlugin)

const { width } = useWindowSize()

const isMobile = computed(() => width.value <= 650)

const modelParams = {
  // Minimum borrow interest rate when utilization is 0%
  baseRate: 0.0002,
  // Utilization rate threshold below which the interest rate increases slowly (linear slope)
  optimalUtil: 0.86,
  // Utilization rate at which the "jump" in interest rate slope begins
  jumpUtil: 0.945,
  // Maximum possible borrow interest rate (when utilization reaches 100%)
  maxRate: 0.9663,
  // Portion of the borrow interest kept by the protocol (not given to suppliers)
  reserveFactor: 0.2,
}

const interestRate = ref(0.0728)

function getBorrowRate(util: number): number {
  const { optimalUtil, jumpUtil, maxRate } = modelParams

  if (util === 0) {
    return 0
  }

  if (util <= optimalUtil) {
    // Без baseRate, как в Camino
    return (util / optimalUtil) * 0.0413
  }

  if (util <= jumpUtil) {
    const slope = (0.0993 - 0.0413) / (jumpUtil - optimalUtil)
    return 0.0413 + (util - optimalUtil) * slope
  }

  const finalSlope = (maxRate - 0.0993) / (1 - jumpUtil)
  return 0.0993 + (util - jumpUtil) * finalSlope
}

// function getSupplyRate(util: number): number {
//   const borrowRate = getBorrowRate(util)
//   return util * borrowRate * (1 - modelParams.reserveFactor)
// }

const dataPoints = computed(() => {
  return Array.from({ length: 201 }, (_, i) => {
    const util = i * 0.005
    return {
      utilization: util * 100,
      borrowAPY: getBorrowRate(util) * 100,
    //   supplyAPY: getSupplyRate(util) * 100,
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
      data: dataPoints.value.map(d => d.borrowAPY),
    },
    // {
    //   type: 'line',
    //   borderColor: '#FFD101',
    //   pointBackgroundColor: '#FFD101',
    //   fill: false,
    //   label: 'Supply APY',
    //   data: dataPoints.value.map(d => d.supplyAPY),
    // },
  ]
}, { immediate: true })

// @ts-expect-error...
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
            return `${context?.dataset?.label}: ${truncatePercent(Number(context?.formattedValue) || 0)}%`
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
            value: interestRate.value * 100 * 2,
            borderColor: '#006CE4',
            borderDash: [2, 2],
            label: {
              display: true,
              content: `Current: ${interestRate.value * 100}%`,
              enabled: true,
              position: 'top',
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
            value: modelParams.optimalUtil * 100 * 2, // annotation x position
            borderColor: '#FFD101',
            borderWidth: 2,
            borderDash: [2, 2],
            label: {
              display: true,
              content: `Optimal: ${modelParams.optimalUtil * 100}%`,
              enabled: true,
              position: 'top',
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
  <div class="market-interest-chart">
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
