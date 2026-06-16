import type { FarmsClient, FarmState } from '@alula/farms-sdk'
import type { FarmReward } from '@alula/farms-sdk/dist/types'

export const useFarmsStore = defineStore('farms', () => {
  const state = reactive<FarmsStoreState>({
    loading: false,
    loadingRewards: false,
    farms: new Map(),
    rewards: new Map(),
  })

  const { publicKey } = useWalletComposable()
  const { getTokenByAddress } = useTokensStore()
  const { getAssetPrice } = usePriceStore()
  const clientStore = useClientStore()
  const marketsStore = useMarketsStore()

  const toast = useToast()

  const markets = computed(() => marketsStore.state.markets)

  const farmStrategyByMarket = computed(() => {
    const result = new Map<string, Map<string, StrategyType>>()

    for (const [marketName, market] of Object.entries(marketsStore.state.markets)) {
      const pools = market?.marketState?.pools_data ?? []
      const strategies = new Map<string, StrategyType>()

      for (const { pool } of pools) {
        const borrowFarmId = toFarmId(pool.farm_debt)
        const supplyFarmId = toFarmId(pool.farm_supply)

        if (borrowFarmId) {
          strategies.set(borrowFarmId, 'borrow')
        }

        if (supplyFarmId) {
          strategies.set(supplyFarmId, 'supply')
        }
      }

      result.set(marketName, strategies)
    }

    return result
  })

  const preparedRewards = computed(() => {
    const result: PreparedReward[] = []

    for (const [marketName, farmRewards] of state.rewards) {
      const strategyByFarmId = farmStrategyByMarket.value.get(marketName)

      for (const farmReward of farmRewards) {
        const farmId = farmReward.farm_id
        const strategyType = strategyByFarmId?.get(farmId)

        for (const rewardItem of farmReward.reward) {
          const [assetAddress, rawAmount] = rewardItem

          if (rawAmount <= 0) {
            continue
          }

          const asset = getTokenByAddress(assetAddress)
          const price = getAssetPrice(asset?.symbol) ?? 0
          const decimals = asset?.decimals ?? 7

          const amount = Number(bigintToNumber(rawAmount, decimals))
          const amountUsd = amount * price

          result.push({
            market: marketName,
            asset,
            rawAmount,
            amount,
            amountUsd,
            strategyType,
          })
        }
      }
    }

    return result
  })

  watch(markets, async (m) => {
    if (Object.keys(m).length === 0 || state.farms.size > 0) {
      return
    }

    const values = Object.values(m)

    try {
      state.loading = true

      const entries = await Promise.all(
        values.map(async (market) => {
          const marketName = market.marketName

          const farmsContractAddress = await market.client?.market?.getFarmsContractAddress()

          if (!farmsContractAddress) {
            return null
          }

          const farmsClient = await clientStore.initFarmsClient(farmsContractAddress)

          const marketFarms = await farmsClient?.getMarketFarms() ?? []

          if (marketFarms.length === 0) {
            return null
          }

          return [
            marketName,
            {
              client: farmsClient,
              data: marketFarms,
            },
          ] as const
        }),
      )

      state.farms = new Map(
        entries.filter((entry): entry is NonNullable<typeof entry> => entry !== null),
      )

      console.log('%c[Farms]', 'color: #2ced53', state.farms)
    } catch (error) {
      console.error(error)

      toast.create({
        title: 'Farms Client Error',
        body: String((error as any)?.message || error),
        variant: 'danger',
        modelValue: 5000,
      })
    } finally {
      state.loading = false
    }
  }, {
    immediate: true,
  })

  watchDebounced([publicKey, () => state.farms], async ([pubkey, farms]) => {
    if (farms.size === 0 || !pubkey) {
      state.rewards = new Map()
      return
    }

    try {
      state.loadingRewards = true
      const entries = await Promise.all(
        [...farms.entries()].map(async ([marketName, farm]) => {
          const rewards = await farm.client?.getUserRewards(pubkey) ?? []
          return [marketName, rewards] as const
        }),
      )

      state.rewards = new Map(entries)

      console.log('%c[Farms Rewards]', 'color: #2ced53', state.rewards)
    } catch (error) {
      console.error(error)
      toast.create({
        title: `Farms Rewards Error`,
        body: String((error as any)?.message || error),
        variant: 'danger',
        modelValue: 5000,
      })
    } finally {
      state.loadingRewards = false
    }
  }, { debounce: 100 })

  function getMarketFarms(marketName: string) {
    return state.farms.get(marketName)?.data
  }

  return {
    state,
    preparedRewards,
    farmStrategyByMarket,

    getMarketFarms,
  }
})

export function toFarmId(value?: Uint8Array | number[] | null) {
  if (!value) {
    return
  }

  return Buffer.from(value).toString('hex')
}

export type FarmsStoreState = {
  loading: boolean
  loadingRewards: boolean
  farms: Map<string, {
    data: FarmState[]
    client?: FarmsClient
  }>
  rewards: Map<string, FarmReward[]>
}

export type StrategyType = 'supply' | 'borrow'

type PreparedReward = {
  market: string
  asset?: TokenItem
  rawAmount: bigint
  amount: number
  amountUsd: number
  strategyType?: StrategyType
}
