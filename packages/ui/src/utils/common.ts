import xlmIcom from '~/assets/img/assets/stellar.png'
import usdcIcon from '~/assets/img/assets/usdc.svg'

export const TOKEN_ICONS: Record<string, string> = {
    xlm: xlmIcom,
    usdc: usdcIcon,
}

export function getTokenIcon(token: string) {
    return TOKEN_ICONS[token.toLowerCase()] ?? TOKEN_ICONS.xlm
}
