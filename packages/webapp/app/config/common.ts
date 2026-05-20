import { getRuntimeConfig } from '~/utils/config'

const config = getRuntimeConfig()

export const ALULA_URL = 'https://alula.finance'

export const DOCS_URL = 'https://docs.alula.finance'

export const GITHUB_URL = 'https://github.com/pointgroup-labs/alula'

export const RPC_NETWORK = config.NUXT_PUBLIC_RPC || 'testnet'

// intervals
export const RELOAD_FEE_INTERVAL = 60_000
export const CLEAR_DIALOG_TIMEOUT = 500
export const TRANSACTION_TIMEOUT = 60_000

export const POOL_REMAINING_BALANCE = 0.01

export const EXPLORER_LINK = 'https://stellar.expert/'

export const DOCS_BORROW_RISKS_LINK = 'https://docs.alula.finance/guides/borrow/borrow-related-risks'

// terms & privacy
export const TERMS_OF_SERVICE_DOCS_LINK = 'https://docs.alula.finance/terms-of-service.md'
export const TERMS_OF_SERVICE_LINK = `${ALULA_URL}/terms`
export const PRIVACY_POLICY_LINK = `${ALULA_URL}/privacy`
