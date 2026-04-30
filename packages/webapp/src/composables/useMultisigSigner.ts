/**
 * Bridges the existing `StellarWalletsKit` instance (held in
 * `useConnectionStore().kit`) to the `signEnvelopeXdr` shape that
 * `signProposal` from the multisig lib expects.
 *
 * Centralising this means the sign page never reaches into the kit
 * directly — it depends only on the multisig public surface.
 */

import type { SignProposalInput } from '~/utils/multisig'

export function useMultisigSigner() {
  const connection = useConnectionStore()
  const wallet = useWallet()

  const signEnvelopeXdr: SignProposalInput['signEnvelopeXdr'] = async (xdrBase64, networkPassphrase) => {
    if (!connection.kit) {
      throw new Error('Wallet kit not initialised — connect a wallet first')
    }
    if (!wallet.publicKey) {
      throw new Error('No wallet connected — connect a wallet first')
    }

    const result = await connection.kit.signTransaction(xdrBase64, {
      address: wallet.publicKey,
      networkPassphrase,
    })

    return {
      signedXdr: result.signedTxXdr,
      // Some wallets echo back a different key than the one we requested
      // (multi-account wallets). We trust the kit's reported signer.
      signerPubkey: result.signerAddress ?? wallet.publicKey,
    }
  }

  return { signEnvelopeXdr }
}
