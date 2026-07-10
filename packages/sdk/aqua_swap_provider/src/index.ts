import { Buffer } from "buffer";
import { Address } from "@stellar/stellar-sdk";
import {
  AssembledTransaction,
  Client as ContractClient,
  ClientOptions as ContractClientOptions,
  MethodOptions,
  Result,
  Spec as ContractSpec,
} from "@stellar/stellar-sdk/contract";
import type {
  u32,
  i32,
  u64,
  i64,
  u128,
  i128,
  u256,
  i256,
  Option,
  Timepoint,
  Duration,
} from "@stellar/stellar-sdk/contract";
export * from "@stellar/stellar-sdk";
export * as contract from "@stellar/stellar-sdk/contract";
export * as rpc from "@stellar/stellar-sdk/rpc";

if (typeof window !== "undefined") {
  //@ts-ignore Buffer exists
  window.Buffer = window.Buffer || Buffer;
}




export type DataKey = {tag: "Admin", values: void} | {tag: "Pool", values: readonly [string, string]};

export const ASPError = {
  1: {message:"OverOrUnderflow"},
  2: {message:"InvalidPath"},
  3: {message:"InvalidSwapResult"},
  4: {message:"NegativeAmount"},
  5: {message:"TokenNotFoundInPool"},
  6: {message:"AmountTooLarge"},
  7: {message:"PoolNotFound"},
  8: {message:"DuplicatePool"},
  9: {message:"IdenticalTokens"},
  10: {message:"Unauthorized"}
}

export interface Client {
  /**
   * Construct and simulate a swap_exact transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  swap_exact: ({user, path, amount_in, min_amount_out}: {user: string, path: Array<string>, amount_in: i128, min_amount_out: i128}, options?: MethodOptions) => Promise<AssembledTransaction<i128>>

  /**
   * Construct and simulate a get_amount_in transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_amount_in: ({path, amount_out}: {path: Array<string>, amount_out: i128}, options?: MethodOptions) => Promise<AssembledTransaction<i128>>

  /**
   * Construct and simulate a add_asset_pool transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  add_asset_pool: ({token_a, token_b, pool_addr}: {token_a: string, token_b: string, pool_addr: string}, options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a get_amount_out transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_amount_out: ({path, amount_in}: {path: Array<string>, amount_in: i128}, options?: MethodOptions) => Promise<AssembledTransaction<i128>>

  /**
   * Construct and simulate a swap_for_exact transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  swap_for_exact: ({user, path, max_amount_in, amount_out}: {user: string, path: Array<string>, max_amount_in: i128, amount_out: i128}, options?: MethodOptions) => Promise<AssembledTransaction<i128>>

}
export class Client extends ContractClient {
  static async deploy<T = Client>(
        /** Constructor/Initialization Args for the contract's `__constructor` method */
        {admin, pools}: {admin: string, pools: Map<readonly [string, string], string>},
    /** Options for initializing a Client as well as for calling a method, with extras specific to deploying. */
    options: MethodOptions &
      Omit<ContractClientOptions, "contractId"> & {
        /** The hash of the Wasm blob, which must already be installed on-chain. */
        wasmHash: Buffer | string;
        /** Salt used to generate the contract's ID. Passed through to {@link Operation.createCustomContract}. Default: random. */
        salt?: Buffer | Uint8Array;
        /** The format used to decode `wasmHash`, if it's provided as a string. */
        format?: "hex" | "base64";
      }
  ): Promise<AssembledTransaction<T>> {
    return ContractClient.deploy({admin, pools}, options)
  }
  constructor(public readonly options: ContractClientOptions) {
    super(
      new ContractSpec([ "AAAAAgAAAAAAAAAAAAAAB0RhdGFLZXkAAAAAAgAAAAAAAAAAAAAABUFkbWluAAAAAAAAAQAAAAAAAAAEUG9vbAAAAAIAAAATAAAAEw==",
        "AAAABAAAAAAAAAAAAAAACEFTUEVycm9yAAAACgAAAAAAAAAPT3Zlck9yVW5kZXJmbG93AAAAAAEAAAAAAAAAC0ludmFsaWRQYXRoAAAAAAIAAAAAAAAAEUludmFsaWRTd2FwUmVzdWx0AAAAAAAAAwAAAAAAAAAOTmVnYXRpdmVBbW91bnQAAAAAAAQAAAAAAAAAE1Rva2VuTm90Rm91bmRJblBvb2wAAAAABQAAAAAAAAAOQW1vdW50VG9vTGFyZ2UAAAAAAAYAAAAAAAAADFBvb2xOb3RGb3VuZAAAAAcAAAAAAAAADUR1cGxpY2F0ZVBvb2wAAAAAAAAIAAAAAAAAAA9JZGVudGljYWxUb2tlbnMAAAAACQAAAAAAAAAMVW5hdXRob3JpemVkAAAACg==",
        "AAAAAAAAAAAAAAAKc3dhcF9leGFjdAAAAAAABAAAAAAAAAAEdXNlcgAAABMAAAAAAAAABHBhdGgAAAPqAAAAEwAAAAAAAAAJYW1vdW50X2luAAAAAAAACwAAAAAAAAAObWluX2Ftb3VudF9vdXQAAAAAAAsAAAABAAAACw==",
        "AAAAAAAAAAAAAAANX19jb25zdHJ1Y3RvcgAAAAAAAAIAAAAAAAAABWFkbWluAAAAAAAAEwAAAAAAAAAFcG9vbHMAAAAAAAPsAAAD7QAAAAIAAAATAAAAEwAAABMAAAAA",
        "AAAAAAAAAAAAAAANZ2V0X2Ftb3VudF9pbgAAAAAAAAIAAAAAAAAABHBhdGgAAAPqAAAAEwAAAAAAAAAKYW1vdW50X291dAAAAAAACwAAAAEAAAAL",
        "AAAAAAAAAAAAAAAOYWRkX2Fzc2V0X3Bvb2wAAAAAAAMAAAAAAAAAB3Rva2VuX2EAAAAAEwAAAAAAAAAHdG9rZW5fYgAAAAATAAAAAAAAAAlwb29sX2FkZHIAAAAAAAATAAAAAA==",
        "AAAAAAAAAAAAAAAOZ2V0X2Ftb3VudF9vdXQAAAAAAAIAAAAAAAAABHBhdGgAAAPqAAAAEwAAAAAAAAAJYW1vdW50X2luAAAAAAAACwAAAAEAAAAL",
        "AAAAAAAAAAAAAAAOc3dhcF9mb3JfZXhhY3QAAAAAAAQAAAAAAAAABHVzZXIAAAATAAAAAAAAAARwYXRoAAAD6gAAABMAAAAAAAAADW1heF9hbW91bnRfaW4AAAAAAAALAAAAAAAAAAphbW91bnRfb3V0AAAAAAALAAAAAQAAAAs=" ]),
      options
    )
  }
  public readonly fromJSON = {
    swap_exact: this.txFromJSON<i128>,
        get_amount_in: this.txFromJSON<i128>,
        add_asset_pool: this.txFromJSON<null>,
        get_amount_out: this.txFromJSON<i128>,
        swap_for_exact: this.txFromJSON<i128>
  }
}