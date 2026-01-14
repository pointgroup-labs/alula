export function useAdditionalApy() {
  const additionalMarketsData = ref<{ marketName: string, [key: string]: any }[]>([])

  function generateMockAdditionalData(markets: any[]) {
    if (!markets) {
      return []
    }
    additionalMarketsData.value = markets?.map((m) => {
      const itemsData = m.tableItems.map((i: { pool_address: any }) => {
        const randomNum = Math.round(Math.random())
        const randomDeposit = Math.round(Math.random())
        const randomBorrow = Math.round(Math.random())
        const data = []
        if (randomNum === 1) {
          if (randomDeposit === 1) {
            data.push({
              name: 'deposit',
              additional_apy: Math.random() * 5,
            })
          }
          if (randomBorrow === 1) {
            data.push({
              name: 'borrow',
              additional_apy: Math.random() * 3,
            })
          }
        }
        return [i.pool_address, data]
      })

      return {
        marketName: m.marketName,
        data: Object.fromEntries(itemsData),
      }
    })
  }
  return {
    additionalMarketsData,
    generateMockAdditionalData,
  }
}
