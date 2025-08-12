import eurcIcon from '~/assets/img/tokens/eurc.png'
import xlmIcom from '~/assets/img/tokens/stellar.png'
import usdcIcon from '~/assets/img/tokens/usdc.svg'

export const TOKEN_ICONS: Record<string, string> = {
  xlm: xlmIcom,
  usdc: usdcIcon,
  eurc: eurcIcon,
}

export function getTokenIcon(token: string) {
  return TOKEN_ICONS[token.toLowerCase()] ?? TOKEN_ICONS.xlm
}

export function getTokenName(token: string) {
  return token === 'XLM' ? 'Stellar' : token
}
