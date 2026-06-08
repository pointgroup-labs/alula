import { CHART_FILTERS } from '~/config'

export function useChartFilter(filters = CHART_FILTERS, defaultActiveFilterIndex: number = 0) {
  const activeFilter = ref<{ label: string, value: string | number }>(filters[defaultActiveFilterIndex]!)

  function filterData(data: any) {
    const now = new Date()
    return [...data]
      ?.toSorted((a, b) => new Date(a.timestamp).getTime() - new Date(b.timestamp).getTime())
      .filter((item) => {
        const itemDate = new Date(item.timestamp)

        if (activeFilter.value?.value === 1) {
          const refDate = new Date(now)
          refDate.setDate(now.getDate() - 1)
          return itemDate >= refDate && itemDate <= now
        }

        if (activeFilter.value?.value === 7) {
          const refDate = new Date(now)
          refDate.setDate(now.getDate() - 7)
          return itemDate >= refDate && itemDate <= now
        }

        if (activeFilter.value?.value === 31) {
          const refDate = new Date(now)
          refDate.setMonth(now.getMonth() - 1)
          return itemDate >= refDate && itemDate <= now
        }

        if (activeFilter.value?.value === 180) {
          const refDate = new Date(now)
          refDate.setMonth(now.getMonth() - 6)
          return itemDate >= refDate && itemDate <= now
        }

        if (activeFilter.value?.value === 360) {
          const refDate = new Date(now)
          refDate.setMonth(now.getMonth() - 12)
          return itemDate >= refDate && itemDate <= now
        }

        return false
      })
  }

  return {
    filters,
    activeFilter,

    filterData,
  }
}
