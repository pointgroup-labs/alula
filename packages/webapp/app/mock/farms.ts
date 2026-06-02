export const mockFarm: FarmView = {
  address: 'CBFARMUSDCXLM',

  token: {
    symbol: 'USDC',
    icon: '/tokens/usdc.webp',
    decimals: 7,
    address: 'CBIELTK...',
  },

  totalStaked: '125000000000',
  tvlUsd: '12500000',

  numUsers: 1842,

  isFrozen: false,

  apr: 12.4,

  rewards: [
    {
      token: {
        symbol: 'ALULA',
        icon: '/tokens/alula.webp',
        decimals: 7,
        address: 'CALULA...',
      },

      rewardType: 'proportional',

      rewardsAvailable: '5000000000',
      rewardsIssuedCumulative: '1200000000',

      apr: 12.4,

      emissionPerDay: '25000000',

      schedule: [
        {
          start: 1_750_000_000,
          emissionPerSecond: '1000000',
        },
      ],
    },
  ],

  config: {
    depositCap: '500000000000',

    minStakeAmount: '1000000',

    lockingMode: 'continuous',

    lockingDuration: 2_592_000,

    warmup: 86_400,

    cooldown: 86_400,

    earlyWithdrawalPenaltyBps: 500,

    treasuryFeeBps: 1000,
  },
}
