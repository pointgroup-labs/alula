export const useFarmsStore = defineStore('farms', () => {
  const state = reactive<FarmsState>({
    loading: false,
    farms: [],
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
