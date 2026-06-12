import type { TableAsset } from '~/types/table'

const DATA_URL = 'https://raw.githubusercontent.com/pointgroup-labs/alula-registry/main/data'
const TOKENS_URL = `${DATA_URL}/tokens.json`

type TokenRegistryItem = {
  id: string
  name: string
  symbol: string
  decimals: number
  token_addresses: {
    testnet: string
    mainnet: string
  }
  icon: string
}

export type TokenItem = {
  id: string
  name: string
  symbol: string
  decimals: number
  tokenAddresses: {
    testnet: string
    mainnet: string
  }
  icon: string
}

function normalizeTokenKey(token: string): string {
  return token.toLowerCase()
}

function normalizeTokenName(name: string): string {
  if (name === name.toLowerCase()) {
    return name.charAt(0).toUpperCase() + name.slice(1)
  }

  return name
}

function normalizeToken(token: TokenRegistryItem): TokenItem {
  return {
    id: normalizeTokenKey(token.id),
    name: normalizeTokenName(token.name),
    symbol: token.symbol,
    decimals: token.decimals,
    tokenAddresses: token.token_addresses,
    icon: `${DATA_URL}${token.icon}`,
  }
}

function setToken(map: Map<string, TokenItem>, token: TokenItem) {
  map.set(token.id, token)
  map.set(normalizeTokenKey(token.symbol), token)

  if (token.id === 'native') {
    map.set('xlm', token)
  }
}

export const useTokensStore = defineStore('tokens', () => {
  const state = reactive({
    tokens: new Map<string, TokenItem>(),
    isLoading: false,
    isLoaded: false,
  })

  const tokensList = computed(() => {
    const uniqueTokens = new Map<string, TokenItem>()

    for (const token of state.tokens.values()) {
      uniqueTokens.set(token.id, token)
    }

    return [...uniqueTokens.values()]
  })

  function getTokenByAddress(address: string) {
    for (const token of state.tokens.values()) {
      if (
        token.tokenAddresses.testnet === address
        || token.tokenAddresses.mainnet === address
      ) {
        return token
      }
    }
  }

  function getToken(token: string): TokenItem | undefined {
    return state.tokens.get(normalizeTokenKey(token))
  }

  function getTokenName(token: string): string {
    return getToken(token)?.name ?? getToken('native')?.name ?? ''
  }

  function getTokenSymbol(token: string): string {
    return getToken(token)?.symbol ?? token
  }

  function getTokenDecimals(token: string): number {
    return getToken(token)?.decimals ?? 7
  }

  function getTokenIcon(token: string): string {
    return getToken(token)?.icon ?? getToken('native')?.icon ?? ''
  }

  function getFullTokenData(symbol: string): TableAsset['asset'] {
    return {
      name: getTokenName(symbol),
      symbol: getTokenSymbol(symbol),
      icon: getTokenIcon(symbol),
      decimals: getTokenDecimals(symbol),
    }
  }

  async function fetchTokens() {
    if (state.isLoading || state.isLoaded) {
      return
    }

    state.isLoading = true

    try {
      const resp = await fetch(TOKENS_URL)

      if (!resp.ok) {
        throw new Error(`Failed to fetch tokens: ${resp.status}`)
      }

      const data = await resp.json() as TokenRegistryItem[]
      const tokens = new Map<string, TokenItem>()

      for (const token of data) {
        setToken(tokens, normalizeToken(token))
      }

      state.tokens = tokens
      state.isLoaded = true
    } finally {
      state.isLoading = false
    }
  }

  onMounted(() => {
    void fetchTokens()
  })

  return {
    state,
    tokensList,
    fetchTokens,
    getToken,
    getTokenIcon,
    getTokenName,
    getTokenSymbol,
    getFullTokenData,
    getTokenByAddress,
  }
})
