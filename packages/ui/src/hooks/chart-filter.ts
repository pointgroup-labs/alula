import { CHART_FILTERS } from '~/config'

export function useChartFilter(filters = CHART_FILTERS) {
    const activeFilter = ref(filters[0])

    function filterData(data: any) {
        const now = new Date()
        return [...data]
            ?.sort((a, b) => new Date(a.timestamp).getTime() - new Date(b.timestamp).getTime())
            .filter((item) => {
                const itemDate = new Date(item.timestamp)

                if (activeFilter.value.value === 7) {
                    const refDate = new Date(now)
                    refDate.setDate(now.getDate() - 7)
                    return itemDate >= refDate && itemDate <= now
                }

                if (activeFilter.value.value === 31) {
                    const refDate = new Date(now)
                    refDate.setMonth(now.getMonth() - 1)
                    return itemDate >= refDate && itemDate <= now
                }

                if (activeFilter.value.value === 180) {
                    const refDate = new Date(now)
                    refDate.setMonth(now.getMonth() - 6)
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
