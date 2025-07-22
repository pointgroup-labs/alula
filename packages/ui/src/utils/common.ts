import xlmIcom from '~/assets/img/assets/stellar.png'
import usdcIcon from '~/assets/img/assets/usdc.svg'
import { RPC_NETWORK } from '~/config'

export const TOKEN_ICONS: Record<string, string> = {
    xlm: xlmIcom,
    usdc: usdcIcon,
}

export function getTokenIcon(token: string) {
    return TOKEN_ICONS[token.toLowerCase()] ?? TOKEN_ICONS.xlm
}

export function getTokenName(token: string) {
    return token === 'XLM' ? 'Stellar' : token
}

export function generateExplorerLink(hash: string, entityId = 'tx') {
    return `https://stellar.expert/explorer/${RPC_NETWORK}/${entityId}/${hash}`
}
