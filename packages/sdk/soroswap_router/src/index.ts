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





export interface InitializedEvent {
  factory: string;
}


export interface AddLiquidityEvent {
  amount_a: i128;
  amount_b: i128;
  liquidity: i128;
  pair: string;
  to: string;
  token_a: string;
  token_b: string;
}


export interface RemoveLiquidityEvent {
  amount_a: i128;
  amount_b: i128;
  liquidity: i128;
  pair: string;
  to: string;
  token_a: string;
  token_b: string;
}


export interface SwapEvent {
  amounts: Array<i128>;
  path: Array<string>;
  to: string;
}

export const SoroswapRouterError = {
  /**
   * SoroswapRouter: not yet initialized
   */
  401: {message:"NotInitialized"},
  /**
   * SoroswapRouter: negative amount is not allowed
   */
  402: {message:"NegativeNotAllowed"},
  /**
   * SoroswapRouter: deadline expired
   */
  403: {message:"DeadlineExpired"},
  /**
   * SoroswapRouter: already initialized
   */
  404: {message:"InitializeAlreadyInitialized"},
  /**
   * SoroswapRouter: insufficient a amount
   */
  405: {message:"InsufficientAAmount"},
  /**
   * SoroswapRouter: insufficient b amount
   */
  406: {message:"InsufficientBAmount"},
  /**
   * SoroswapRouter: insufficient output amount
   */
  407: {message:"InsufficientOutputAmount"},
  /**
   * SoroswapRouter: excessive input amount
   */
  408: {message:"ExcessiveInputAmount"},
  /**
   * SoroswapRouter: pair does not exist
   */
  409: {message:"PairDoesNotExist"}
}

export const CombinedRouterError = {
  501: {message:"RouterNotInitialized"},
  502: {message:"RouterNegativeNotAllowed"},
  503: {message:"RouterDeadlineExpired"},
  504: {message:"RouterInitializeAlreadyInitialized"},
  505: {message:"RouterInsufficientAAmount"},
  506: {message:"RouterInsufficientBAmount"},
  507: {message:"RouterInsufficientOutputAmount"},
  508: {message:"RouterExcessiveInputAmount"},
  509: {message:"RouterPairDoesNotExist"},
  510: {message:"LibraryInsufficientAmount"},
  511: {message:"LibraryInsufficientLiquidity"},
  512: {message:"LibraryInsufficientInputAmount"},
  513: {message:"LibraryInsufficientOutputAmount"},
  514: {message:"LibraryInvalidPath"},
  515: {message:"LibrarySortIdenticalTokens"}
}

export const SoroswapLibraryError = {
  /**
   * SoroswapLibrary: insufficient amount
   */
  301: {message:"InsufficientAmount"},
  /**
   * SoroswapLibrary: insufficient liquidity
   */
  302: {message:"InsufficientLiquidity"},
  /**
   * SoroswapLibrary: insufficient input amount
   */
  303: {message:"InsufficientInputAmount"},
  /**
   * SoroswapLibrary: insufficient output amount
   */
  304: {message:"InsufficientOutputAmount"},
  /**
   * SoroswapLibrary: invalid path
   */
  305: {message:"InvalidPath"},
  /**
   * SoroswapLibrary: token_a and token_b have identical addresses
   */
  306: {message:"SortIdenticalTokens"}
}

export interface Client {
  /**
   * Construct and simulate a initialize transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Initializes the contract and sets the factory address
   */
  initialize: ({factory}: {factory: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a add_liquidity transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Adds liquidity to a token pair's pool, creating it if it doesn't exist. Ensures that exactly the desired amounts
   * of both tokens are added, subject to minimum requirements.
   * This function is responsible for transferring tokens from the user to the pool and minting liquidity tokens in return.
   * # Arguments
   * * `token_a` - The address of the first token to add liquidity for.
   * * `token_b` - The address of the second token to add liquidity for.
   * * `amount_a_desired` - The desired amount of the first token to add.
   * * `amount_b_desired` - The desired amount of the second token to add.
   * * `amount_a_min` - The minimum required amount of the first token to add.
   * * `amount_b_min` - The minimum required amount of the second token to add.
   * * `to` - The address where the liquidity tokens will be minted and sent.
   * * `deadline` - The deadline for executing the operation.
   * # Returns
   * A tuple containing: amounts of token A and B added to the pool.
   * plus the amount of liquidity tokens minted.
   */
  add_liquidity: ({token_a, token_b, amount_a_desired, amount_b_desired, amount_a_min, amount_b_min, to, deadline}: {token_a: string, token_b: string, amount_a_desired: i128, amount_b_desired: i128, amount_a_min: i128, amount_b_min: i128, to: string, deadline: u64}, options?: MethodOptions) => Promise<AssembledTransaction<Result<readonly [i128, i128, i128]>>>

  /**
   * Construct and simulate a remove_liquidity transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Removes liquidity from a token pair's pool.
   * 
   * This function facilitates the removal of liquidity from a Soroswap Liquidity Pool by burning a specified amount
   * of Liquidity Pool tokens (`liquidity`) owned by the caller. In return, it transfers back the corresponding
   * amounts of the paired tokens (`token_a` and `token_b`) to the caller's specified address (`to`).
   * 
   * # Arguments
   * * `token_a` - The address of the first token in the Liquidity Pool.
   * * `token_b` - The address of the second token in the Liquidity Pool.
   * * `liquidity` - The desired amount of Liquidity Pool tokens to be burned.
   * * `amount_a_min` - The minimum required amount of the first token to receive.
   * * `amount_b_min` - The minimum required amount of the second token to receive.
   * * `to` - The address where the paired tokens will be sent to, and from where the LP tokens will be taken.
   * * `deadline` - The deadline for executing the operation.
   * 
   * # Returns
   * A tuple containing the amounts of `token_a` and `token_b` withdrawn from the pool.
   */
  remove_liquidity: ({token_a, token_b, liquidity, amount_a_min, amount_b_min, to, deadline}: {token_a: string, token_b: string, liquidity: i128, amount_a_min: i128, amount_b_min: i128, to: string, deadline: u64}, options?: MethodOptions) => Promise<AssembledTransaction<Result<readonly [i128, i128]>>>

  /**
   * Construct and simulate a swap_exact_tokens_for_tokens transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Swaps an exact amount of input tokens for as many output tokens as possible
   * along the specified trading route. The route is determined by the `path` vector,
   * where the first element is the input token, the last is the output token,
   * and any intermediate elements represent pairs to trade through if a direct pair does not exist.
   * 
   * # Arguments
   * * `amount_in` - The exact amount of input tokens to be swapped.
   * * `amount_out_min` - The minimum required amount of output tokens to receive.
   * * `path` - A vector representing the trading route, where the first element is the input token
   * and the last is the output token. Intermediate elements represent pairs to trade through.
   * * `to` - The address where the output tokens will be sent to.
   * * `deadline` - The deadline for executing the operation.
   * 
   * # Returns
   * A vector containing the amounts of tokens received at each step of the trading route.
   */
  swap_exact_tokens_for_tokens: ({amount_in, amount_out_min, path, to, deadline}: {amount_in: i128, amount_out_min: i128, path: Array<string>, to: string, deadline: u64}, options?: MethodOptions) => Promise<AssembledTransaction<Result<Array<i128>>>>

  /**
   * Construct and simulate a swap_tokens_for_exact_tokens transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Swaps tokens for an exact amount of output token, following the specified trading route.
   * The route is determined by the `path` vector, where the first element is the input token,
   * the last is the output token, and any intermediate elements represent pairs to trade through.
   * 
   * # Arguments
   * * `amount_out` - The exact amount of output token to be received.
   * * `amount_in_max` - The maximum allowed amount of input tokens to be swapped.
   * * `path` - A vector representing the trading route, where the first element is the input token
   * and the last is the output token. Intermediate elements represent pairs to trade through.
   * * `to` - The address where the output tokens will be sent to.
   * * `deadline` - The deadline for executing the operation.
   * 
   * # Returns
   * A vector containing the amounts of tokens used at each step of the trading route.
   */
  swap_tokens_for_exact_tokens: ({amount_out, amount_in_max, path, to, deadline}: {amount_out: i128, amount_in_max: i128, path: Array<string>, to: string, deadline: u64}, options?: MethodOptions) => Promise<AssembledTransaction<Result<Array<i128>>>>

  /**
   * Construct and simulate a get_factory transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * This function retrieves the factory contract's address associated with the provided environment.
   * It also checks if the factory has been initialized and raises an assertion error if not.
   * If the factory is not initialized, this code will raise an assertion error with the message "SoroswapRouter: not yet initialized".
   * https://github.com/benjaminsalon/malicious_sorochat
   * # Arguments
   * * `e` - The contract environment (`Env`) in which the contract is executing.
   */
  get_factory: (options?: MethodOptions) => Promise<AssembledTransaction<Result<string>>>

  /**
   * Construct and simulate a router_pair_for transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Calculates the deterministic address for a pair without making any external calls.
   * check <https://github.com/paltalabs/deterministic-address-soroban>
   * 
   * # Arguments
   * 
   * * `e` - The environment.
   * * `token_a` - The address of the first token.
   * * `token_b` - The address of the second token.
   * 
   * # Returns
   * 
   * Returns `Result<Address, SoroswapLibraryError>` where `Ok` contains the deterministic address for the pair, and `Err` indicates an error such as identical tokens or an issue with sorting.
   */
  router_pair_for: ({token_a, token_b}: {token_a: string, token_b: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<string>>>

  /**
   * Construct and simulate a router_quote transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Given some amount of an asset and pair reserves, returns an equivalent amount of the other asset.
   * 
   * # Arguments
   * 
   * * `amount_a` - The amount of the first asset.
   * * `reserve_a` - Reserves of the first asset in the pair.
   * * `reserve_b` - Reserves of the second asset in the pair.
   * 
   * # Returns
   * 
   * Returns `Result<i128, SoroswapLibraryError>` where `Ok` contains the calculated equivalent amount, and `Err` indicates an error such as insufficient amount or liquidity
   */
  router_quote: ({amount_a, reserve_a, reserve_b}: {amount_a: i128, reserve_a: i128, reserve_b: i128}, options?: MethodOptions) => Promise<AssembledTransaction<Result<i128>>>

  /**
   * Construct and simulate a router_get_amount_out transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Given an input amount of an asset and pair reserves, returns the maximum output amount of the other asset.
   * 
   * # Arguments
   * 
   * * `amount_in` - The input amount of the asset.
   * * `reserve_in` - Reserves of the input asset in the pair.
   * * `reserve_out` - Reserves of the output asset in the pair.
   * 
   * # Returns
   * 
   * Returns `Result<i128, SoroswapLibraryError>` where `Ok` contains the calculated maximum output amount, and `Err` indicates an error such as insufficient input amount or liquidity.
   */
  router_get_amount_out: ({amount_in, reserve_in, reserve_out}: {amount_in: i128, reserve_in: i128, reserve_out: i128}, options?: MethodOptions) => Promise<AssembledTransaction<Result<i128>>>

  /**
   * Construct and simulate a router_get_amount_in transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Given an output amount of an asset and pair reserves, returns a required input amount of the other asset.
   * 
   * # Arguments
   * 
   * * `amount_out` - The output amount of the asset.
   * * `reserve_in` - Reserves of the input asset in the pair.
   * * `reserve_out` - Reserves of the output asset in the pair.
   * 
   * # Returns
   * 
   * Returns `Result<i128, SoroswapLibraryError>` where `Ok` contains the required input amount, and `Err` indicates an error such as insufficient output amount or liquidity.
   */
  router_get_amount_in: ({amount_out, reserve_in, reserve_out}: {amount_out: i128, reserve_in: i128, reserve_out: i128}, options?: MethodOptions) => Promise<AssembledTransaction<Result<i128>>>

  /**
   * Construct and simulate a router_get_amounts_out transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Performs chained get_amount_out calculations on any number of pairs.
   * 
   * # Arguments
   * 
   * * `e` - The environment.
   * * `amount_in` - The input amount.
   * * `path` - Vector of token addresses representing the path.
   * 
   * # Returns
   * 
   * Returns `Result<Vec<i128>, SoroswapLibraryError>` where `Ok` contains a vector of calculated amounts, and `Err` indicates an error such as an invalid path.
   */
  router_get_amounts_out: ({amount_in, path}: {amount_in: i128, path: Array<string>}, options?: MethodOptions) => Promise<AssembledTransaction<Result<Array<i128>>>>

  /**
   * Construct and simulate a router_get_amounts_in transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Performs chained get_amount_in calculations on any number of pairs.
   * 
   * # Arguments
   * 
   * * `e` - The environment.
   * * `amount_out` - The output amount.
   * * `path` - Vector of token addresses representing the path.
   * 
   * # Returns
   * 
   * Returns `Result<Vec<i128>, SoroswapLibraryError>` where `Ok` contains a vector of calculated amounts, and `Err` indicates an error such as an invalid path.
   */
  router_get_amounts_in: ({amount_out, path}: {amount_out: i128, path: Array<string>}, options?: MethodOptions) => Promise<AssembledTransaction<Result<Array<i128>>>>

  /**
   * Construct and simulate a sort_tokens transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Sorts two token addresses in a consistent order.
   * 
   * # Arguments
   * 
   * * `token_a` - The address of the first token.
   * * `token_b` - The address of the second token.
   * 
   * # Returns
   * 
   * Returns `Result<(Address, Address), SoroswapLibraryError>` where `Ok` contains a tuple with the sorted token addresses, and `Err` indicates an error such as identical tokens.
   */
  sort_tokens: ({token_a, token_b}: {token_a: string, token_b: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<readonly [string, string]>>>

  /**
   * Construct and simulate a pair_for transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Calculates the deterministic address for a pair without making any external calls.
   * check <https://github.com/paltalabs/deterministic-address-soroban>
   * 
   * # Arguments
   * 
   * * `e` - The environment.
   * * `factory` - The factory address.
   * * `token_a` - The address of the first token.
   * * `token_b` - The address of the second token.
   * 
   * # Returns
   * 
   * Returns `Result<Address, SoroswapLibraryError>` where `Ok` contains the deterministic address for the pair, and `Err` indicates an error such as identical tokens or an issue with sorting.
   */
  pair_for: ({factory, token_a, token_b}: {factory: string, token_a: string, token_b: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<string>>>

  /**
   * Construct and simulate a get_reserves transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Fetches and sorts the reserves for a pair of tokens.
   * 
   * # Arguments
   * 
   * * `e` - The environment.
   * * `factory` - The factory address.
   * * `token_a` - The address of the first token.
   * * `token_b` - The address of the second token.
   * 
   * # Returns
   * 
   * Returns `Result<(i128, i128), SoroswapLibraryError>` where `Ok` contains a tuple of sorted reserves, and `Err` indicates an error such as identical tokens or an issue with sorting.
   */
  get_reserves: ({factory, token_a, token_b}: {factory: string, token_a: string, token_b: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<readonly [i128, i128]>>>

  /**
   * Construct and simulate a quote transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Given some amount of an asset and pair reserves, returns an equivalent amount of the other asset.
   * 
   * # Arguments
   * 
   * * `amount_a` - The amount of the first asset.
   * * `reserve_a` - Reserves of the first asset in the pair.
   * * `reserve_b` - Reserves of the second asset in the pair.
   * 
   * # Returns
   * 
   * Returns `Result<i128, SoroswapLibraryError>` where `Ok` contains the calculated equivalent amount, and `Err` indicates an error such as insufficient amount or liquidity
   */
  quote: ({amount_a, reserve_a, reserve_b}: {amount_a: i128, reserve_a: i128, reserve_b: i128}, options?: MethodOptions) => Promise<AssembledTransaction<Result<i128>>>

  /**
   * Construct and simulate a get_amount_out transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Given an input amount of an asset and pair reserves, returns the maximum output amount of the other asset.
   * 
   * # Arguments
   * 
   * * `amount_in` - The input amount of the asset.
   * * `reserve_in` - Reserves of the input asset in the pair.
   * * `reserve_out` - Reserves of the output asset in the pair.
   * 
   * # Returns
   * 
   * Returns `Result<i128, SoroswapLibraryError>` where `Ok` contains the calculated maximum output amount, and `Err` indicates an error such as insufficient input amount or liquidity.
   */
  get_amount_out: ({amount_in, reserve_in, reserve_out}: {amount_in: i128, reserve_in: i128, reserve_out: i128}, options?: MethodOptions) => Promise<AssembledTransaction<Result<i128>>>

  /**
   * Construct and simulate a get_amount_in transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Given an output amount of an asset and pair reserves, returns a required input amount of the other asset.
   * 
   * # Arguments
   * 
   * * `amount_out` - The output amount of the asset.
   * * `reserve_in` - Reserves of the input asset in the pair.
   * * `reserve_out` - Reserves of the output asset in the pair.
   * 
   * # Returns
   * 
   * Returns `Result<i128, SoroswapLibraryError>` where `Ok` contains the required input amount, and `Err` indicates an error such as insufficient output amount or liquidity.
   */
  get_amount_in: ({amount_out, reserve_in, reserve_out}: {amount_out: i128, reserve_in: i128, reserve_out: i128}, options?: MethodOptions) => Promise<AssembledTransaction<Result<i128>>>

  /**
   * Construct and simulate a get_amounts_out transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Performs chained get_amount_out calculations on any number of pairs.
   * 
   * # Arguments
   * 
   * * `e` - The environment.
   * * `factory` - The factory address.
   * * `amount_in` - The input amount.
   * * `path` - Vector of token addresses representing the path.
   * 
   * # Returns
   * 
   * Returns `Result<Vec<i128>, SoroswapLibraryError>` where `Ok` contains a vector of calculated amounts, and `Err` indicates an error such as an invalid path.
   */
  get_amounts_out: ({factory, amount_in, path}: {factory: string, amount_in: i128, path: Array<string>}, options?: MethodOptions) => Promise<AssembledTransaction<Result<Array<i128>>>>

  /**
   * Construct and simulate a get_amounts_in transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Performs chained get_amount_in calculations on any number of pairs.
   * 
   * # Arguments
   * 
   * * `e` - The environment.
   * * `factory` - The factory address.
   * * `amount_out` - The output amount.
   * * `path` - Vector of token addresses representing the path.
   * 
   * # Returns
   * 
   * Returns `Result<Vec<i128>, SoroswapLibraryError>` where `Ok` contains a vector of calculated amounts, and `Err` indicates an error such as an invalid path.
   */
  get_amounts_in: ({factory, amount_out, path}: {factory: string, amount_out: i128, path: Array<string>}, options?: MethodOptions) => Promise<AssembledTransaction<Result<Array<i128>>>>

}
export class Client extends ContractClient {
  static async deploy<T = Client>(
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
    return ContractClient.deploy(null, options)
  }
  constructor(public readonly options: ContractClientOptions) {
    super(
      new ContractSpec([ "AAAAAQAAAAAAAAAAAAAAEEluaXRpYWxpemVkRXZlbnQAAAABAAAAAAAAAAdmYWN0b3J5AAAAABM=",
        "AAAAAQAAAAAAAAAAAAAAEUFkZExpcXVpZGl0eUV2ZW50AAAAAAAABwAAAAAAAAAIYW1vdW50X2EAAAALAAAAAAAAAAhhbW91bnRfYgAAAAsAAAAAAAAACWxpcXVpZGl0eQAAAAAAAAsAAAAAAAAABHBhaXIAAAATAAAAAAAAAAJ0bwAAAAAAEwAAAAAAAAAHdG9rZW5fYQAAAAATAAAAAAAAAAd0b2tlbl9iAAAAABM=",
        "AAAAAQAAAAAAAAAAAAAAFFJlbW92ZUxpcXVpZGl0eUV2ZW50AAAABwAAAAAAAAAIYW1vdW50X2EAAAALAAAAAAAAAAhhbW91bnRfYgAAAAsAAAAAAAAACWxpcXVpZGl0eQAAAAAAAAsAAAAAAAAABHBhaXIAAAATAAAAAAAAAAJ0bwAAAAAAEwAAAAAAAAAHdG9rZW5fYQAAAAATAAAAAAAAAAd0b2tlbl9iAAAAABM=",
        "AAAAAQAAAAAAAAAAAAAACVN3YXBFdmVudAAAAAAAAAMAAAAAAAAAB2Ftb3VudHMAAAAD6gAAAAsAAAAAAAAABHBhdGgAAAPqAAAAEwAAAAAAAAACdG8AAAAAABM=",
        "AAAABAAAAAAAAAAAAAAAE1Nvcm9zd2FwUm91dGVyRXJyb3IAAAAACQAAACNTb3Jvc3dhcFJvdXRlcjogbm90IHlldCBpbml0aWFsaXplZAAAAAAOTm90SW5pdGlhbGl6ZWQAAAAAAZEAAAAuU29yb3N3YXBSb3V0ZXI6IG5lZ2F0aXZlIGFtb3VudCBpcyBub3QgYWxsb3dlZAAAAAAAEk5lZ2F0aXZlTm90QWxsb3dlZAAAAAABkgAAACBTb3Jvc3dhcFJvdXRlcjogZGVhZGxpbmUgZXhwaXJlZAAAAA9EZWFkbGluZUV4cGlyZWQAAAABkwAAACNTb3Jvc3dhcFJvdXRlcjogYWxyZWFkeSBpbml0aWFsaXplZAAAAAAcSW5pdGlhbGl6ZUFscmVhZHlJbml0aWFsaXplZAAAAZQAAAAlU29yb3N3YXBSb3V0ZXI6IGluc3VmZmljaWVudCBhIGFtb3VudAAAAAAAABNJbnN1ZmZpY2llbnRBQW1vdW50AAAAAZUAAAAlU29yb3N3YXBSb3V0ZXI6IGluc3VmZmljaWVudCBiIGFtb3VudAAAAAAAABNJbnN1ZmZpY2llbnRCQW1vdW50AAAAAZYAAAAqU29yb3N3YXBSb3V0ZXI6IGluc3VmZmljaWVudCBvdXRwdXQgYW1vdW50AAAAAAAYSW5zdWZmaWNpZW50T3V0cHV0QW1vdW50AAABlwAAACZTb3Jvc3dhcFJvdXRlcjogZXhjZXNzaXZlIGlucHV0IGFtb3VudAAAAAAAFEV4Y2Vzc2l2ZUlucHV0QW1vdW50AAABmAAAACNTb3Jvc3dhcFJvdXRlcjogcGFpciBkb2VzIG5vdCBleGlzdAAAAAAQUGFpckRvZXNOb3RFeGlzdAAAAZk=",
        "AAAABAAAAAAAAAAAAAAAE0NvbWJpbmVkUm91dGVyRXJyb3IAAAAADwAAAAAAAAAUUm91dGVyTm90SW5pdGlhbGl6ZWQAAAH1AAAAAAAAABhSb3V0ZXJOZWdhdGl2ZU5vdEFsbG93ZWQAAAH2AAAAAAAAABVSb3V0ZXJEZWFkbGluZUV4cGlyZWQAAAAAAAH3AAAAAAAAACJSb3V0ZXJJbml0aWFsaXplQWxyZWFkeUluaXRpYWxpemVkAAAAAAH4AAAAAAAAABlSb3V0ZXJJbnN1ZmZpY2llbnRBQW1vdW50AAAAAAAB+QAAAAAAAAAZUm91dGVySW5zdWZmaWNpZW50QkFtb3VudAAAAAAAAfoAAAAAAAAAHlJvdXRlckluc3VmZmljaWVudE91dHB1dEFtb3VudAAAAAAB+wAAAAAAAAAaUm91dGVyRXhjZXNzaXZlSW5wdXRBbW91bnQAAAAAAfwAAAAAAAAAFlJvdXRlclBhaXJEb2VzTm90RXhpc3QAAAAAAf0AAAAAAAAAGUxpYnJhcnlJbnN1ZmZpY2llbnRBbW91bnQAAAAAAAH+AAAAAAAAABxMaWJyYXJ5SW5zdWZmaWNpZW50TGlxdWlkaXR5AAAB/wAAAAAAAAAeTGlicmFyeUluc3VmZmljaWVudElucHV0QW1vdW50AAAAAAIAAAAAAAAAAB9MaWJyYXJ5SW5zdWZmaWNpZW50T3V0cHV0QW1vdW50AAAAAgEAAAAAAAAAEkxpYnJhcnlJbnZhbGlkUGF0aAAAAAACAgAAAAAAAAAaTGlicmFyeVNvcnRJZGVudGljYWxUb2tlbnMAAAAAAgM=",
        "AAAAAAAAADVJbml0aWFsaXplcyB0aGUgY29udHJhY3QgYW5kIHNldHMgdGhlIGZhY3RvcnkgYWRkcmVzcwAAAAAAAAppbml0aWFsaXplAAAAAAABAAAAAAAAAAdmYWN0b3J5AAAAABMAAAABAAAD6QAAA+0AAAAAAAAH0AAAABNDb21iaW5lZFJvdXRlckVycm9yAA==",
        "AAAAAAAAA81BZGRzIGxpcXVpZGl0eSB0byBhIHRva2VuIHBhaXIncyBwb29sLCBjcmVhdGluZyBpdCBpZiBpdCBkb2Vzbid0IGV4aXN0LiBFbnN1cmVzIHRoYXQgZXhhY3RseSB0aGUgZGVzaXJlZCBhbW91bnRzCm9mIGJvdGggdG9rZW5zIGFyZSBhZGRlZCwgc3ViamVjdCB0byBtaW5pbXVtIHJlcXVpcmVtZW50cy4KVGhpcyBmdW5jdGlvbiBpcyByZXNwb25zaWJsZSBmb3IgdHJhbnNmZXJyaW5nIHRva2VucyBmcm9tIHRoZSB1c2VyIHRvIHRoZSBwb29sIGFuZCBtaW50aW5nIGxpcXVpZGl0eSB0b2tlbnMgaW4gcmV0dXJuLgojIEFyZ3VtZW50cwoqIGB0b2tlbl9hYCAtIFRoZSBhZGRyZXNzIG9mIHRoZSBmaXJzdCB0b2tlbiB0byBhZGQgbGlxdWlkaXR5IGZvci4KKiBgdG9rZW5fYmAgLSBUaGUgYWRkcmVzcyBvZiB0aGUgc2Vjb25kIHRva2VuIHRvIGFkZCBsaXF1aWRpdHkgZm9yLgoqIGBhbW91bnRfYV9kZXNpcmVkYCAtIFRoZSBkZXNpcmVkIGFtb3VudCBvZiB0aGUgZmlyc3QgdG9rZW4gdG8gYWRkLgoqIGBhbW91bnRfYl9kZXNpcmVkYCAtIFRoZSBkZXNpcmVkIGFtb3VudCBvZiB0aGUgc2Vjb25kIHRva2VuIHRvIGFkZC4KKiBgYW1vdW50X2FfbWluYCAtIFRoZSBtaW5pbXVtIHJlcXVpcmVkIGFtb3VudCBvZiB0aGUgZmlyc3QgdG9rZW4gdG8gYWRkLgoqIGBhbW91bnRfYl9taW5gIC0gVGhlIG1pbmltdW0gcmVxdWlyZWQgYW1vdW50IG9mIHRoZSBzZWNvbmQgdG9rZW4gdG8gYWRkLgoqIGB0b2AgLSBUaGUgYWRkcmVzcyB3aGVyZSB0aGUgbGlxdWlkaXR5IHRva2VucyB3aWxsIGJlIG1pbnRlZCBhbmQgc2VudC4KKiBgZGVhZGxpbmVgIC0gVGhlIGRlYWRsaW5lIGZvciBleGVjdXRpbmcgdGhlIG9wZXJhdGlvbi4KIyBSZXR1cm5zCkEgdHVwbGUgY29udGFpbmluZzogYW1vdW50cyBvZiB0b2tlbiBBIGFuZCBCIGFkZGVkIHRvIHRoZSBwb29sLgpwbHVzIHRoZSBhbW91bnQgb2YgbGlxdWlkaXR5IHRva2VucyBtaW50ZWQuAAAAAAAADWFkZF9saXF1aWRpdHkAAAAAAAAIAAAAAAAAAAd0b2tlbl9hAAAAABMAAAAAAAAAB3Rva2VuX2IAAAAAEwAAAAAAAAAQYW1vdW50X2FfZGVzaXJlZAAAAAsAAAAAAAAAEGFtb3VudF9iX2Rlc2lyZWQAAAALAAAAAAAAAAxhbW91bnRfYV9taW4AAAALAAAAAAAAAAxhbW91bnRfYl9taW4AAAALAAAAAAAAAAJ0bwAAAAAAEwAAAAAAAAAIZGVhZGxpbmUAAAAGAAAAAQAAA+kAAAPtAAAAAwAAAAsAAAALAAAACwAAB9AAAAATQ29tYmluZWRSb3V0ZXJFcnJvcgA=",
        "AAAAAAAAA+ZSZW1vdmVzIGxpcXVpZGl0eSBmcm9tIGEgdG9rZW4gcGFpcidzIHBvb2wuCgpUaGlzIGZ1bmN0aW9uIGZhY2lsaXRhdGVzIHRoZSByZW1vdmFsIG9mIGxpcXVpZGl0eSBmcm9tIGEgU29yb3N3YXAgTGlxdWlkaXR5IFBvb2wgYnkgYnVybmluZyBhIHNwZWNpZmllZCBhbW91bnQKb2YgTGlxdWlkaXR5IFBvb2wgdG9rZW5zIChgbGlxdWlkaXR5YCkgb3duZWQgYnkgdGhlIGNhbGxlci4gSW4gcmV0dXJuLCBpdCB0cmFuc2ZlcnMgYmFjayB0aGUgY29ycmVzcG9uZGluZwphbW91bnRzIG9mIHRoZSBwYWlyZWQgdG9rZW5zIChgdG9rZW5fYWAgYW5kIGB0b2tlbl9iYCkgdG8gdGhlIGNhbGxlcidzIHNwZWNpZmllZCBhZGRyZXNzIChgdG9gKS4KCiMgQXJndW1lbnRzCiogYHRva2VuX2FgIC0gVGhlIGFkZHJlc3Mgb2YgdGhlIGZpcnN0IHRva2VuIGluIHRoZSBMaXF1aWRpdHkgUG9vbC4KKiBgdG9rZW5fYmAgLSBUaGUgYWRkcmVzcyBvZiB0aGUgc2Vjb25kIHRva2VuIGluIHRoZSBMaXF1aWRpdHkgUG9vbC4KKiBgbGlxdWlkaXR5YCAtIFRoZSBkZXNpcmVkIGFtb3VudCBvZiBMaXF1aWRpdHkgUG9vbCB0b2tlbnMgdG8gYmUgYnVybmVkLgoqIGBhbW91bnRfYV9taW5gIC0gVGhlIG1pbmltdW0gcmVxdWlyZWQgYW1vdW50IG9mIHRoZSBmaXJzdCB0b2tlbiB0byByZWNlaXZlLgoqIGBhbW91bnRfYl9taW5gIC0gVGhlIG1pbmltdW0gcmVxdWlyZWQgYW1vdW50IG9mIHRoZSBzZWNvbmQgdG9rZW4gdG8gcmVjZWl2ZS4KKiBgdG9gIC0gVGhlIGFkZHJlc3Mgd2hlcmUgdGhlIHBhaXJlZCB0b2tlbnMgd2lsbCBiZSBzZW50IHRvLCBhbmQgZnJvbSB3aGVyZSB0aGUgTFAgdG9rZW5zIHdpbGwgYmUgdGFrZW4uCiogYGRlYWRsaW5lYCAtIFRoZSBkZWFkbGluZSBmb3IgZXhlY3V0aW5nIHRoZSBvcGVyYXRpb24uCgojIFJldHVybnMKQSB0dXBsZSBjb250YWluaW5nIHRoZSBhbW91bnRzIG9mIGB0b2tlbl9hYCBhbmQgYHRva2VuX2JgIHdpdGhkcmF3biBmcm9tIHRoZSBwb29sLgAAAAAAEHJlbW92ZV9saXF1aWRpdHkAAAAHAAAAAAAAAAd0b2tlbl9hAAAAABMAAAAAAAAAB3Rva2VuX2IAAAAAEwAAAAAAAAAJbGlxdWlkaXR5AAAAAAAACwAAAAAAAAAMYW1vdW50X2FfbWluAAAACwAAAAAAAAAMYW1vdW50X2JfbWluAAAACwAAAAAAAAACdG8AAAAAABMAAAAAAAAACGRlYWRsaW5lAAAABgAAAAEAAAPpAAAD7QAAAAIAAAALAAAACwAAB9AAAAATQ29tYmluZWRSb3V0ZXJFcnJvcgA=",
        "AAAAAAAAA3JTd2FwcyBhbiBleGFjdCBhbW91bnQgb2YgaW5wdXQgdG9rZW5zIGZvciBhcyBtYW55IG91dHB1dCB0b2tlbnMgYXMgcG9zc2libGUKYWxvbmcgdGhlIHNwZWNpZmllZCB0cmFkaW5nIHJvdXRlLiBUaGUgcm91dGUgaXMgZGV0ZXJtaW5lZCBieSB0aGUgYHBhdGhgIHZlY3RvciwKd2hlcmUgdGhlIGZpcnN0IGVsZW1lbnQgaXMgdGhlIGlucHV0IHRva2VuLCB0aGUgbGFzdCBpcyB0aGUgb3V0cHV0IHRva2VuLAphbmQgYW55IGludGVybWVkaWF0ZSBlbGVtZW50cyByZXByZXNlbnQgcGFpcnMgdG8gdHJhZGUgdGhyb3VnaCBpZiBhIGRpcmVjdCBwYWlyIGRvZXMgbm90IGV4aXN0LgoKIyBBcmd1bWVudHMKKiBgYW1vdW50X2luYCAtIFRoZSBleGFjdCBhbW91bnQgb2YgaW5wdXQgdG9rZW5zIHRvIGJlIHN3YXBwZWQuCiogYGFtb3VudF9vdXRfbWluYCAtIFRoZSBtaW5pbXVtIHJlcXVpcmVkIGFtb3VudCBvZiBvdXRwdXQgdG9rZW5zIHRvIHJlY2VpdmUuCiogYHBhdGhgIC0gQSB2ZWN0b3IgcmVwcmVzZW50aW5nIHRoZSB0cmFkaW5nIHJvdXRlLCB3aGVyZSB0aGUgZmlyc3QgZWxlbWVudCBpcyB0aGUgaW5wdXQgdG9rZW4KYW5kIHRoZSBsYXN0IGlzIHRoZSBvdXRwdXQgdG9rZW4uIEludGVybWVkaWF0ZSBlbGVtZW50cyByZXByZXNlbnQgcGFpcnMgdG8gdHJhZGUgdGhyb3VnaC4KKiBgdG9gIC0gVGhlIGFkZHJlc3Mgd2hlcmUgdGhlIG91dHB1dCB0b2tlbnMgd2lsbCBiZSBzZW50IHRvLgoqIGBkZWFkbGluZWAgLSBUaGUgZGVhZGxpbmUgZm9yIGV4ZWN1dGluZyB0aGUgb3BlcmF0aW9uLgoKIyBSZXR1cm5zCkEgdmVjdG9yIGNvbnRhaW5pbmcgdGhlIGFtb3VudHMgb2YgdG9rZW5zIHJlY2VpdmVkIGF0IGVhY2ggc3RlcCBvZiB0aGUgdHJhZGluZyByb3V0ZS4AAAAAABxzd2FwX2V4YWN0X3Rva2Vuc19mb3JfdG9rZW5zAAAABQAAAAAAAAAJYW1vdW50X2luAAAAAAAACwAAAAAAAAAOYW1vdW50X291dF9taW4AAAAAAAsAAAAAAAAABHBhdGgAAAPqAAAAEwAAAAAAAAACdG8AAAAAABMAAAAAAAAACGRlYWRsaW5lAAAABgAAAAEAAAPpAAAD6gAAAAsAAAfQAAAAE0NvbWJpbmVkUm91dGVyRXJyb3IA",
        "AAAAAAAAAzpTd2FwcyB0b2tlbnMgZm9yIGFuIGV4YWN0IGFtb3VudCBvZiBvdXRwdXQgdG9rZW4sIGZvbGxvd2luZyB0aGUgc3BlY2lmaWVkIHRyYWRpbmcgcm91dGUuClRoZSByb3V0ZSBpcyBkZXRlcm1pbmVkIGJ5IHRoZSBgcGF0aGAgdmVjdG9yLCB3aGVyZSB0aGUgZmlyc3QgZWxlbWVudCBpcyB0aGUgaW5wdXQgdG9rZW4sCnRoZSBsYXN0IGlzIHRoZSBvdXRwdXQgdG9rZW4sIGFuZCBhbnkgaW50ZXJtZWRpYXRlIGVsZW1lbnRzIHJlcHJlc2VudCBwYWlycyB0byB0cmFkZSB0aHJvdWdoLgoKIyBBcmd1bWVudHMKKiBgYW1vdW50X291dGAgLSBUaGUgZXhhY3QgYW1vdW50IG9mIG91dHB1dCB0b2tlbiB0byBiZSByZWNlaXZlZC4KKiBgYW1vdW50X2luX21heGAgLSBUaGUgbWF4aW11bSBhbGxvd2VkIGFtb3VudCBvZiBpbnB1dCB0b2tlbnMgdG8gYmUgc3dhcHBlZC4KKiBgcGF0aGAgLSBBIHZlY3RvciByZXByZXNlbnRpbmcgdGhlIHRyYWRpbmcgcm91dGUsIHdoZXJlIHRoZSBmaXJzdCBlbGVtZW50IGlzIHRoZSBpbnB1dCB0b2tlbgphbmQgdGhlIGxhc3QgaXMgdGhlIG91dHB1dCB0b2tlbi4gSW50ZXJtZWRpYXRlIGVsZW1lbnRzIHJlcHJlc2VudCBwYWlycyB0byB0cmFkZSB0aHJvdWdoLgoqIGB0b2AgLSBUaGUgYWRkcmVzcyB3aGVyZSB0aGUgb3V0cHV0IHRva2VucyB3aWxsIGJlIHNlbnQgdG8uCiogYGRlYWRsaW5lYCAtIFRoZSBkZWFkbGluZSBmb3IgZXhlY3V0aW5nIHRoZSBvcGVyYXRpb24uCgojIFJldHVybnMKQSB2ZWN0b3IgY29udGFpbmluZyB0aGUgYW1vdW50cyBvZiB0b2tlbnMgdXNlZCBhdCBlYWNoIHN0ZXAgb2YgdGhlIHRyYWRpbmcgcm91dGUuAAAAAAAcc3dhcF90b2tlbnNfZm9yX2V4YWN0X3Rva2VucwAAAAUAAAAAAAAACmFtb3VudF9vdXQAAAAAAAsAAAAAAAAADWFtb3VudF9pbl9tYXgAAAAAAAALAAAAAAAAAARwYXRoAAAD6gAAABMAAAAAAAAAAnRvAAAAAAATAAAAAAAAAAhkZWFkbGluZQAAAAYAAAABAAAD6QAAA+oAAAALAAAH0AAAABNDb21iaW5lZFJvdXRlckVycm9yAA==",
        "AAAAAAAAAclUaGlzIGZ1bmN0aW9uIHJldHJpZXZlcyB0aGUgZmFjdG9yeSBjb250cmFjdCdzIGFkZHJlc3MgYXNzb2NpYXRlZCB3aXRoIHRoZSBwcm92aWRlZCBlbnZpcm9ubWVudC4KSXQgYWxzbyBjaGVja3MgaWYgdGhlIGZhY3RvcnkgaGFzIGJlZW4gaW5pdGlhbGl6ZWQgYW5kIHJhaXNlcyBhbiBhc3NlcnRpb24gZXJyb3IgaWYgbm90LgpJZiB0aGUgZmFjdG9yeSBpcyBub3QgaW5pdGlhbGl6ZWQsIHRoaXMgY29kZSB3aWxsIHJhaXNlIGFuIGFzc2VydGlvbiBlcnJvciB3aXRoIHRoZSBtZXNzYWdlICJTb3Jvc3dhcFJvdXRlcjogbm90IHlldCBpbml0aWFsaXplZCIuCmh0dHBzOi8vZ2l0aHViLmNvbS9iZW5qYW1pbnNhbG9uL21hbGljaW91c19zb3JvY2hhdAojIEFyZ3VtZW50cwoqIGBlYCAtIFRoZSBjb250cmFjdCBlbnZpcm9ubWVudCAoYEVudmApIGluIHdoaWNoIHRoZSBjb250cmFjdCBpcyBleGVjdXRpbmcuAAAAAAAAC2dldF9mYWN0b3J5AAAAAAAAAAABAAAD6QAAABMAAAfQAAAAE0NvbWJpbmVkUm91dGVyRXJyb3IA",
        "AAAAAAAAAeFDYWxjdWxhdGVzIHRoZSBkZXRlcm1pbmlzdGljIGFkZHJlc3MgZm9yIGEgcGFpciB3aXRob3V0IG1ha2luZyBhbnkgZXh0ZXJuYWwgY2FsbHMuCmNoZWNrIDxodHRwczovL2dpdGh1Yi5jb20vcGFsdGFsYWJzL2RldGVybWluaXN0aWMtYWRkcmVzcy1zb3JvYmFuPgoKIyBBcmd1bWVudHMKCiogYGVgIC0gVGhlIGVudmlyb25tZW50LgoqIGB0b2tlbl9hYCAtIFRoZSBhZGRyZXNzIG9mIHRoZSBmaXJzdCB0b2tlbi4KKiBgdG9rZW5fYmAgLSBUaGUgYWRkcmVzcyBvZiB0aGUgc2Vjb25kIHRva2VuLgoKIyBSZXR1cm5zCgpSZXR1cm5zIGBSZXN1bHQ8QWRkcmVzcywgU29yb3N3YXBMaWJyYXJ5RXJyb3I+YCB3aGVyZSBgT2tgIGNvbnRhaW5zIHRoZSBkZXRlcm1pbmlzdGljIGFkZHJlc3MgZm9yIHRoZSBwYWlyLCBhbmQgYEVycmAgaW5kaWNhdGVzIGFuIGVycm9yIHN1Y2ggYXMgaWRlbnRpY2FsIHRva2VucyBvciBhbiBpc3N1ZSB3aXRoIHNvcnRpbmcuAAAAAAAAD3JvdXRlcl9wYWlyX2ZvcgAAAAACAAAAAAAAAAd0b2tlbl9hAAAAABMAAAAAAAAAB3Rva2VuX2IAAAAAEwAAAAEAAAPpAAAAEwAAB9AAAAATQ29tYmluZWRSb3V0ZXJFcnJvcgA=",
        "AAAAAAAAAcVHaXZlbiBzb21lIGFtb3VudCBvZiBhbiBhc3NldCBhbmQgcGFpciByZXNlcnZlcywgcmV0dXJucyBhbiBlcXVpdmFsZW50IGFtb3VudCBvZiB0aGUgb3RoZXIgYXNzZXQuCgojIEFyZ3VtZW50cwoKKiBgYW1vdW50X2FgIC0gVGhlIGFtb3VudCBvZiB0aGUgZmlyc3QgYXNzZXQuCiogYHJlc2VydmVfYWAgLSBSZXNlcnZlcyBvZiB0aGUgZmlyc3QgYXNzZXQgaW4gdGhlIHBhaXIuCiogYHJlc2VydmVfYmAgLSBSZXNlcnZlcyBvZiB0aGUgc2Vjb25kIGFzc2V0IGluIHRoZSBwYWlyLgoKIyBSZXR1cm5zCgpSZXR1cm5zIGBSZXN1bHQ8aTEyOCwgU29yb3N3YXBMaWJyYXJ5RXJyb3I+YCB3aGVyZSBgT2tgIGNvbnRhaW5zIHRoZSBjYWxjdWxhdGVkIGVxdWl2YWxlbnQgYW1vdW50LCBhbmQgYEVycmAgaW5kaWNhdGVzIGFuIGVycm9yIHN1Y2ggYXMgaW5zdWZmaWNpZW50IGFtb3VudCBvciBsaXF1aWRpdHkAAAAAAAAMcm91dGVyX3F1b3RlAAAAAwAAAAAAAAAIYW1vdW50X2EAAAALAAAAAAAAAAlyZXNlcnZlX2EAAAAAAAALAAAAAAAAAAlyZXNlcnZlX2IAAAAAAAALAAAAAQAAA+kAAAALAAAH0AAAABNDb21iaW5lZFJvdXRlckVycm9yAA==",
        "AAAAAAAAAd1HaXZlbiBhbiBpbnB1dCBhbW91bnQgb2YgYW4gYXNzZXQgYW5kIHBhaXIgcmVzZXJ2ZXMsIHJldHVybnMgdGhlIG1heGltdW0gb3V0cHV0IGFtb3VudCBvZiB0aGUgb3RoZXIgYXNzZXQuCgojIEFyZ3VtZW50cwoKKiBgYW1vdW50X2luYCAtIFRoZSBpbnB1dCBhbW91bnQgb2YgdGhlIGFzc2V0LgoqIGByZXNlcnZlX2luYCAtIFJlc2VydmVzIG9mIHRoZSBpbnB1dCBhc3NldCBpbiB0aGUgcGFpci4KKiBgcmVzZXJ2ZV9vdXRgIC0gUmVzZXJ2ZXMgb2YgdGhlIG91dHB1dCBhc3NldCBpbiB0aGUgcGFpci4KCiMgUmV0dXJucwoKUmV0dXJucyBgUmVzdWx0PGkxMjgsIFNvcm9zd2FwTGlicmFyeUVycm9yPmAgd2hlcmUgYE9rYCBjb250YWlucyB0aGUgY2FsY3VsYXRlZCBtYXhpbXVtIG91dHB1dCBhbW91bnQsIGFuZCBgRXJyYCBpbmRpY2F0ZXMgYW4gZXJyb3Igc3VjaCBhcyBpbnN1ZmZpY2llbnQgaW5wdXQgYW1vdW50IG9yIGxpcXVpZGl0eS4AAAAAAAAVcm91dGVyX2dldF9hbW91bnRfb3V0AAAAAAAAAwAAAAAAAAAJYW1vdW50X2luAAAAAAAACwAAAAAAAAAKcmVzZXJ2ZV9pbgAAAAAACwAAAAAAAAALcmVzZXJ2ZV9vdXQAAAAACwAAAAEAAAPpAAAACwAAB9AAAAATQ29tYmluZWRSb3V0ZXJFcnJvcgA=",
        "AAAAAAAAAdRHaXZlbiBhbiBvdXRwdXQgYW1vdW50IG9mIGFuIGFzc2V0IGFuZCBwYWlyIHJlc2VydmVzLCByZXR1cm5zIGEgcmVxdWlyZWQgaW5wdXQgYW1vdW50IG9mIHRoZSBvdGhlciBhc3NldC4KCiMgQXJndW1lbnRzCgoqIGBhbW91bnRfb3V0YCAtIFRoZSBvdXRwdXQgYW1vdW50IG9mIHRoZSBhc3NldC4KKiBgcmVzZXJ2ZV9pbmAgLSBSZXNlcnZlcyBvZiB0aGUgaW5wdXQgYXNzZXQgaW4gdGhlIHBhaXIuCiogYHJlc2VydmVfb3V0YCAtIFJlc2VydmVzIG9mIHRoZSBvdXRwdXQgYXNzZXQgaW4gdGhlIHBhaXIuCgojIFJldHVybnMKClJldHVybnMgYFJlc3VsdDxpMTI4LCBTb3Jvc3dhcExpYnJhcnlFcnJvcj5gIHdoZXJlIGBPa2AgY29udGFpbnMgdGhlIHJlcXVpcmVkIGlucHV0IGFtb3VudCwgYW5kIGBFcnJgIGluZGljYXRlcyBhbiBlcnJvciBzdWNoIGFzIGluc3VmZmljaWVudCBvdXRwdXQgYW1vdW50IG9yIGxpcXVpZGl0eS4AAAAUcm91dGVyX2dldF9hbW91bnRfaW4AAAADAAAAAAAAAAphbW91bnRfb3V0AAAAAAALAAAAAAAAAApyZXNlcnZlX2luAAAAAAALAAAAAAAAAAtyZXNlcnZlX291dAAAAAALAAAAAQAAA+kAAAALAAAH0AAAABNDb21iaW5lZFJvdXRlckVycm9yAA==",
        "AAAAAAAAAXFQZXJmb3JtcyBjaGFpbmVkIGdldF9hbW91bnRfb3V0IGNhbGN1bGF0aW9ucyBvbiBhbnkgbnVtYmVyIG9mIHBhaXJzLgoKIyBBcmd1bWVudHMKCiogYGVgIC0gVGhlIGVudmlyb25tZW50LgoqIGBhbW91bnRfaW5gIC0gVGhlIGlucHV0IGFtb3VudC4KKiBgcGF0aGAgLSBWZWN0b3Igb2YgdG9rZW4gYWRkcmVzc2VzIHJlcHJlc2VudGluZyB0aGUgcGF0aC4KCiMgUmV0dXJucwoKUmV0dXJucyBgUmVzdWx0PFZlYzxpMTI4PiwgU29yb3N3YXBMaWJyYXJ5RXJyb3I+YCB3aGVyZSBgT2tgIGNvbnRhaW5zIGEgdmVjdG9yIG9mIGNhbGN1bGF0ZWQgYW1vdW50cywgYW5kIGBFcnJgIGluZGljYXRlcyBhbiBlcnJvciBzdWNoIGFzIGFuIGludmFsaWQgcGF0aC4AAAAAAAAWcm91dGVyX2dldF9hbW91bnRzX291dAAAAAAAAgAAAAAAAAAJYW1vdW50X2luAAAAAAAACwAAAAAAAAAEcGF0aAAAA+oAAAATAAAAAQAAA+kAAAPqAAAACwAAB9AAAAATQ29tYmluZWRSb3V0ZXJFcnJvcgA=",
        "AAAAAAAAAXJQZXJmb3JtcyBjaGFpbmVkIGdldF9hbW91bnRfaW4gY2FsY3VsYXRpb25zIG9uIGFueSBudW1iZXIgb2YgcGFpcnMuCgojIEFyZ3VtZW50cwoKKiBgZWAgLSBUaGUgZW52aXJvbm1lbnQuCiogYGFtb3VudF9vdXRgIC0gVGhlIG91dHB1dCBhbW91bnQuCiogYHBhdGhgIC0gVmVjdG9yIG9mIHRva2VuIGFkZHJlc3NlcyByZXByZXNlbnRpbmcgdGhlIHBhdGguCgojIFJldHVybnMKClJldHVybnMgYFJlc3VsdDxWZWM8aTEyOD4sIFNvcm9zd2FwTGlicmFyeUVycm9yPmAgd2hlcmUgYE9rYCBjb250YWlucyBhIHZlY3RvciBvZiBjYWxjdWxhdGVkIGFtb3VudHMsIGFuZCBgRXJyYCBpbmRpY2F0ZXMgYW4gZXJyb3Igc3VjaCBhcyBhbiBpbnZhbGlkIHBhdGguAAAAAAAVcm91dGVyX2dldF9hbW91bnRzX2luAAAAAAAAAgAAAAAAAAAKYW1vdW50X291dAAAAAAACwAAAAAAAAAEcGF0aAAAA+oAAAATAAAAAQAAA+kAAAPqAAAACwAAB9AAAAATQ29tYmluZWRSb3V0ZXJFcnJvcgA=",
        "AAAABAAAAAAAAAAAAAAAFFNvcm9zd2FwTGlicmFyeUVycm9yAAAABgAAACRTb3Jvc3dhcExpYnJhcnk6IGluc3VmZmljaWVudCBhbW91bnQAAAASSW5zdWZmaWNpZW50QW1vdW50AAAAAAEtAAAAJ1Nvcm9zd2FwTGlicmFyeTogaW5zdWZmaWNpZW50IGxpcXVpZGl0eQAAAAAVSW5zdWZmaWNpZW50TGlxdWlkaXR5AAAAAAABLgAAACpTb3Jvc3dhcExpYnJhcnk6IGluc3VmZmljaWVudCBpbnB1dCBhbW91bnQAAAAAABdJbnN1ZmZpY2llbnRJbnB1dEFtb3VudAAAAAEvAAAAK1Nvcm9zd2FwTGlicmFyeTogaW5zdWZmaWNpZW50IG91dHB1dCBhbW91bnQAAAAAGEluc3VmZmljaWVudE91dHB1dEFtb3VudAAAATAAAAAdU29yb3N3YXBMaWJyYXJ5OiBpbnZhbGlkIHBhdGgAAAAAAAALSW52YWxpZFBhdGgAAAABMQAAAD1Tb3Jvc3dhcExpYnJhcnk6IHRva2VuX2EgYW5kIHRva2VuX2IgaGF2ZSBpZGVudGljYWwgYWRkcmVzc2VzAAAAAAAAE1NvcnRJZGVudGljYWxUb2tlbnMAAAABMg==",
        "AAAAAAAAAVZTb3J0cyB0d28gdG9rZW4gYWRkcmVzc2VzIGluIGEgY29uc2lzdGVudCBvcmRlci4KCiMgQXJndW1lbnRzCgoqIGB0b2tlbl9hYCAtIFRoZSBhZGRyZXNzIG9mIHRoZSBmaXJzdCB0b2tlbi4KKiBgdG9rZW5fYmAgLSBUaGUgYWRkcmVzcyBvZiB0aGUgc2Vjb25kIHRva2VuLgoKIyBSZXR1cm5zCgpSZXR1cm5zIGBSZXN1bHQ8KEFkZHJlc3MsIEFkZHJlc3MpLCBTb3Jvc3dhcExpYnJhcnlFcnJvcj5gIHdoZXJlIGBPa2AgY29udGFpbnMgYSB0dXBsZSB3aXRoIHRoZSBzb3J0ZWQgdG9rZW4gYWRkcmVzc2VzLCBhbmQgYEVycmAgaW5kaWNhdGVzIGFuIGVycm9yIHN1Y2ggYXMgaWRlbnRpY2FsIHRva2Vucy4AAAAAAAtzb3J0X3Rva2VucwAAAAACAAAAAAAAAAd0b2tlbl9hAAAAABMAAAAAAAAAB3Rva2VuX2IAAAAAEwAAAAEAAAPpAAAD7QAAAAIAAAATAAAAEwAAB9AAAAAUU29yb3N3YXBMaWJyYXJ5RXJyb3I=",
        "AAAAAAAAAgRDYWxjdWxhdGVzIHRoZSBkZXRlcm1pbmlzdGljIGFkZHJlc3MgZm9yIGEgcGFpciB3aXRob3V0IG1ha2luZyBhbnkgZXh0ZXJuYWwgY2FsbHMuCmNoZWNrIDxodHRwczovL2dpdGh1Yi5jb20vcGFsdGFsYWJzL2RldGVybWluaXN0aWMtYWRkcmVzcy1zb3JvYmFuPgoKIyBBcmd1bWVudHMKCiogYGVgIC0gVGhlIGVudmlyb25tZW50LgoqIGBmYWN0b3J5YCAtIFRoZSBmYWN0b3J5IGFkZHJlc3MuCiogYHRva2VuX2FgIC0gVGhlIGFkZHJlc3Mgb2YgdGhlIGZpcnN0IHRva2VuLgoqIGB0b2tlbl9iYCAtIFRoZSBhZGRyZXNzIG9mIHRoZSBzZWNvbmQgdG9rZW4uCgojIFJldHVybnMKClJldHVybnMgYFJlc3VsdDxBZGRyZXNzLCBTb3Jvc3dhcExpYnJhcnlFcnJvcj5gIHdoZXJlIGBPa2AgY29udGFpbnMgdGhlIGRldGVybWluaXN0aWMgYWRkcmVzcyBmb3IgdGhlIHBhaXIsIGFuZCBgRXJyYCBpbmRpY2F0ZXMgYW4gZXJyb3Igc3VjaCBhcyBpZGVudGljYWwgdG9rZW5zIG9yIGFuIGlzc3VlIHdpdGggc29ydGluZy4AAAAIcGFpcl9mb3IAAAADAAAAAAAAAAdmYWN0b3J5AAAAABMAAAAAAAAAB3Rva2VuX2EAAAAAEwAAAAAAAAAHdG9rZW5fYgAAAAATAAAAAQAAA+kAAAATAAAH0AAAABRTb3Jvc3dhcExpYnJhcnlFcnJvcg==",
        "AAAAAAAAAZxGZXRjaGVzIGFuZCBzb3J0cyB0aGUgcmVzZXJ2ZXMgZm9yIGEgcGFpciBvZiB0b2tlbnMuCgojIEFyZ3VtZW50cwoKKiBgZWAgLSBUaGUgZW52aXJvbm1lbnQuCiogYGZhY3RvcnlgIC0gVGhlIGZhY3RvcnkgYWRkcmVzcy4KKiBgdG9rZW5fYWAgLSBUaGUgYWRkcmVzcyBvZiB0aGUgZmlyc3QgdG9rZW4uCiogYHRva2VuX2JgIC0gVGhlIGFkZHJlc3Mgb2YgdGhlIHNlY29uZCB0b2tlbi4KCiMgUmV0dXJucwoKUmV0dXJucyBgUmVzdWx0PChpMTI4LCBpMTI4KSwgU29yb3N3YXBMaWJyYXJ5RXJyb3I+YCB3aGVyZSBgT2tgIGNvbnRhaW5zIGEgdHVwbGUgb2Ygc29ydGVkIHJlc2VydmVzLCBhbmQgYEVycmAgaW5kaWNhdGVzIGFuIGVycm9yIHN1Y2ggYXMgaWRlbnRpY2FsIHRva2VucyBvciBhbiBpc3N1ZSB3aXRoIHNvcnRpbmcuAAAADGdldF9yZXNlcnZlcwAAAAMAAAAAAAAAB2ZhY3RvcnkAAAAAEwAAAAAAAAAHdG9rZW5fYQAAAAATAAAAAAAAAAd0b2tlbl9iAAAAABMAAAABAAAD6QAAA+0AAAACAAAACwAAAAsAAAfQAAAAFFNvcm9zd2FwTGlicmFyeUVycm9y",
        "AAAAAAAAAcVHaXZlbiBzb21lIGFtb3VudCBvZiBhbiBhc3NldCBhbmQgcGFpciByZXNlcnZlcywgcmV0dXJucyBhbiBlcXVpdmFsZW50IGFtb3VudCBvZiB0aGUgb3RoZXIgYXNzZXQuCgojIEFyZ3VtZW50cwoKKiBgYW1vdW50X2FgIC0gVGhlIGFtb3VudCBvZiB0aGUgZmlyc3QgYXNzZXQuCiogYHJlc2VydmVfYWAgLSBSZXNlcnZlcyBvZiB0aGUgZmlyc3QgYXNzZXQgaW4gdGhlIHBhaXIuCiogYHJlc2VydmVfYmAgLSBSZXNlcnZlcyBvZiB0aGUgc2Vjb25kIGFzc2V0IGluIHRoZSBwYWlyLgoKIyBSZXR1cm5zCgpSZXR1cm5zIGBSZXN1bHQ8aTEyOCwgU29yb3N3YXBMaWJyYXJ5RXJyb3I+YCB3aGVyZSBgT2tgIGNvbnRhaW5zIHRoZSBjYWxjdWxhdGVkIGVxdWl2YWxlbnQgYW1vdW50LCBhbmQgYEVycmAgaW5kaWNhdGVzIGFuIGVycm9yIHN1Y2ggYXMgaW5zdWZmaWNpZW50IGFtb3VudCBvciBsaXF1aWRpdHkAAAAAAAAFcXVvdGUAAAAAAAADAAAAAAAAAAhhbW91bnRfYQAAAAsAAAAAAAAACXJlc2VydmVfYQAAAAAAAAsAAAAAAAAACXJlc2VydmVfYgAAAAAAAAsAAAABAAAD6QAAAAsAAAfQAAAAFFNvcm9zd2FwTGlicmFyeUVycm9y",
        "AAAAAAAAAd1HaXZlbiBhbiBpbnB1dCBhbW91bnQgb2YgYW4gYXNzZXQgYW5kIHBhaXIgcmVzZXJ2ZXMsIHJldHVybnMgdGhlIG1heGltdW0gb3V0cHV0IGFtb3VudCBvZiB0aGUgb3RoZXIgYXNzZXQuCgojIEFyZ3VtZW50cwoKKiBgYW1vdW50X2luYCAtIFRoZSBpbnB1dCBhbW91bnQgb2YgdGhlIGFzc2V0LgoqIGByZXNlcnZlX2luYCAtIFJlc2VydmVzIG9mIHRoZSBpbnB1dCBhc3NldCBpbiB0aGUgcGFpci4KKiBgcmVzZXJ2ZV9vdXRgIC0gUmVzZXJ2ZXMgb2YgdGhlIG91dHB1dCBhc3NldCBpbiB0aGUgcGFpci4KCiMgUmV0dXJucwoKUmV0dXJucyBgUmVzdWx0PGkxMjgsIFNvcm9zd2FwTGlicmFyeUVycm9yPmAgd2hlcmUgYE9rYCBjb250YWlucyB0aGUgY2FsY3VsYXRlZCBtYXhpbXVtIG91dHB1dCBhbW91bnQsIGFuZCBgRXJyYCBpbmRpY2F0ZXMgYW4gZXJyb3Igc3VjaCBhcyBpbnN1ZmZpY2llbnQgaW5wdXQgYW1vdW50IG9yIGxpcXVpZGl0eS4AAAAAAAAOZ2V0X2Ftb3VudF9vdXQAAAAAAAMAAAAAAAAACWFtb3VudF9pbgAAAAAAAAsAAAAAAAAACnJlc2VydmVfaW4AAAAAAAsAAAAAAAAAC3Jlc2VydmVfb3V0AAAAAAsAAAABAAAD6QAAAAsAAAfQAAAAFFNvcm9zd2FwTGlicmFyeUVycm9y",
        "AAAAAAAAAdRHaXZlbiBhbiBvdXRwdXQgYW1vdW50IG9mIGFuIGFzc2V0IGFuZCBwYWlyIHJlc2VydmVzLCByZXR1cm5zIGEgcmVxdWlyZWQgaW5wdXQgYW1vdW50IG9mIHRoZSBvdGhlciBhc3NldC4KCiMgQXJndW1lbnRzCgoqIGBhbW91bnRfb3V0YCAtIFRoZSBvdXRwdXQgYW1vdW50IG9mIHRoZSBhc3NldC4KKiBgcmVzZXJ2ZV9pbmAgLSBSZXNlcnZlcyBvZiB0aGUgaW5wdXQgYXNzZXQgaW4gdGhlIHBhaXIuCiogYHJlc2VydmVfb3V0YCAtIFJlc2VydmVzIG9mIHRoZSBvdXRwdXQgYXNzZXQgaW4gdGhlIHBhaXIuCgojIFJldHVybnMKClJldHVybnMgYFJlc3VsdDxpMTI4LCBTb3Jvc3dhcExpYnJhcnlFcnJvcj5gIHdoZXJlIGBPa2AgY29udGFpbnMgdGhlIHJlcXVpcmVkIGlucHV0IGFtb3VudCwgYW5kIGBFcnJgIGluZGljYXRlcyBhbiBlcnJvciBzdWNoIGFzIGluc3VmZmljaWVudCBvdXRwdXQgYW1vdW50IG9yIGxpcXVpZGl0eS4AAAANZ2V0X2Ftb3VudF9pbgAAAAAAAAMAAAAAAAAACmFtb3VudF9vdXQAAAAAAAsAAAAAAAAACnJlc2VydmVfaW4AAAAAAAsAAAAAAAAAC3Jlc2VydmVfb3V0AAAAAAsAAAABAAAD6QAAAAsAAAfQAAAAFFNvcm9zd2FwTGlicmFyeUVycm9y",
        "AAAAAAAAAZRQZXJmb3JtcyBjaGFpbmVkIGdldF9hbW91bnRfb3V0IGNhbGN1bGF0aW9ucyBvbiBhbnkgbnVtYmVyIG9mIHBhaXJzLgoKIyBBcmd1bWVudHMKCiogYGVgIC0gVGhlIGVudmlyb25tZW50LgoqIGBmYWN0b3J5YCAtIFRoZSBmYWN0b3J5IGFkZHJlc3MuCiogYGFtb3VudF9pbmAgLSBUaGUgaW5wdXQgYW1vdW50LgoqIGBwYXRoYCAtIFZlY3RvciBvZiB0b2tlbiBhZGRyZXNzZXMgcmVwcmVzZW50aW5nIHRoZSBwYXRoLgoKIyBSZXR1cm5zCgpSZXR1cm5zIGBSZXN1bHQ8VmVjPGkxMjg+LCBTb3Jvc3dhcExpYnJhcnlFcnJvcj5gIHdoZXJlIGBPa2AgY29udGFpbnMgYSB2ZWN0b3Igb2YgY2FsY3VsYXRlZCBhbW91bnRzLCBhbmQgYEVycmAgaW5kaWNhdGVzIGFuIGVycm9yIHN1Y2ggYXMgYW4gaW52YWxpZCBwYXRoLgAAAA9nZXRfYW1vdW50c19vdXQAAAAAAwAAAAAAAAAHZmFjdG9yeQAAAAATAAAAAAAAAAlhbW91bnRfaW4AAAAAAAALAAAAAAAAAARwYXRoAAAD6gAAABMAAAABAAAD6QAAA+oAAAALAAAH0AAAABRTb3Jvc3dhcExpYnJhcnlFcnJvcg==",
        "AAAAAAAAAZVQZXJmb3JtcyBjaGFpbmVkIGdldF9hbW91bnRfaW4gY2FsY3VsYXRpb25zIG9uIGFueSBudW1iZXIgb2YgcGFpcnMuCgojIEFyZ3VtZW50cwoKKiBgZWAgLSBUaGUgZW52aXJvbm1lbnQuCiogYGZhY3RvcnlgIC0gVGhlIGZhY3RvcnkgYWRkcmVzcy4KKiBgYW1vdW50X291dGAgLSBUaGUgb3V0cHV0IGFtb3VudC4KKiBgcGF0aGAgLSBWZWN0b3Igb2YgdG9rZW4gYWRkcmVzc2VzIHJlcHJlc2VudGluZyB0aGUgcGF0aC4KCiMgUmV0dXJucwoKUmV0dXJucyBgUmVzdWx0PFZlYzxpMTI4PiwgU29yb3N3YXBMaWJyYXJ5RXJyb3I+YCB3aGVyZSBgT2tgIGNvbnRhaW5zIGEgdmVjdG9yIG9mIGNhbGN1bGF0ZWQgYW1vdW50cywgYW5kIGBFcnJgIGluZGljYXRlcyBhbiBlcnJvciBzdWNoIGFzIGFuIGludmFsaWQgcGF0aC4AAAAAAAAOZ2V0X2Ftb3VudHNfaW4AAAAAAAMAAAAAAAAAB2ZhY3RvcnkAAAAAEwAAAAAAAAAKYW1vdW50X291dAAAAAAACwAAAAAAAAAEcGF0aAAAA+oAAAATAAAAAQAAA+kAAAPqAAAACwAAB9AAAAAUU29yb3N3YXBMaWJyYXJ5RXJyb3I=" ]),
      options
    )
  }
  public readonly fromJSON = {
    initialize: this.txFromJSON<Result<void>>,
        add_liquidity: this.txFromJSON<Result<readonly [i128, i128, i128]>>,
        remove_liquidity: this.txFromJSON<Result<readonly [i128, i128]>>,
        swap_exact_tokens_for_tokens: this.txFromJSON<Result<Array<i128>>>,
        swap_tokens_for_exact_tokens: this.txFromJSON<Result<Array<i128>>>,
        get_factory: this.txFromJSON<Result<string>>,
        router_pair_for: this.txFromJSON<Result<string>>,
        router_quote: this.txFromJSON<Result<i128>>,
        router_get_amount_out: this.txFromJSON<Result<i128>>,
        router_get_amount_in: this.txFromJSON<Result<i128>>,
        router_get_amounts_out: this.txFromJSON<Result<Array<i128>>>,
        router_get_amounts_in: this.txFromJSON<Result<Array<i128>>>,
        sort_tokens: this.txFromJSON<Result<readonly [string, string]>>,
        pair_for: this.txFromJSON<Result<string>>,
        get_reserves: this.txFromJSON<Result<readonly [i128, i128]>>,
        quote: this.txFromJSON<Result<i128>>,
        get_amount_out: this.txFromJSON<Result<i128>>,
        get_amount_in: this.txFromJSON<Result<i128>>,
        get_amounts_out: this.txFromJSON<Result<Array<i128>>>,
        get_amounts_in: this.txFromJSON<Result<Array<i128>>>
  }
}