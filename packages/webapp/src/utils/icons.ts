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

export const TOKEN_NAMES: Record<string, string> = {
  native: 'Stellar',
  xlm: 'Stellar',
  usdc: 'USD Coin',
  eurc: 'Euro Coin',
  aqua: 'Aqua',
}

export function getTokenIcon(token: string): string {
  return TOKEN_ICONS[token.toLowerCase()] || TOKEN_ICONS.xlm || ''
}

export function getTokenName(token: string): string {
  return TOKEN_NAMES[token.toLowerCase()] ?? TOKEN_NAMES.native ?? ''
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
