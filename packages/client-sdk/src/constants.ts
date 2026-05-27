export const DEVNET_RPC_URL = 'https://horizon-futurenet.stellar.org'
export const TESTNET_RPC_URL = 'https://horizon-testnet.stellar.org'
export const PUBLIC_RPC_URL = 'https://horizon.stellar.org'

export const SOROBAN_TESTNET_RPC_URL = 'https://soroban-testnet.stellar.org'
export const SOROBAN_PUBLIC_RPC_URL = 'https://rpc.lightsail.network'

export const RPC_URLS: Record<string, string> = {
  testnet: TESTNET_RPC_URL,
  devnet: DEVNET_RPC_URL,
  public: PUBLIC_RPC_URL,
}

export const SOROBAN_RPC_URLS: Record<string, string> = {
  testnet: SOROBAN_TESTNET_RPC_URL,
  public: SOROBAN_PUBLIC_RPC_URL,
}

export const MARKET_CONTRACT_ID = 'CDZMGP24Y77FJP2H2247B5AFSEGXYJVQMNOYSU3HPSDQ7LVSV32RHMP5'
export const PUBLIC_CONTRACT_ID = 'CBQNZKCJFGVHUWGKAHT3BD7ZVPRBSQESSRFBYIBQQ46K7VKVOTN3Q6BU'

export const SOROSWAP_PROVIDER_ADDRESS: Record<string, string> = {
  testnet: 'CCBMBQGNR3NCR2FPN3AYQMJ65UMGQHCOJUB34RQCGIYMX77D5HUUSQDG',
  public: '',
}

export const AQUA_PROVIDER_ADDRESS: Record<string, string> = {
  testnet: 'CAAMNEXA7BOLMJKHDWNWLW6NQONLW3D6EXIKBBDJJEIDOGJYXRD7PJG4',
  public: 'CCIKCDHGZ7EZFVL4YK72L6RGIW5IY4PEYCDXSQA5ZEIZTJOCZLEA4TMF',
}

export const MAX_I128 = 170_141_183_460_469_231_731_687_303_715_884_105_727n

export const CONTRACT_ID: Record<string, string> = {
  testnet: MARKET_CONTRACT_ID,
  public: PUBLIC_CONTRACT_ID,
}

export const SWAP_PROVIDERS: Record<string, Record<string, string>> = {
  aquarius: AQUA_PROVIDER_ADDRESS,
  soroswap: SOROSWAP_PROVIDER_ADDRESS,
}

export const VALID_PAIRS = new Set([
  'USDC-XLM',
  'AQUA-USDC',
])
