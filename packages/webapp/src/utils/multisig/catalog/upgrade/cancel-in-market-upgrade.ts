import type { FunctionDef } from '../../types'

export const cancelInMarketUpgrade: FunctionDef<Record<string, never>, { queued_wasm_hash: string | null }> = {
  multisig: 'upgrade',
  contract: 'market_manager',
  function: 'cancel_market_upgrade',
  id: 'market_manager.cancel_market_upgrade',
  displayName: 'Cancel queued Market WASM upgrade',
  description: 'Removes the queued Market WASM upgrade before it can be applied. Use this to abort a queued upgrade while the timelock is still running.',
  argSchema: {},
  isTimelocked: false,
  affectsAllMarkets: true,
  renderSummary: (_args, snapshot) => ({
    title: 'Cancel queued Market WASM upgrade',
    rows: [
      {
        label: 'Currently queued',
        after: snapshot?.queued_wasm_hash ?? '(none)',
        severity: 'warning',
      },
    ],
  }),
}
