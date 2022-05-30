import * as anchor from '@project-serum/anchor';
import { Connection, PublicKey } from '@solana/web3.js';
import { IDL, SurePool } from './../anchor/types/sure_pool';
import {
	SURE_POOLS_SEED,
	SURE_TICK_SEED,
	POOL_SEED,
	SURE_BITMAP,
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
		const [surePoolsPDA, surePoolsBump] = await PublicKey.findProgramAddress(
			[SURE_POOLS_SEED],
			this.program.programId
		);

		return surePoolsPDA;
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
		poolPDA: PublicKey,
		tokenMint: PublicKey,
		tick: number,
		creator: PublicKey
	): Promise<PublicKey> {
		const tickAccountPDA = await this.getTickAccountPDA(
			poolPDA,
			tokenMint,
			tick
		);

		try {
			await this.program.methods
				.initializeTick(poolPDA, tokenMint, tick)
				.accounts({
					creator: creator,
					tickAccount: tickAccountPDA,
					systemProgram: this.program.programId,
				});
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
				await this.createTickAccount(pool, tokenMint, tick, owner);
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
}
