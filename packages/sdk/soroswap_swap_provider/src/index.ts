import type {
  Duration,
  i32,
  i64,
  i128,
  i256,
  Option,
  Timepoint,
  u32,
  u64,
  u128,
  u256,
} from '@stellar/stellar-sdk/contract'
import { Buffer } from 'node:buffer'
import { Address } from '@stellar/stellar-sdk'
import {
  AssembledTransaction,
  Client as ContractClient,
  ClientOptions as ContractClientOptions,
  Spec as ContractSpec,
  MethodOptions,
  Result,
} from '@stellar/stellar-sdk/contract'

export * from '@stellar/stellar-sdk'
export * as contract from '@stellar/stellar-sdk/contract'
export * as rpc from '@stellar/stellar-sdk/rpc'

if (typeof window !== 'undefined') {
  // @ts-ignore Buffer exists
  window.Buffer = window.Buffer || Buffer
}

export const SSPError = {
  1: { message: 'OverOrUnderflow' },
  2: { message: 'InvalidPath' },
  3: { message: 'ZeroSwapResult' },
}

export interface Client {
  /**
   * Construct and simulate a get_router transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_router: (options?: MethodOptions) => Promise<AssembledTransaction<string>>

  /**
   * Construct and simulate a upgrade transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  upgrade: ({ new_contract_wasm_hash}: { new_contract_wasm_hash: Buffer }, options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a swap_exact transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  swap_exact: ({ user, path, amount_in, min_amount_out}: { user: string, path: Array<string>, amount_in: i128, min_amount_out: i128 }, options?: MethodOptions) => Promise<AssembledTransaction<i128>>

  /**
   * Construct and simulate a swap_for_exact transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  swap_for_exact: ({ user, path, amount_in_max, amount_out}: { user: string, path: Array<string>, amount_in_max: i128, amount_out: i128 }, options?: MethodOptions) => Promise<AssembledTransaction<i128>>

  /**
   * Construct and simulate a get_amount_out transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_amount_out: ({ path, amount_in}: { path: Array<string>, amount_in: i128 }, options?: MethodOptions) => Promise<AssembledTransaction<i128>>

  /**
   * Construct and simulate a get_amount_in transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_amount_in: ({ path, amount_out}: { path: Array<string>, amount_out: i128 }, options?: MethodOptions) => Promise<AssembledTransaction<i128>>

}
export class Client extends ContractClient {
  static async deploy<T = Client>(
    /** Constructor/Initialization Args for the contract's `__constructor` method */
    { router, admin}: { router: string, admin: string },
    /** Options for initializing a Client as well as for calling a method, with extras specific to deploying. */
    options: MethodOptions
      & Omit<ContractClientOptions, 'contractId'> & {
        /** The hash of the Wasm blob, which must already be installed on-chain. */
        wasmHash: Buffer | string
        /** Salt used to generate the contract's ID. Passed through to {@link Operation.createCustomContract}. Default: random. */
        salt?: Buffer | Uint8Array
        /** The format used to decode `wasmHash`, if it's provided as a string. */
        format?: 'hex' | 'base64'
      },
  ): Promise<AssembledTransaction<T>> {
    return ContractClient.deploy({ router, admin }, options)
  }

  constructor(public readonly options: ContractClientOptions) {
    super(
      new ContractSpec(['AAAABAAAAAAAAAAAAAAACFNTUEVycm9yAAAAAwAAAAAAAAAPT3Zlck9yVW5kZXJmbG93AAAAAAEAAAAAAAAAC0ludmFsaWRQYXRoAAAAAAIAAAAAAAAADlplcm9Td2FwUmVzdWx0AAAAAAAD',
        'AAAAAAAAAAAAAAANX19jb25zdHJ1Y3RvcgAAAAAAAAIAAAAAAAAABnJvdXRlcgAAAAAAEwAAAAAAAAAFYWRtaW4AAAAAAAATAAAAAA==',
        'AAAAAAAAAAAAAAAKZ2V0X3JvdXRlcgAAAAAAAAAAAAEAAAAT',
        'AAAAAAAAAAAAAAAHdXBncmFkZQAAAAABAAAAAAAAABZuZXdfY29udHJhY3Rfd2FzbV9oYXNoAAAAAAPuAAAAIAAAAAA=',
        'AAAAAAAAAAAAAAAKc3dhcF9leGFjdAAAAAAABAAAAAAAAAAEdXNlcgAAABMAAAAAAAAABHBhdGgAAAPqAAAAEwAAAAAAAAAJYW1vdW50X2luAAAAAAAACwAAAAAAAAAObWluX2Ftb3VudF9vdXQAAAAAAAsAAAABAAAACw==',
        'AAAAAAAAAAAAAAAOc3dhcF9mb3JfZXhhY3QAAAAAAAQAAAAAAAAABHVzZXIAAAATAAAAAAAAAARwYXRoAAAD6gAAABMAAAAAAAAADWFtb3VudF9pbl9tYXgAAAAAAAALAAAAAAAAAAphbW91bnRfb3V0AAAAAAALAAAAAQAAAAs=',
        'AAAAAAAAAAAAAAAOZ2V0X2Ftb3VudF9vdXQAAAAAAAIAAAAAAAAABHBhdGgAAAPqAAAAEwAAAAAAAAAJYW1vdW50X2luAAAAAAAACwAAAAEAAAAL',
        'AAAAAAAAAAAAAAANZ2V0X2Ftb3VudF9pbgAAAAAAAAIAAAAAAAAABHBhdGgAAAPqAAAAEwAAAAAAAAAKYW1vdW50X291dAAAAAAACwAAAAEAAAAL']),
      options,
    )
  }

  public readonly fromJSON = {
    get_router: this.txFromJSON<string>,
    upgrade: this.txFromJSON<null>,
    swap_exact: this.txFromJSON<i128>,
    swap_for_exact: this.txFromJSON<i128>,
    get_amount_out: this.txFromJSON<i128>,
    get_amount_in: this.txFromJSON<i128>,
  }
}
