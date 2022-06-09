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

	/// Fee paid when buying insurance.
	/// in basis points
	insuranceFee: number; // 4 bytes

	/// The public key of the smart contract that is
	/// insured
	smartContract: PublicKey; // 32 bytes

	/// Token pools
	tokenPools: PublicKey[];

	/// Whether the insurance pool is locked
	locked: boolean; // 1 byte
}

export interface TokenPool {
	/// Token mint of pool
	tokenMint: PublicKey; // 32 bytes

	/// Liquidity in Token Pool
	liquidity: anchor.BN; // 8 bytes

	/// Used liquidity
	usedLiquidity: anchor.BN; // 8 bytes
}

export interface PoolInformation {
	name: string;
	tokenMint: PublicKey;
	insuranceFee: number;
	smartContract: PublicKey;
	liquidity: string;
	usedLiquidity: string;
	lowestPremium: number;
	locked: boolean;
}

export interface PoolInsuranceContract {
	/// The bump
	bump: number; //1 byte

	/// Contract expiry
	expiryTs: anchor.BN; // 8 byte

	/// Contract Amount
	insuredAmount: anchor.BN; // 8 byte

	/// token mint
	tokenMint: PublicKey;

	/// Owner of contract
	owner: PublicKey; // 32 byte
}

export interface LiquidityTickInfo {
	/// The bump identity of the PDA
	bump: number; // 1 byte

	/// The active liquidity at the tick
	liquidity: anchor.BN; // 8bytes

	/// Amount of liquidity used from the pool
	usedLiquidity: anchor.BN; // 8 bytes

	/// token mint used as liqudiity
	tokenMint: PublicKey;

	/// last slot the tick was updated on
	lastUpdated: anchor.BN; // 8 bytes

	/// The tick in basis points
	tick: number; // 8 bytes

	/// Boolean representing whether the liquidity is active
	active: boolean; // 1 byte

	/// Ids of liquidity positions
	liquidityPositionId: number[]; // 1*255 =255

	/// Accumulation of Liquidity Provided
	liquidityPositionAccumulated: anchor.BN[]; // 8*255 =

	/// rewards
	liquidityPositionRewards: anchor.BN[]; // 8*255

	lastLiquidityPositionIdx: number; // 1
}

export interface TokenPoolStatistics {
	// pool
	pool: PublicKey;

	// token mint
	tokenMint: PublicKey;

	// amount insured in pool
	amountInsured: anchor.BN;

	// Provided liquidity
	liquidity: anchor.BN;

	// Lowest premium
	premiumLow: number;

	// Highest premium
	premiumHigh: number;
}
