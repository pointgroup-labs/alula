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





export interface MarketInitParams {
  bad_debt_lock_d: u64;
  insolvency_ltv_bps: i128;
  is_owned: boolean;
  max_positions: u32;
  min_collateral_value_cents: i128;
  update_in_queue_period: u64;
}

/**
 * Market Manager Contract Error
 */
export const MMCError = {
  1: {message:"InvalidInputAmount"},
  9: {message:"OverOrUnderflow"},
  1000: {message:"MarketAlreadyExists"},
  1001: {message:"InvalidMarketState"},
  1002: {message:"UpgradeAlreadyExists"},
  1003: {message:"UpgradeDoesNotExist"},
  1004: {message:"UpgradeIsNotYetApplicable"}
}

export type DataKey = {tag: "Admin", values: void} | {tag: "MarketsList", values: void} | {tag: "MarketWasmHash", values: void} | {tag: "QueuedInMarketUpgrade", values: void} | {tag: "QueuedInManagerUpgrade", values: void};


export interface Config {
  admin: string;
  market_wasm_hash: Buffer;
}


export interface QueuedInUpgrade {
  queued_in_timestamp: u64;
  wasm_hash: Buffer;
}

export interface Client {
  /**
   * Construct and simulate a deploy transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  deploy: ({salt, market_admin, name, oracle, insurance_fund, params}: {salt: Buffer, market_admin: string, name: string, oracle: string, insurance_fund: string, params: MarketInitParams}, options?: MethodOptions) => Promise<AssembledTransaction<Result<string>>>

  /**
   * Construct and simulate a get_markets transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_markets: (options?: MethodOptions) => Promise<AssembledTransaction<Map<string, void>>>

  /**
   * Construct and simulate a get_config transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_config: (options?: MethodOptions) => Promise<AssembledTransaction<Config>>

  /**
   * Construct and simulate a get_market_wasm_hash transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_market_wasm_hash: (options?: MethodOptions) => Promise<AssembledTransaction<Buffer>>

  /**
   * Construct and simulate a get_queued_in_market_upgrade transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_queued_in_market_upgrade: (options?: MethodOptions) => Promise<AssembledTransaction<Option<QueuedInUpgrade>>>

  /**
   * Construct and simulate a get_queued_in_manager_upgrade transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_queued_in_manager_upgrade: (options?: MethodOptions) => Promise<AssembledTransaction<Option<QueuedInUpgrade>>>

  /**
   * Construct and simulate a queue_in_market_upgrade transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  queue_in_market_upgrade: ({new_wasm_hash}: {new_wasm_hash: Buffer}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a queue_in_manager_upgrade transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  queue_in_manager_upgrade: ({new_wasm_hash}: {new_wasm_hash: Buffer}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a cancel_market_upgrade transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  cancel_market_upgrade: (options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a cancel_manager_upgrade transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  cancel_manager_upgrade: (options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a apply_market_upgrade transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  apply_market_upgrade: (options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a apply_manager_upgrade transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  apply_manager_upgrade: (options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

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
      new ContractSpec([ "AAAAAQAAAAAAAAAAAAAAEE1hcmtldEluaXRQYXJhbXMAAAAGAAAAAAAAAA9iYWRfZGVidF9sb2NrX2QAAAAABgAAAAAAAAASaW5zb2x2ZW5jeV9sdHZfYnBzAAAAAAALAAAAAAAAAAhpc19vd25lZAAAAAEAAAAAAAAADW1heF9wb3NpdGlvbnMAAAAAAAAEAAAAAAAAABptaW5fY29sbGF0ZXJhbF92YWx1ZV9jZW50cwAAAAAACwAAAAAAAAAWdXBkYXRlX2luX3F1ZXVlX3BlcmlvZAAAAAAABg==",
        "AAAAAAAAAAAAAAAGZGVwbG95AAAAAAAGAAAAAAAAAARzYWx0AAAD7gAAACAAAAAAAAAADG1hcmtldF9hZG1pbgAAABMAAAAAAAAABG5hbWUAAAAQAAAAAAAAAAZvcmFjbGUAAAAAABMAAAAAAAAADmluc3VyYW5jZV9mdW5kAAAAAAATAAAAAAAAAAZwYXJhbXMAAAAAB9AAAAAQTWFya2V0SW5pdFBhcmFtcwAAAAEAAAPpAAAAEwAAB9AAAAAITU1DRXJyb3I=",
        "AAAAAAAAAAAAAAALZ2V0X21hcmtldHMAAAAAAAAAAAEAAAPsAAAAEwAAA+0AAAAA",
        "AAAAAAAAAAAAAAAKZ2V0X2NvbmZpZwAAAAAAAAAAAAEAAAfQAAAABkNvbmZpZwAA",
        "AAAAAAAAAAAAAAAUZ2V0X21hcmtldF93YXNtX2hhc2gAAAAAAAAAAQAAA+4AAAAg",
        "AAAAAAAAAAAAAAAcZ2V0X3F1ZXVlZF9pbl9tYXJrZXRfdXBncmFkZQAAAAAAAAABAAAD6AAAB9AAAAAPUXVldWVkSW5VcGdyYWRlAA==",
        "AAAAAAAAAAAAAAAdZ2V0X3F1ZXVlZF9pbl9tYW5hZ2VyX3VwZ3JhZGUAAAAAAAAAAAAAAQAAA+gAAAfQAAAAD1F1ZXVlZEluVXBncmFkZQA=",
        "AAAAAAAAAAAAAAAXcXVldWVfaW5fbWFya2V0X3VwZ3JhZGUAAAAAAQAAAAAAAAANbmV3X3dhc21faGFzaAAAAAAAA+4AAAAgAAAAAQAAA+kAAAPtAAAAAAAAB9AAAAAITU1DRXJyb3I=",
        "AAAAAAAAAAAAAAAYcXVldWVfaW5fbWFuYWdlcl91cGdyYWRlAAAAAQAAAAAAAAANbmV3X3dhc21faGFzaAAAAAAAA+4AAAAgAAAAAQAAA+kAAAPtAAAAAAAAB9AAAAAITU1DRXJyb3I=",
        "AAAAAAAAAAAAAAAVY2FuY2VsX21hcmtldF91cGdyYWRlAAAAAAAAAAAAAAEAAAPpAAAD7QAAAAAAAAfQAAAACE1NQ0Vycm9y",
        "AAAAAAAAAAAAAAAWY2FuY2VsX21hbmFnZXJfdXBncmFkZQAAAAAAAAAAAAEAAAPpAAAD7QAAAAAAAAfQAAAACE1NQ0Vycm9y",
        "AAAAAAAAAAAAAAAUYXBwbHlfbWFya2V0X3VwZ3JhZGUAAAAAAAAAAQAAA+kAAAPtAAAAAAAAB9AAAAAITU1DRXJyb3I=",
        "AAAAAAAAAAAAAAAVYXBwbHlfbWFuYWdlcl91cGdyYWRlAAAAAAAAAAAAAAEAAAPpAAAD7QAAAAAAAAfQAAAACE1NQ0Vycm9y",
        "AAAAAAAAAAAAAAANX19jb25zdHJ1Y3RvcgAAAAAAAAIAAAAAAAAABWFkbWluAAAAAAAAEwAAAAAAAAAZbWFya2V0X2NvbnRyYWN0X3dhc21faGFzaAAAAAAAA+4AAAAgAAAAAA==",
        "AAAABAAAAB1NYXJrZXQgTWFuYWdlciBDb250cmFjdCBFcnJvcgAAAAAAAAAAAAAITU1DRXJyb3IAAAAHAAAAAAAAABJJbnZhbGlkSW5wdXRBbW91bnQAAAAAAAEAAAAAAAAAD092ZXJPclVuZGVyZmxvdwAAAAAJAAAAAAAAABNNYXJrZXRBbHJlYWR5RXhpc3RzAAAAA+gAAAAAAAAAEkludmFsaWRNYXJrZXRTdGF0ZQAAAAAD6QAAAAAAAAAUVXBncmFkZUFscmVhZHlFeGlzdHMAAAPqAAAAAAAAABNVcGdyYWRlRG9lc05vdEV4aXN0AAAAA+sAAAAAAAAAGVVwZ3JhZGVJc05vdFlldEFwcGxpY2FibGUAAAAAAAPs",
        "AAAAAgAAAAAAAAAAAAAAB0RhdGFLZXkAAAAABQAAAAAAAAAAAAAABUFkbWluAAAAAAAAAAAAAAAAAAALTWFya2V0c0xpc3QAAAAAAAAAAAAAAAAOTWFya2V0V2FzbUhhc2gAAAAAAAAAAAAAAAAAFVF1ZXVlZEluTWFya2V0VXBncmFkZQAAAAAAAAAAAAAAAAAAFlF1ZXVlZEluTWFuYWdlclVwZ3JhZGUAAA==",
        "AAAAAQAAAAAAAAAAAAAABkNvbmZpZwAAAAAAAgAAAAAAAAAFYWRtaW4AAAAAAAATAAAAAAAAABBtYXJrZXRfd2FzbV9oYXNoAAAD7gAAACA=",
        "AAAAAQAAAAAAAAAAAAAAD1F1ZXVlZEluVXBncmFkZQAAAAACAAAAAAAAABNxdWV1ZWRfaW5fdGltZXN0YW1wAAAAAAYAAAAAAAAACXdhc21faGFzaAAAAAAAA+4AAAAg" ]),
      options
    )
  }
  public readonly fromJSON = {
    deploy: this.txFromJSON<Result<string>>,
        get_markets: this.txFromJSON<Map<string, void>>,
        get_config: this.txFromJSON<Config>,
        get_market_wasm_hash: this.txFromJSON<Buffer>,
        get_queued_in_market_upgrade: this.txFromJSON<Option<QueuedInUpgrade>>,
        get_queued_in_manager_upgrade: this.txFromJSON<Option<QueuedInUpgrade>>,
        queue_in_market_upgrade: this.txFromJSON<Result<void>>,
        queue_in_manager_upgrade: this.txFromJSON<Result<void>>,
        cancel_market_upgrade: this.txFromJSON<Result<void>>,
        cancel_manager_upgrade: this.txFromJSON<Result<void>>,
        apply_market_upgrade: this.txFromJSON<Result<void>>,
        apply_manager_upgrade: this.txFromJSON<Result<void>>
  }
}