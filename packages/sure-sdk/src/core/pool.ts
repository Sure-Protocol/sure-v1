import * as anchor from '@project-serum/anchor';
import { TOKEN_PROGRAM_ID } from '@solana/spl-token';
import {
	Connection,
	PublicKey,
	SystemProgram,
	TransactionInstruction,
} from '@solana/web3.js';
import {
	LiquidityTickInfo,
	PoolAccount,
	PoolInformation,
	TokenPool,
	TokenPoolStatistics,
} from 'src/types';
import { Bitmap, Money, sendTransaction } from './../utils';
import { SurePool } from './../anchor/types/sure_pool';
import { Common } from './commont';
import { SURE_POOL_MANAGER_SEED } from './seeds';
import { token } from '@project-serum/anchor/dist/cjs/utils';

export class Pool extends Common {
	constructor(
		readonly program: anchor.Program<SurePool>,
		readonly connection: Connection,
		readonly wallet: anchor.Wallet
	) {
		super(program, connection, wallet);
	}

	async getPoolManager() {
		const [managerPDA, _] = await PublicKey.findProgramAddress(
			[SURE_POOL_MANAGER_SEED],
			this.program.programId
		);
		return managerPDA;
	}

	/**
	 * Get token pool statistics
	 *
	 * gather statistics on pool for a given mint
	 *
	 * @param pool
	 * @param tokenMint
	 *
	 * @returns TokenPoolStatistics: Struct containing information about pool for given token mint
	 */
	getTokenPoolStatistics = async (
		pool: PublicKey,
		tokenMint: PublicKey
	): Promise<TokenPoolStatistics> => {
		try {
			// find pool bitmap over
			const poolLiquidityTickBitmap = await this.getPoolLiquidityTickBitmapPDA(
				pool,
				tokenMint
			);
			let liquidityPositions;
			try {
				liquidityPositions = await this.program.account.bitMap.fetch(
					poolLiquidityTickBitmap
				);
			} catch (err) {
				throw new Error('could not get liquidity position bitmap. ' + err);
			}
			const bitmap = Bitmap.new(liquidityPositions);

			let tick = bitmap.getLowestTick();
			let highestTick = bitmap.getHighestTick();
			let liquidityInfo: LiquidityTickInfo;
			let tokenPoolStatistics: TokenPoolStatistics = {
				tokenMint: tokenMint,
				pool: pool,
				amountInsured: new anchor.BN(0),
				liquidity: new anchor.BN(0),
				premiumLow: tick,
				premiumHigh: highestTick,
			};
			while (tick !== -1) {
				liquidityInfo = await this.getLiquidityTickInfo(pool, tokenMint, tick);
				tokenPoolStatistics.amountInsured =
					tokenPoolStatistics.amountInsured.add(liquidityInfo.usedLiquidity);
				tokenPoolStatistics.liquidity = tokenPoolStatistics.liquidity.add(
					liquidityInfo.liquidity
				);
				tick = bitmap.getNextTick(tick);
			}
			return tokenPoolStatistics;
		} catch (err) {
			throw new Error(
				'sure-sdk.liquidity.getTokenPoolStatistics.error: ' + err
			);
		}
	};

	/**
	 * Get all the associated token acounts
	 * @param poolPDA
	 */
	async getAllPoolTokenAccounts(poolPDA: PublicKey): Promise<PublicKey[]> {
		const accountsByPool = await this.connection.getParsedTokenAccountsByOwner(
			poolPDA,
			{
				programId: TOKEN_PROGRAM_ID,
			}
		);

		return accountsByPool.value.map((account) => {
			return account.pubkey;
		});
	}

	/**
	 * Get Token Pools Information
	 * 	Get token pool information
	 *
	 * @returns PoolInformation array
	 */
	async getTokenPoolsInformation(): Promise<PoolInformation[]> {
		let pool: PoolAccount;
		let liquidityBitmapPDA: PublicKey;
		let liquidityBitmap: Bitmap;
		const tokenPools = await this.program.account.tokenPool.all();
		const pools = await this.program.account.poolAccount.all();
		const promises: Promise<PoolInformation>[] = [];
		for (const tokenPool of tokenPools) {
			promises.push(
				new Promise(async (resolve, reject) => {
					try {
						liquidityBitmapPDA = await this.getPoolLiquidityTickBitmapPDA(
							tokenPool.account.pool,
							tokenPool.account.tokenMint
						);

						liquidityBitmap = Bitmap.new(
							await this.program.account.bitMap.fetch(liquidityBitmapPDA)
						);

						pool = pools.filter(
							(pool) =>
								pool.publicKey.toBase58() == tokenPool.account.pool.toBase58()
						)[0].account;

						resolve({
							name: 'some name',
							tokenMint: tokenPool.account.tokenMint,
							insuranceFee: 0,
							smartContract: pool.smartContract,
							liquidity: await Money.convertBNFromDecimals(
								this.connection,
								tokenPool.account.liquidity,
								tokenPool.account.tokenMint
							),
							usedLiquidity: await Money.convertBNFromDecimals(
								this.connection,
								tokenPool.account.usedLiquidity,
								tokenPool.account.tokenMint
							),
							lowestPremium: liquidityBitmap.getLowestTick(),
							locked: false,
						});
					} catch (err) {
						console.log('could not get token pool');
						reject(err);
					}
				})
			);
		}
		return await Promise.all(promises);
	}

	async getOrCreatePool(
		smartContractAddress: PublicKey,
		insuranceFee: number,
		name: string
	): Promise<PublicKey> {
		const poolPDA = await this.getPoolPDA(smartContractAddress);
		try {
			await this.program.account.poolAccount.fetch(poolPDA);
		} catch (_) {
			await this.createPool(smartContractAddress, insuranceFee, name);
		}

		return poolPDA;
	}

	async getOrCreatePoolInstruction(
		pool: PublicKey,
		smartContractAddress: PublicKey,
		insuranceFee: number,
		name: string
	): Promise<TransactionInstruction | null> {
		try {
			await this.program.account.poolAccount.fetch(pool);
			return null;
		} catch (_) {
			return await this.createPoolInstruction(
				smartContractAddress,
				insuranceFee,
				name
			);
		}
	}

	async createPool(
		smartContractAddress: PublicKey,
		insuranceFee: number,
		name: string
	): Promise<string> {
		try {
			let tx = new anchor.web3.Transaction();

			tx.add(
				await this.createPoolInstruction(
					smartContractAddress,
					insuranceFee,
					name
				)
			);

			return await sendTransaction(this.connection, tx, this.wallet);
		} catch (err) {
			console.error(err);
			throw new Error(
				'sure-sdk.pool.createPool. Could not create pool. Cause: ' + err
			);
		}
	}

	async createPoolInstruction(
		smartContractAddress: PublicKey,
		insuranceFee: number,
		name: string
	): Promise<anchor.web3.TransactionInstruction> {
		const [protocolOwnerPDA, protocolOwnerBump] = await this.getProtocolOwner();
		const poolPDA = await this.getPoolPDA(smartContractAddress);
		const poolsPDA = await this.getPoolsPDA();

		return await this.program.methods
			.createPool(insuranceFee, name)
			.accounts({
				poolCreator: this.wallet.publicKey,
				protocolOwner: protocolOwnerPDA,
				pool: poolPDA,
				pools: poolsPDA,
				smartContract: smartContractAddress,
				rent: anchor.web3.SYSVAR_RENT_PUBKEY,
				systemProgram: SystemProgram.programId,
			})
			.instruction();
	}

	async initializeTokenPool(
		smartContractAddress: PublicKey,
		tokenMint: PublicKey,
		insuranceFee: number,
		name: string
	): Promise<string> {
		try {
			const pool = await this.getPoolPDA(smartContractAddress);
			const liquidityVaultPDA = await this.getPoolVaultPDA(pool, tokenMint);
			const premiumVaultPDA = await this.getPremiumVaultPDA(pool, tokenMint);
			const poolLiquidityTickBitmapPDA =
				await this.getPoolLiquidityTickBitmapPDA(pool, tokenMint);
			const tokenPoolPDA = await this.getTokenPoolPDA(pool, tokenMint);

			const tx = new anchor.web3.Transaction();
			// Check if pool exists
			const createPoolIx = await this.getOrCreatePoolInstruction(
				pool,
				smartContractAddress,
				insuranceFee,
				name
			);
			if (createPoolIx != null) {
				tx.add(createPoolIx);
			}

			tx.add(
				await this.program.methods
					.initializeTokenPool()
					.accounts({
						creator: this.wallet.publicKey,
						pool: pool,
						poolVaultTokenMint: tokenMint,
						poolVault: liquidityVaultPDA,
						premiumVault: premiumVaultPDA,
						poolLiquidityTickBitmap: poolLiquidityTickBitmapPDA,
						rent: anchor.web3.SYSVAR_RENT_PUBKEY,
						tokenPool: tokenPoolPDA,
						tokenProgram: TOKEN_PROGRAM_ID,
						systemProgram: SystemProgram.programId,
					})
					.instruction()
			);
			return await sendTransaction(this.connection, tx, this.wallet);
		} catch (err) {
			if (err.logs) {
				console.log(err.logs);
			}

			throw new Error('sure.pool.initializeTokenPool.error. ' + err + '\n');
		}
	}

	async initializePoolManager() {
		try {
			const poolManagerPDA = await this.getPoolManager();
			await this.program.methods
				.initializePoolManager()
				.accounts({
					initialManager: this.wallet.publicKey,
					manager: poolManagerPDA,
					systemProgram: SystemProgram.programId,
				})
				.rpc();
		} catch (err) {
			throw new Error('sure.pool.initializePoolManager.error. Cause: ' + err);
		}
	}
}
