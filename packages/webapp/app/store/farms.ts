export const useFarmsStore = defineStore('farms', () => {
  const state = reactive<FarmsState>({
    loading: false,
    farms: [],
  })

  const marketsStore = useMarketsStore()
  const markets = computed(() => marketsStore.state.markets)

  watch(markets, async (m) => {
    if (Object.keys(m).length === 0) {
      return
    }
    const values = Object.values(m)
    const addresses = await Promise.all(values.map(async (val) => {
      return {
        market: val.marketName,
        address: await val.client?.market.getFarmsContractAddress(),
      }
    }))

    console.log('%c[Farms Addresses]', 'color: #1dc978', addresses)
  })

  return {
    state,
  }
})

export type FarmsState = {
  loading: boolean
  farms: FarmView[]
}

export type FarmView = {
  address: string

  token: {
    symbol: string
    icon: string
    decimals: number
    address: string
  }

  totalStaked: string
  tvlUsd: string
  numUsers: number

  isFrozen: boolean

  apr: number

  rewards: RewardView[]

  config: {
    depositCap: string
    minStakeAmount: string

    lockingMode: 'none' | 'continuous' | 'with_expiry'

    lockingDuration: number
    warmup: number
    cooldown: number

    earlyWithdrawalPenaltyBps: number

    treasuryFeeBps: number
  }
}

export type RewardView = {
  token: {
    symbol: string
    icon: string
    decimals: number
    address: string
  }

  rewardType: 'proportional' | 'constant'

  rewardsAvailable: string
  rewardsIssuedCumulative: string

  apr: number

  emissionPerDay: string

  schedule: {
    start: number
    emissionPerSecond: string
  }[]
}

export type FarmingPositionView = {
  owner: string

  activeStake: string

  pendingDeposit: {
    amount: string
    unlockAt: number
  }

  pendingWithdrawal: {
    amount: string
    unlockAt: number
  }

  unclaimedRewards: {
    token: string
    amount: string
    usdValue: string
  }[]

  totalRewardsUsd: string

  lastStakeTs: number
}
