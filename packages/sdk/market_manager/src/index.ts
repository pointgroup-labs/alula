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
  9: {message:"OverOrUnderflow"},
  1000: {message:"MarketAlreadyExists"},
  1001: {message:"UpgradeAlreadyExists"},
  1002: {message:"UpgradeDoesNotExist"},
  1003: {message:"UpgradeIsNotYetApplicable"},
  1004: {message:"NoPendingAdmin"},
  1005: {message:"MarketNotDeployedByManager"},
  1006: {message:"BadUpgradeInQueuePeriod"}
}

export type DataKey = {tag: "Admin", values: void} | {tag: "PendingAdmin", values: void} | {tag: "QueuedInManagerUpgrade", values: void} | {tag: "DeployedMarket", values: readonly [string]} | {tag: "QueuedInMarketUpgrade", values: readonly [string]};


export interface QueuedInUpgrade {
  queued_in_timestamp: u64;
  wasm_hash: Buffer;
}


export interface MarketInitParams {
  bad_debt_lock_d: u64;
  insolvency_ltv_bps: i128;
  is_owned: boolean;
  max_positions: u32;
  min_collateral_value_cents: i128;
  update_in_queue_period: u64;
}

export interface Client {
  /**
   * Construct and simulate a deploy transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  deploy: ({salt, market_wasm_hash, market_admin, name, oracle, insurance_fund, params, upgrade_in_queue_period}: {salt: Buffer, market_wasm_hash: Buffer, market_admin: string, name: string, oracle: string, insurance_fund: string, params: MarketInitParams, upgrade_in_queue_period: u64}, options?: MethodOptions) => Promise<AssembledTransaction<Result<string>>>

  /**
   * Construct and simulate a get_admin transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_admin: (options?: MethodOptions) => Promise<AssembledTransaction<string>>

  /**
   * Construct and simulate a accept_admin transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  accept_admin: (options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a propose_admin transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  propose_admin: ({new_admin}: {new_admin: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a apply_market_upgrade transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  apply_market_upgrade: ({market_address}: {market_address: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a apply_manager_upgrade transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  apply_manager_upgrade: (options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a cancel_market_upgrade transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  cancel_market_upgrade: ({market_address}: {market_address: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a cancel_manager_upgrade transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  cancel_manager_upgrade: (options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a is_deployed_by_manager transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  is_deployed_by_manager: ({market_address}: {market_address: string}, options?: MethodOptions) => Promise<AssembledTransaction<boolean>>

  /**
   * Construct and simulate a queue_in_market_upgrade transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  queue_in_market_upgrade: ({market_address, new_wasm_hash}: {market_address: string, new_wasm_hash: Buffer}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a queue_in_manager_upgrade transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  queue_in_manager_upgrade: ({new_wasm_hash}: {new_wasm_hash: Buffer}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a get_queued_in_market_upgrade transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_queued_in_market_upgrade: ({market_address}: {market_address: string}, options?: MethodOptions) => Promise<AssembledTransaction<Option<QueuedInUpgrade>>>

  /**
   * Construct and simulate a get_queued_in_manager_upgrade transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_queued_in_manager_upgrade: (options?: MethodOptions) => Promise<AssembledTransaction<Option<QueuedInUpgrade>>>

}
export class Client extends ContractClient {
  static async deploy<T = Client>(
        /** Constructor/Initialization Args for the contract's `__constructor` method */
        {admin}: {admin: string},
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
    return ContractClient.deploy({admin}, options)
  }
  constructor(public readonly options: ContractClientOptions) {
    super(
      new ContractSpec([ "AAAABAAAAB1NYXJrZXQgTWFuYWdlciBDb250cmFjdCBFcnJvcgAAAAAAAAAAAAAITU1DRXJyb3IAAAAJAAAAAAAAABJJbnZhbGlkSW5wdXRBbW91bnQAAAAAAAEAAAAAAAAAD092ZXJPclVuZGVyZmxvdwAAAAAJAAAAAAAAABNNYXJrZXRBbHJlYWR5RXhpc3RzAAAAA+gAAAAAAAAAFFVwZ3JhZGVBbHJlYWR5RXhpc3RzAAAD6QAAAAAAAAATVXBncmFkZURvZXNOb3RFeGlzdAAAAAPqAAAAAAAAABlVcGdyYWRlSXNOb3RZZXRBcHBsaWNhYmxlAAAAAAAD6wAAAAAAAAAOTm9QZW5kaW5nQWRtaW4AAAAAA+wAAAAAAAAAGk1hcmtldE5vdERlcGxveWVkQnlNYW5hZ2VyAAAAAAPtAAAAAAAAABdCYWRVcGdyYWRlSW5RdWV1ZVBlcmlvZAAAAAPu",
        "AAAAAgAAAAAAAAAAAAAAB0RhdGFLZXkAAAAABQAAAAAAAAAAAAAABUFkbWluAAAAAAAAAAAAAAAAAAAMUGVuZGluZ0FkbWluAAAAAAAAAAAAAAAWUXVldWVkSW5NYW5hZ2VyVXBncmFkZQAAAAAAAQAAAAAAAAAORGVwbG95ZWRNYXJrZXQAAAAAAAEAAAATAAAAAQAAAAAAAAAVUXVldWVkSW5NYXJrZXRVcGdyYWRlAAAAAAAAAQAAABM=",
        "AAAAAQAAAAAAAAAAAAAAD1F1ZXVlZEluVXBncmFkZQAAAAACAAAAAAAAABNxdWV1ZWRfaW5fdGltZXN0YW1wAAAAAAYAAAAAAAAACXdhc21faGFzaAAAAAAAA+4AAAAg",
        "AAAAAQAAAAAAAAAAAAAAEE1hcmtldEluaXRQYXJhbXMAAAAGAAAAAAAAAA9iYWRfZGVidF9sb2NrX2QAAAAABgAAAAAAAAASaW5zb2x2ZW5jeV9sdHZfYnBzAAAAAAALAAAAAAAAAAhpc19vd25lZAAAAAEAAAAAAAAADW1heF9wb3NpdGlvbnMAAAAAAAAEAAAAAAAAABptaW5fY29sbGF0ZXJhbF92YWx1ZV9jZW50cwAAAAAACwAAAAAAAAAWdXBkYXRlX2luX3F1ZXVlX3BlcmlvZAAAAAAABg==",
        "AAAAAAAAAAAAAAAGZGVwbG95AAAAAAAIAAAAAAAAAARzYWx0AAAD7gAAACAAAAAAAAAAEG1hcmtldF93YXNtX2hhc2gAAAPuAAAAIAAAAAAAAAAMbWFya2V0X2FkbWluAAAAEwAAAAAAAAAEbmFtZQAAABAAAAAAAAAABm9yYWNsZQAAAAAAEwAAAAAAAAAOaW5zdXJhbmNlX2Z1bmQAAAAAABMAAAAAAAAABnBhcmFtcwAAAAAH0AAAABBNYXJrZXRJbml0UGFyYW1zAAAAAAAAABd1cGdyYWRlX2luX3F1ZXVlX3BlcmlvZAAAAAAGAAAAAQAAA+kAAAATAAAH0AAAAAhNTUNFcnJvcg==",
        "AAAAAAAAAAAAAAAJZ2V0X2FkbWluAAAAAAAAAAAAAAEAAAAT",
        "AAAAAAAAAAAAAAAMYWNjZXB0X2FkbWluAAAAAAAAAAEAAAPpAAAD7QAAAAAAAAfQAAAACE1NQ0Vycm9y",
        "AAAAAAAAAAAAAAANX19jb25zdHJ1Y3RvcgAAAAAAAAEAAAAAAAAABWFkbWluAAAAAAAAEwAAAAA=",
        "AAAAAAAAAAAAAAANcHJvcG9zZV9hZG1pbgAAAAAAAAEAAAAAAAAACW5ld19hZG1pbgAAAAAAABMAAAABAAAD6QAAA+0AAAAAAAAH0AAAAAhNTUNFcnJvcg==",
        "AAAAAAAAAAAAAAAUYXBwbHlfbWFya2V0X3VwZ3JhZGUAAAABAAAAAAAAAA5tYXJrZXRfYWRkcmVzcwAAAAAAEwAAAAEAAAPpAAAD7QAAAAAAAAfQAAAACE1NQ0Vycm9y",
        "AAAAAAAAAAAAAAAVYXBwbHlfbWFuYWdlcl91cGdyYWRlAAAAAAAAAAAAAAEAAAPpAAAD7QAAAAAAAAfQAAAACE1NQ0Vycm9y",
        "AAAAAAAAAAAAAAAVY2FuY2VsX21hcmtldF91cGdyYWRlAAAAAAAAAQAAAAAAAAAObWFya2V0X2FkZHJlc3MAAAAAABMAAAABAAAD6QAAA+0AAAAAAAAH0AAAAAhNTUNFcnJvcg==",
        "AAAAAAAAAAAAAAAWY2FuY2VsX21hbmFnZXJfdXBncmFkZQAAAAAAAAAAAAEAAAPpAAAD7QAAAAAAAAfQAAAACE1NQ0Vycm9y",
        "AAAAAAAAAAAAAAAWaXNfZGVwbG95ZWRfYnlfbWFuYWdlcgAAAAAAAQAAAAAAAAAObWFya2V0X2FkZHJlc3MAAAAAABMAAAABAAAAAQ==",
        "AAAAAAAAAAAAAAAXcXVldWVfaW5fbWFya2V0X3VwZ3JhZGUAAAAAAgAAAAAAAAAObWFya2V0X2FkZHJlc3MAAAAAABMAAAAAAAAADW5ld193YXNtX2hhc2gAAAAAAAPuAAAAIAAAAAEAAAPpAAAD7QAAAAAAAAfQAAAACE1NQ0Vycm9y",
        "AAAAAAAAAAAAAAAYcXVldWVfaW5fbWFuYWdlcl91cGdyYWRlAAAAAQAAAAAAAAANbmV3X3dhc21faGFzaAAAAAAAA+4AAAAgAAAAAQAAA+kAAAPtAAAAAAAAB9AAAAAITU1DRXJyb3I=",
        "AAAAAAAAAAAAAAAcZ2V0X3F1ZXVlZF9pbl9tYXJrZXRfdXBncmFkZQAAAAEAAAAAAAAADm1hcmtldF9hZGRyZXNzAAAAAAATAAAAAQAAA+gAAAfQAAAAD1F1ZXVlZEluVXBncmFkZQA=",
        "AAAAAAAAAAAAAAAdZ2V0X3F1ZXVlZF9pbl9tYW5hZ2VyX3VwZ3JhZGUAAAAAAAAAAAAAAQAAA+gAAAfQAAAAD1F1ZXVlZEluVXBncmFkZQA=" ]),
      options
    )
  }
  public readonly fromJSON = {
    deploy: this.txFromJSON<Result<string>>,
        get_admin: this.txFromJSON<string>,
        accept_admin: this.txFromJSON<Result<void>>,
        propose_admin: this.txFromJSON<Result<void>>,
        apply_market_upgrade: this.txFromJSON<Result<void>>,
        apply_manager_upgrade: this.txFromJSON<Result<void>>,
        cancel_market_upgrade: this.txFromJSON<Result<void>>,
        cancel_manager_upgrade: this.txFromJSON<Result<void>>,
        is_deployed_by_manager: this.txFromJSON<boolean>,
        queue_in_market_upgrade: this.txFromJSON<Result<void>>,
        queue_in_manager_upgrade: this.txFromJSON<Result<void>>,
        get_queued_in_market_upgrade: this.txFromJSON<Option<QueuedInUpgrade>>,
        get_queued_in_manager_upgrade: this.txFromJSON<Option<QueuedInUpgrade>>
  }
}