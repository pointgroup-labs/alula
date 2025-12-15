import { BorrowPosition, DepositPosition, Obligation } from '@alula/market-sdk'

export type RPCcluster = 'devnet' | 'testnet' | 'public'

export type ObligationUI = Record<string, ObligationArray | undefined>

export type ObligationArray = Omit<Obligation, 'borrows' | 'deposits'> & {
  borrows: Array<[string, BorrowPosition]>
  deposits: Array<[string, DepositPosition]>
}
