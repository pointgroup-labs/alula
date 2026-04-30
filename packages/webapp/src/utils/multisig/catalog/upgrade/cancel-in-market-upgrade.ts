import type { FunctionDef } from '../../types'

export const cancelInMarketUpgrade: FunctionDef<Record<string, never>, { queued_wasm_hash: string | null }> = {
  multisig: 'upgrade',
  contract: 'market_manager',
  function: 'cancel_market_upgrade',
  id: 'market_manager.cancel_market_upgrade',
  displayName: 'Cancel queued Market WASM upgrade',
  description: 'Drops the currently-queued Market WASM upgrade before it can be applied. Use to abort a proposal during the timelock window.',
  argSchema: {},
  isTimelocked: false,
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
