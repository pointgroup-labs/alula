import { getRuntimeConfig } from '~/utils/config'

const config = getRuntimeConfig()

export const RPC_NETWORK = config.NUXT_PUBLIC_RPC || 'testnet'

// intervals
export const RELOAD_FEE_INTERVAL = 60_000

export const EXPLORER_LINK = 'https://stellar.expert/'
