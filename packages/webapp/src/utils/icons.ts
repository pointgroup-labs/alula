import type { TableAsset } from '~/types/table'
import aquaIcon from '~/assets/img/tokens/aqua.webp'
import eurcIcon from '~/assets/img/tokens/eurc.webp'
import xlmIcom from '~/assets/img/tokens/stellar.webp'
import usdcIcon from '~/assets/img/tokens/usdc.webp'

export const TOKEN_ICONS: Record<string, string> = {
  xlm: xlmIcom,
  usdc: usdcIcon,
  eurc: eurcIcon,
  aqua: aquaIcon,
}

export function getTokenIcon(token: string): string {
  return TOKEN_ICONS[token.toLowerCase()] || TOKEN_ICONS.xlm || ''
}

export function getTokenName(token: string) {
  return token === 'native' ? 'Stellar' : token
}

export function getTokenSymbol(token: string) {
  return token === 'native' ? 'XLM' : token
}

export function getFullTokenData(symbol: string): TableAsset['asset'] {
  return {
    name: getTokenName(symbol),
    symbol: getTokenSymbol(symbol),
    icon: getTokenIcon(symbol),
  }
}
