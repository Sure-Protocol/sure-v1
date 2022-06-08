import * as anchor from '@project-serum/anchor';
import { getAccount, getMint, TOKEN_PROGRAM_ID } from '@solana/spl-token';
import { Connection, PublicKey, SystemProgram } from '@solana/web3.js';
import { LiquidityTickInfo, PoolAccount, TokenPoolStatistics } from 'src/types';
import { Bitmap, Money } from './../utils';
import { SurePool } from './../anchor/types/sure_pool';
import { Common } from './commont';
import { SURE_POOL_MANAGER_SEED } from './seeds';
import { liquidity } from '.';

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
	 * Get All Sure Pools
	 *
	 * Find all available pools based on vaults
	 *
	 * @returns Pool accounts
	 */
	async getPoolAccounts(): Promise<PoolAccount[]> {
		const surePools = await this.getPoolsPDA();
		const pools = await this.program.account.surePools.fetch(surePools);
		const poolAccounts: PoolAccount[] = [];
		let pool;
		let tokenAccountsPk: PublicKey[];
		let poolStatistics: TokenPoolStatistics;
		for (const poolPDA of pools.pools) {
			pool = await this.program.account.poolAccount.fetch(poolPDA);
			tokenAccountsPk = await this.getAllPoolTokenAccounts(poolPDA);
			for (const tokenAccountPk of tokenAccountsPk) {
				const tokenAccount = await getAccount(this.connection, tokenAccountPk);

				// Get the premium vault pda
				const premiumVaultPDA = await this.getPremiumVaultPDA(
					poolPDA,
					tokenAccount.mint
				);

				// We are only interested in the pool vaults
				if (premiumVaultPDA.toBase58() === tokenAccountPk.toBase58()) {
					poolStatistics = await this.getTokenPoolStatistics(
						poolPDA,
						tokenAccount.mint
					);

					poolAccounts.push({
						name: pool.name,
						tokenMint: poolStatistics.tokenMint,
						insuranceFee: pool.insuranceFee,
						liquidity: await this.convertBNFromDecimals(
							poolStatistics.liquidity,
							tokenAccount
						),
						usedLiquidity: await this.convertBNFromDecimals(
							poolStatistics.amountInsured,
							tokenAccount
						),
						premiumRate: poolStatistics.premiumLow,
						smartContract: pool.smartContract,
						locked: pool.locked,
					});
				}
			}
		}

		return poolAccounts;
	}

	async getOrCreatePool(
		smartContractAddress: PublicKey,
		insuranceFee: number,
		name: string
	) {
		const poolPDA = await this.getPoolPDA(smartContractAddress);
		try {
			await this.program.account.poolAccount.fetch(poolPDA);
		} catch (_) {
			await this.createPool(smartContractAddress, insuranceFee, name);
		}

		return poolPDA;
	}
	async createPool(
		smartContractAddress: PublicKey,
		insuranceFee: number,
		name: string
	) {
		const [protocolOwnerPDA, protocolOwnerBump] = await this.getProtocolOwner();
		const poolPDA = await this.getPoolPDA(smartContractAddress);
		const poolsPDA = await this.getPoolsPDA();

		try {
			await this.program.methods
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
				.rpc();
		} catch (err) {
			throw new Error(
				'addr: ' +
					this.program.programId +
					' sure.pool.createPool.error. Cause: ' +
					err
			);
		}
	}

	async createPoolVault(tokenMint: PublicKey, smartContractAddress: PublicKey) {
		const poolPDA = await this.getPoolPDA(smartContractAddress);
		const liquidityVaultPDA = await this.getPoolVaultPDA(poolPDA, tokenMint);
		const premiumVaultPDA = await this.getPremiumVaultPDA(poolPDA, tokenMint);
		const poolLiquidityTickBitmapPDA = await this.getPoolLiquidityTickBitmapPDA(
			poolPDA,
			tokenMint
		);

		try {
			await this.program.methods
				.createPoolVaults()
				.accounts({
					creator: this.wallet.publicKey,
					pool: poolPDA,
					poolVaultTokenMint: tokenMint,
					poolVault: liquidityVaultPDA,
					premiumVault: premiumVaultPDA,
					poolLiquidityTickBitmap: poolLiquidityTickBitmapPDA,
					rent: anchor.web3.SYSVAR_RENT_PUBKEY,
					tokenProgram: TOKEN_PROGRAM_ID,
					systemProgram: SystemProgram.programId,
				})
				.rpc();
		} catch (err) {
			if (err.logs) {
				console.log(err.logs);
			}

			throw new Error('sure.pool.createPoolVaults.error. Cause: ' + err);
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
