import type { TableAsset } from '~/types/table'

export function useAdditionalApy() {
  const additionalMarketsData = ref<{ marketName: string, [key: string]: any }[]>([])

  function generateMockAdditionalData(markets: any[]) {
    if (!markets) {
      return []
    }
    const assetNames = [...new Set(markets.flatMap(m => m.assets.map((a: TableAsset['asset']) => a.symbol)))]
    additionalMarketsData.value = markets?.map((m) => {
      const itemsData = m.tableItems.map((item: { pool_address: any, raw?: any }) => {
        const randomNum = Math.round(Math.random())
        const randomDeposit = Math.round(Math.random())
        const randomBorrow = Math.round(Math.random())
        const data = []
        if (randomNum === 1) {
          if (randomDeposit === 1) {
            for (let j = 0; j < Math.ceil(Math.random() * 3); j++) {
              data.push({
                name: 'deposit',
                additional_apy: Math.random() * 3,
                token_symbol: assetNames[Math.ceil(Math.random() * assetNames.length) - 1],
              })
            }
          }
          if (randomBorrow === 1) {
            for (let k = 0; k < Math.ceil(Math.random() * 3); k++) {
              data.push({
                name: 'borrow',
                additional_apy: Math.random() * 3,
                token_symbol: assetNames[Math.ceil(Math.random() * assetNames.length) - 1],
              })
            }
          }
        }
        return [item.pool_address, data]
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
