import * as anchor from '@project-serum/anchor';
import { Connection, PublicKey, SystemProgram } from '@solana/web3.js';
import { IDL, SurePool } from './../anchor/types/sure_pool';
import {
	SURE_POOLS_SEED,
	SURE_TICK_SEED,
	POOL_SEED,
	SURE_BITMAP,
	SURE_PREMIUM_POOL_SEED,
	SURE_VAULT_POOL_SEED,
	SURE_PROTOCOL_OWNER,
	SURE_TOKEN_POOL_SEED,
} from './seeds';
import { LiquidityTickInfo } from 'src/types';
import { Account, getMint } from '@solana/spl-token';

export class Common {
	constructor(
		readonly program: anchor.Program<SurePool>,
		readonly connection: Connection,
		readonly wallet: anchor.Wallet
	) {}

	// async convertBNFromDecimals(
	// 	amount: anchor.BN,
	// 	mint: PublicKey
	// ): Promise<string> {
	// 	const mintInfo = await getMint(this.connection, mint);
	// 	const base = new anchor.BN(10 ** mintInfo.decimals);
	// 	let fraction = amount.mod(base).toString(10);
	// 	while (fraction.length < mintInfo.decimals) {
	// 		fraction = `0${fraction}`;
	// 	}
	// 	const whole = amount.div(base).toString(10);
	// 	return `${whole}${fraction == '0' ? '' : `.${fraction}`}`;
	// }

	// async convertBNToDecimals(
	// 	amount: anchor.BN,
	// 	mint: PublicKey
	// ): Promise<anchor.BN> {
	// 	const mintInfo = await getMint(this.connection, mint);
	// 	return amount.mul(new anchor.BN(10 ** mintInfo.decimals));
	// }

	async getProtocolOwner(): Promise<[PublicKey, number]> {
		return await PublicKey.findProgramAddress(
			[SURE_PROTOCOL_OWNER],
			this.program.programId
		);
	}

	async getPoolsPDA(): Promise<PublicKey> {
		try {
			const [surePoolsPDA, surePoolsBump] = await PublicKey.findProgramAddress(
				[SURE_POOLS_SEED],
				this.program.programId
			);

			return surePoolsPDA;
		} catch (err) {
			throw new Error('sure.common.getSurePoolsPDA. Cause: ' + err);
		}
	}

	async getPools() {
		try {
			const surePoolsPDA = await this.getPoolsPDA();
			const surePools = await this.program.account.surePools.fetch(
				surePoolsPDA
			);
			return surePools;
		} catch (err) {
			throw new Error('sure.common.getSurePools. Cause: ' + err);
		}
	}

	async getPoolPDA(
		smartContractToInsure: PublicKey
	): Promise<anchor.web3.PublicKey> {
		const [poolPDA, poolBump] = await PublicKey.findProgramAddress(
			[POOL_SEED, smartContractToInsure.toBytes()],
			this.program.programId
		);
		return poolPDA;
	}

	/**
	 * Get the public key of the token pool
	 *
	 * The token pool holds information about the pool
	 * for the given token
	 *
	 * @param pool: Publickey of the parent pool
	 * @param tokenMint: Publickey of the mint used in the pool
	 * @returns PDA
	 */
	async getTokenPoolPDA(
		pool: PublicKey,
		tokenMint: PublicKey
	): Promise<anchor.web3.PublicKey> {
		const [tokenPoolPDA, _] = await PublicKey.findProgramAddress(
			[SURE_TOKEN_POOL_SEED, pool.toBytes(), tokenMint.toBytes()],
			this.program.programId
		);
		return tokenPoolPDA;
	}

	async getPoolLiquidityTickBitmapPDA(
		pool: PublicKey,
		tokenMint: PublicKey
	): Promise<anchor.web3.PublicKey> {
		const [bitmapPDA, bitmapBump] = await PublicKey.findProgramAddress(
			[SURE_BITMAP, pool.toBytes(), tokenMint.toBytes()],
			this.program.programId
		);
		return bitmapPDA;
	}

	/// ============ TICK =================
	async getLiquidityTickInfoPDA(
		pool: PublicKey,
		tokenMint: PublicKey,
		tick: number
	): Promise<PublicKey> {
		let tickBN = new anchor.BN(tick);
		const [tickAccountPDA, tickAccountBump] =
			await PublicKey.findProgramAddress(
				[
					SURE_TICK_SEED,
					pool.toBytes(),
					tokenMint.toBytes(),
					tickBN.toBuffer('le', 2),
				],
				this.program.programId
			);
		return tickAccountPDA;
	}

	async getLiquidityTickInfo(
		pool: PublicKey,
		tokenMint: PublicKey,
		tick: number
	): Promise<LiquidityTickInfo> {
		const liqudityTickInfoPDA = await this.getLiquidityTickInfoPDA(
			pool,
			tokenMint,
			tick
		);
		try {
			const liquidityTickInfo = await this.program.account.tick.fetch(
				liqudityTickInfoPDA
			);
			return liquidityTickInfo;
		} catch (err) {
			throw new Error(
				'sure-sdk.common.getLiquidityTickInfo.error. Cause: ' + err
			);
		}
	}

	/// Check if tick account exists for the pool,
	/// if not, create the account.
	async createLiquidityTickInfo(
		pool: PublicKey,
		tokenMint: PublicKey,
		tick: number
	): Promise<anchor.web3.TransactionInstruction> {
		const liquidityTickInfoPDA = await this.getLiquidityTickInfoPDA(
			pool,
			tokenMint,
			tick
		);

		try {
			return this.program.methods
				.initializePoolLiquidityTick(pool, tokenMint, tick)
				.accounts({
					creator: this.wallet.publicKey,
					liquidityTickInfo: liquidityTickInfoPDA,
					systemProgram: SystemProgram.programId,
				})
				.instruction();
		} catch (e) {
			console.log('logs?: ', e.logs);
			throw new Error('Could not create tick account: ' + e);
		}
	}

	// async getOrCreateLiquidityTickInfo(
	// 	pool: PublicKey,
	// 	tokenMint: PublicKey,
	// 	tick: number
	// ): Promise<anchor.web3.PublicKey> {
	// 	const liquidityTickInfo = await this.getLiquidityTickInfoPDA(
	// 		pool,
	// 		tokenMint,
	// 		tick
	// 	);

	// 	try {
	// 		await this.program.account.tick.fetch(liquidityTickInfo);
	// 	} catch (e) {
	// 		console.log(
	// 			'sure.getTickAccount.error Could not fetch tick account. Cause: ' + e
	// 		);
	// 		// create account
	// 		try {
	// 			await this.createLiquidityTickInfo(pool, tokenMint, tick);
	// 		} catch (e) {
	// 			throw new Error(
	// 				'sure.createTickAccount.error. could not create tick account. cause: ' +
	// 					e
	// 			);
	// 		}
	// 	}
	// 	return liquidityTickInfo;
	// }

	/**
	 * Current tick position in tick pool
	 *
	 * @param poolPDA PDA for pool
	 * @param tick Tick in basis points to supply liquidity to
	 * @param tokenMint The mint of the token to be supplied to the pool. This could be USDC
	 * @return Nothing
	 */
	async getCurrentTickPosition(
		poolPDA: PublicKey,
		tokenMint: PublicKey,
		tick: number
	): Promise<number> {
		const liquidityTickInfoPDA = await this.getLiquidityTickInfoPDA(
			poolPDA,
			tokenMint,
			tick
		);
		try {
			const liquidityTickInfo = await this.program.account.tick.fetch(
				liquidityTickInfoPDA
			);
			return liquidityTickInfo.lastLiquidityPositionIdx;
		} catch (e) {
			return 0;
		}
	}

	/**
	 * Get the Premium Vault PDA
	 *
	 * @param pool      Pool associated with the premium vault
	 * @param tokenMint The token mint for the premium vault
	 */
	public async getPremiumVaultPDA(
		pool: PublicKey,
		tokenMint: PublicKey
	): Promise<PublicKey> {
		const [premiumVaultPDA, premiumVaultBump] =
			await PublicKey.findProgramAddress(
				[SURE_PREMIUM_POOL_SEED, pool.toBytes(), tokenMint.toBytes()],
				this.program.programId
			);
		return premiumVaultPDA;
	}

	async getPoolVaultPDA(
		pool: PublicKey,
		tokenMint: PublicKey
	): Promise<PublicKey> {
		const [liquidityVaultPDA, liquidityVaultBump] =
			await PublicKey.findProgramAddress(
				[SURE_VAULT_POOL_SEED, pool.toBytes(), tokenMint.toBytes()],
				this.program.programId
			);
		return liquidityVaultPDA;
	}
}
