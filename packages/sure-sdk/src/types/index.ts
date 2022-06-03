import * as anchor from '@project-serum/anchor';
import { PublicKey, AccountInfo, ParsedAccountData } from '@solana/web3.js';
// Account representing an spl-token
export interface TokenAccount {
	pubkey: PublicKey;
	account: AccountInfo<ParsedAccountData>;
}

/// --- INSURANCE ---
export interface InsuranceContractsInfo {
	insuredAmount: anchor.BN;
	expiryTs: anchor.BN;
}

/// --- POOLS ---
export interface PoolAccount {
	/// Name of pool visible to the user
	name: string; // 4 + 200 bytes

	/// Token Mint
	tokenMint: PublicKey; // 32 bytes

	/// Fee paid when buying insurance.
	/// in basis points
	insuranceFee: number; // 4 bytes

	/// The total liquidity in the pool
	liquidity: anchor.BN; // 8 bytes

	/// Used Liquidity in the pool
	usedLiquidity: anchor.BN; // 8 bytes

	/// Current premium rate in basis points (0.01%).
	premiumRate: anchor.BN; // 8 bytes

	/// The public key of the smart contract that is
	/// insured
	smartContract: PublicKey; // 32 bytes

	/// Whether the insurance pool is locked
	locked: boolean; // 1 byte
}
