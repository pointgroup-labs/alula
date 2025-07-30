import { getRuntimeConfig } from '~/utils/config'

const config = getRuntimeConfig()

export const RPC_NETWORK = config.NUXT_PUBLIC_RPC || 'testnet'

// intervals
export const RELOAD_FEE_INTERVAL = 60_000
export const CLEAR_DIALOG_TIMEOUT = 500

export const POOL_REMAINING_BALANCE = 0.01

export const EXPLORER_LINK = 'https://stellar.expert/'
