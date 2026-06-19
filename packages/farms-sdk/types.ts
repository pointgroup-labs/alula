export type RPCcluster = 'devnet' | 'testnet' | 'public'

export type FarmRewardPair = [string, bigint]

export type FarmReward = {
  farm_id: string
  reward: FarmRewardPair[]
}
