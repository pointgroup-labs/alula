export const DEVNET_RPC_URL = 'https://horizon-futurenet.stellar.org'
export const TESTNET_RPC_URL = 'https://horizon-testnet.stellar.org'
export const PUBLIC_RPC_URL = 'https://horizon.stellar.org'

export const SOROBAN_TESTNET_RPC_URL = 'https://soroban-testnet.stellar.org'
export const SOROBAN_PUBLIC_RPC_URL = 'https://soroban.stellar.org'

export const RPC_URLS: Record<string, string> = {
  testnet: TESTNET_RPC_URL,
  devnet: DEVNET_RPC_URL,
  public: PUBLIC_RPC_URL,
}

export const SOROBAN_RPC_URLS: Record<string, string> = {
  testnet: SOROBAN_TESTNET_RPC_URL,
  public: SOROBAN_PUBLIC_RPC_URL,
}

export const MARKET_CONTRACT_ID = 'CBTRCEVHNOSXSX2R4WF4ZUFVL2QPEFR3LLHT2I2GYW36H6ROIYU3ONXD'

export const SOROSWAP_PROVIDER_ADDRESS = 'CCBMBQGNR3NCR2FPN3AYQMJ65UMGQHCOJUB34RQCGIYMX77D5HUUSQDG'
export const AQUA_PROVIDER_ADDRESS = 'CAAMNEXA7BOLMJKHDWNWLW6NQONLW3D6EXIKBBDJJEIDOGJYXRD7PJG4'

export const MAX_I128 = 170_141_183_460_469_231_731_687_303_715_884_105_727n

export const CONTRACT_ID: Record<string, string> = {
  testnet: MARKET_CONTRACT_ID,
}

export const SWAP_PROVIDERS = {
  aquarius: AQUA_PROVIDER_ADDRESS,
  soroswap: SOROSWAP_PROVIDER_ADDRESS,
}
