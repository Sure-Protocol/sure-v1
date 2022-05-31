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
} from './seeds';
import { PROGRAM_ID } from './constants';

export class Common {
	constructor(
		readonly program: anchor.Program<SurePool>,
		readonly connection: Connection,
		readonly wallet: anchor.Wallet
	) {}

	static init(connection: anchor.web3.Connection, wallet: anchor.Wallet) {
		const provider = new anchor.AnchorProvider(connection, wallet, {
			skipPreflight: false,
		});

		const sureProgram = new anchor.Program<SurePool>(IDL, PROGRAM_ID, provider);

		return new this(sureProgram, connection, wallet);
	}

	async getProtocolOwner(): Promise<[PublicKey, number]> {
		return await PublicKey.findProgramAddress([], this.program.programId);
	}

	async getSurePools(): Promise<PublicKey> {
		try {
			const [surePoolsPDA, surePoolsBump] = await PublicKey.findProgramAddress(
				[SURE_POOLS_SEED],
				this.program.programId
			);

			return surePoolsPDA;
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

	async getLiquidityPositionBitmapPDA(
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
	getTickAccountPDA = async (
		pool: PublicKey,
		tokenMint: PublicKey,
		tick: number
	): Promise<PublicKey> => {
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
	};

	/// Check if tick account exists for the pool,
	/// if not, create the account.
	async createTickAccount(
		pool: PublicKey,
		tokenMint: PublicKey,
		tick: number
	): Promise<PublicKey> {
		const tickAccountPDA = await this.getTickAccountPDA(pool, tokenMint, tick);

		try {
			await this.program.methods
				.initializeTick(pool, tokenMint, tick)
				.accounts({
					creator: this.wallet.publicKey,
					tickAccount: tickAccountPDA,
					systemProgram: SystemProgram.programId,
				})
				.rpc();
		} catch (e) {
			console.log('logs?: ', e.logs);
			throw new Error('Could not create tick account: ' + e);
		}

		return tickAccountPDA;
	}

	async getOrCreateTickAccount(
		owner: PublicKey,
		pool: PublicKey,
		tokenMint: PublicKey,
		tick: number
	): Promise<anchor.web3.PublicKey> {
		const tickAccountPDA = await this.getTickAccountPDA(pool, tokenMint, tick);

		try {
			await this.program.account.tick.fetch(tickAccountPDA);
		} catch (e) {
			console.log(
				'sure.getTickAccount.error Could not fetch tick account. Cause: ' + e
			);
			// create account
			try {
				await this.createTickAccount(pool, tokenMint, tick);
			} catch (e) {
				throw new Error(
					'sure.createTickAccount.error. could not create tick account. cause: ' +
						e
				);
			}
		}
		return tickAccountPDA;
	}

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
		const tickPDA = await this.getTickAccountPDA(poolPDA, tokenMint, tick);
		try {
			const tickAccount = await this.program.account.tick.fetch(tickPDA);
			return tickAccount.lastLiquidityPositionIdx;
		} catch (e) {
			throw new Error('Tick account does not exist. Cause: ' + e);
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

	async getLiquidityVaultPDA(
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
