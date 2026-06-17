import type { FarmsClient, FarmState } from '@alula/farms-sdk'
import type { FarmReward } from '@alula/farms-sdk/dist/types'
import type { RewardsTableItem } from '~/types/table'

export const useFarmsStore = defineStore('farms', () => {
  const state = reactive<FarmsStoreState>({
    loading: false,
    loadingRewards: false,
    claiming: false,
    claimFarmId: undefined,
    farms: new Map(),
    rewards: new Map(),
  })

  const { publicKey, balances } = useWalletComposable()
  const { getTokenByAddress } = useTokensStore()
  const { getAssetPrice } = usePriceStore()
  const clientStore = useClientStore()
  const marketsStore = useMarketsStore()
  const connectionStore = useConnectionStore()

  const toast = useToast()

  const { addTrustLine } = useMarketActions()

  const kit = computed(() => connectionStore.kit)

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

        for (const [index, rewardItem] of farmReward.reward.entries()) {
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
            farmId,
            rewardIndex: index,
          })
        }
      }
    }

    return result
  })

  async function claim(data: RewardsTableItem) {
    const market = data.market
    const farm_id = data.farmId
    const reward_index = data.rewardIndex

    let trustlineToast
    let claimToast

    try {
      state.claiming = true
      state.claimFarmId = farm_id
      const client = state.farms.get(market)?.client

      if (!publicKey.value) {
        return
      }

      if (!client) {
        throw new Error('No client. Please, try again or refresh the page')
      }

      const isTrustline = balances.value?.find((b: any) => b.asset_code.toLowerCase() === data.asset?.symbol?.toLowerCase())

      if (!isTrustline) {
        trustlineToast = await toast.create({
          title: 'Add Trustline',
          body: `You need to add trustline for ${data.asset?.symbol}`,
          modelValue: 30_000,
          variant: 'info',
          noProgress: false,
        })
        await addTrustLine(data.asset!.symbol, data.asset!.assetIssuer)
        toast.create({
          title: 'Add Trustline Success',
          body: `You added trustline for ${data.asset?.symbol}. Now you can claim rewards!`,
          variant: 'success',
        })
        return
      }

      claimToast = await toast.create({
        title: `Claiming ${data.asset?.symbol}`,
        body: `Claiming ${formatPrice(data.pending.amount, 5, 5)} ${data.asset?.symbol}...`,
        variant: 'info',
        noProgress: false,
      })

      await client?.claimRewards(
        publicKey.value,
        farm_id,
        reward_index,
        kit.value,
      )
      await getRewards()
      await toast.create({
        title: `Claim Success`,
        body: `You claimed ${formatPrice(data.pending.amount, 5, 5)} ${data.asset?.symbol}`,
        variant: 'success',
      })
    } catch (error: any) {
      console.error(error)
      toast.create({
        title: `Claim Error`,
        body: toast.parseErrorMessage(error),
        variant: 'danger',
        modelValue: 5000,
      })
    } finally {
      trustlineToast?.dismiss()
      claimToast?.dismiss()
      state.claiming = false
      state.claimFarmId = undefined
    }
  }

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
    } catch (error: any) {
      console.error(error)

      toast.create({
        title: 'Farms Client Error',
        body: toast.parseErrorMessage(error),
        variant: 'danger',
        modelValue: 5000,
      })
    } finally {
      state.loading = false
    }
  }, {
    immediate: true,
  })

  watchDebounced([publicKey, () => state.farms], async () => {
    await getRewards()
  }, { debounce: 100 })

  function getMarketFarms(marketName: string) {
    return state.farms.get(marketName)?.data
  }

  async function getRewards() {
    if (state.farms.size === 0 || !publicKey.value) {
      state.rewards = new Map()
      return
    }

    try {
      state.loadingRewards = true
      const entries = await Promise.all(
        [...state.farms.entries()].map(async ([marketName, farm]) => {
          const rewards = await farm.client?.getUserRewards(publicKey.value) ?? []
          return [marketName, rewards] as const
        }),
      )

      state.rewards = new Map(entries)

      console.log('%c[Farms Rewards]', 'color: #2ced53', state.rewards)
    } catch (error: any) {
      console.error(error)
      toast.create({
        title: `Farms Rewards Error`,
        body: toast.parseErrorMessage(error),
        variant: 'danger',
        modelValue: 5000,
      })
    } finally {
      state.loadingRewards = false
    }
  }

  return {
    state,
    preparedRewards,
    farmStrategyByMarket,

    claim,
    getRewards,
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
  claiming: boolean
  claimFarmId?: string
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
  farmId: string
  rewardIndex: number
}
