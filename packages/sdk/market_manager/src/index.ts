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




/**
 * Market Manager Contract Error
 */
export const MMCError = {
  1: {message:"InvalidInputAmount"},
  1000: {message:"MarketAlreadyExists"},
  1001: {message:"InvalidMarketState"}
}

export type DataKey = {tag: "Admin", values: void} | {tag: "MarketContractWasmHash", values: void} | {tag: "MarketList", values: void};


export interface Config {
  admin: string;
  market_contract_wasm_hash: Buffer;
}

export interface Client {
  /**
   * Construct and simulate a deploy transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  deploy: ({salt, market_admin, name, oracle, swap_provider, insurance_fund, max_positions, min_collateral_value_cents, insolvency_ltv_bps, update_in_queue_period}: {salt: Buffer, market_admin: string, name: string, oracle: string, swap_provider: string, insurance_fund: string, max_positions: u32, min_collateral_value_cents: i128, insolvency_ltv_bps: i128, update_in_queue_period: Option<u64>}, options?: MethodOptions) => Promise<AssembledTransaction<Result<string>>>

  /**
   * Construct and simulate a get_markets transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_markets: (options?: MethodOptions) => Promise<AssembledTransaction<Map<string, void>>>

  /**
   * Construct and simulate a get_config transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_config: (options?: MethodOptions) => Promise<AssembledTransaction<Config>>

  /**
   * Construct and simulate a upgrade transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  upgrade: ({new_wasm_hash}: {new_wasm_hash: Buffer}, options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a upgrade_deployed_markets transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  upgrade_deployed_markets: ({new_market_contract_wasm_hash}: {new_market_contract_wasm_hash: Buffer}, options?: MethodOptions) => Promise<AssembledTransaction<null>>

}
export class Client extends ContractClient {
  static async deploy<T = Client>(
        /** Constructor/Initialization Args for the contract's `__constructor` method */
        {admin, market_contract_wasm_hash}: {admin: string, market_contract_wasm_hash: Buffer},
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
    return ContractClient.deploy({admin, market_contract_wasm_hash}, options)
  }
  constructor(public readonly options: ContractClientOptions) {
    super(
      new ContractSpec([ "AAAAAAAAAAAAAAAGZGVwbG95AAAAAAAKAAAAAAAAAARzYWx0AAAD7gAAACAAAAAAAAAADG1hcmtldF9hZG1pbgAAABMAAAAAAAAABG5hbWUAAAAQAAAAAAAAAAZvcmFjbGUAAAAAABMAAAAAAAAADXN3YXBfcHJvdmlkZXIAAAAAAAATAAAAAAAAAA5pbnN1cmFuY2VfZnVuZAAAAAAAEwAAAAAAAAANbWF4X3Bvc2l0aW9ucwAAAAAAAAQAAAAAAAAAGm1pbl9jb2xsYXRlcmFsX3ZhbHVlX2NlbnRzAAAAAAALAAAAAAAAABJpbnNvbHZlbmN5X2x0dl9icHMAAAAAAAsAAAAAAAAAFnVwZGF0ZV9pbl9xdWV1ZV9wZXJpb2QAAAAAA+gAAAAGAAAAAQAAA+kAAAATAAAH0AAAAAhNTUNFcnJvcg==",
        "AAAAAAAAAAAAAAALZ2V0X21hcmtldHMAAAAAAAAAAAEAAAPsAAAAEwAAA+0AAAAA",
        "AAAAAAAAAAAAAAAKZ2V0X2NvbmZpZwAAAAAAAAAAAAEAAAfQAAAABkNvbmZpZwAA",
        "AAAAAAAAAAAAAAANX19jb25zdHJ1Y3RvcgAAAAAAAAIAAAAAAAAABWFkbWluAAAAAAAAEwAAAAAAAAAZbWFya2V0X2NvbnRyYWN0X3dhc21faGFzaAAAAAAAA+4AAAAgAAAAAA==",
        "AAAAAAAAAAAAAAAHdXBncmFkZQAAAAABAAAAAAAAAA1uZXdfd2FzbV9oYXNoAAAAAAAD7gAAACAAAAAA",
        "AAAAAAAAAAAAAAAYdXBncmFkZV9kZXBsb3llZF9tYXJrZXRzAAAAAQAAAAAAAAAdbmV3X21hcmtldF9jb250cmFjdF93YXNtX2hhc2gAAAAAAAPuAAAAIAAAAAA=",
        "AAAABAAAAB1NYXJrZXQgTWFuYWdlciBDb250cmFjdCBFcnJvcgAAAAAAAAAAAAAITU1DRXJyb3IAAAADAAAAAAAAABJJbnZhbGlkSW5wdXRBbW91bnQAAAAAAAEAAAAAAAAAE01hcmtldEFscmVhZHlFeGlzdHMAAAAD6AAAAAAAAAASSW52YWxpZE1hcmtldFN0YXRlAAAAAAPp",
        "AAAAAgAAAAAAAAAAAAAAB0RhdGFLZXkAAAAAAwAAAAAAAAAAAAAABUFkbWluAAAAAAAAAAAAAAAAAAAWTWFya2V0Q29udHJhY3RXYXNtSGFzaAAAAAAAAAAAAAAAAAAKTWFya2V0TGlzdAAA",
        "AAAAAQAAAAAAAAAAAAAABkNvbmZpZwAAAAAAAgAAAAAAAAAFYWRtaW4AAAAAAAATAAAAAAAAABltYXJrZXRfY29udHJhY3Rfd2FzbV9oYXNoAAAAAAAD7gAAACA=" ]),
      options
    )
  }
  public readonly fromJSON = {
    deploy: this.txFromJSON<Result<string>>,
        get_markets: this.txFromJSON<Map<string, void>>,
        get_config: this.txFromJSON<Config>,
        upgrade: this.txFromJSON<null>,
        upgrade_deployed_markets: this.txFromJSON<null>
  }
}