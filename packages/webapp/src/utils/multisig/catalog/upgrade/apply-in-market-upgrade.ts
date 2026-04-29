import type { FunctionDef } from '../../types'

export const applyInMarketUpgrade: FunctionDef<Record<string, never>, { queued_wasm_hash: string }> = {
  multisig: 'upgrade',
  contract: 'market_manager',
  function: 'apply_in_market_upgrade',
  id: 'market_manager.apply_in_market_upgrade',
  argSchema: {},
  // The function itself is not "timelocked" by contract logic; the timelock is
  // encoded in the tx envelope's minTime. The flag here means "this entry must
  // be composed with a future minTime" — the build layer enforces it.
  isTimelocked: true,
  renderSummary: (_args, snapshot) => ({
    title: 'Apply queued Market WASM upgrade',
    rows: [
      {
        label: 'Queued WASM hash',
        after: snapshot?.queued_wasm_hash ?? '(read from chain at submit time)',
        severity: 'critical',
      },
    ],
  }),
}
