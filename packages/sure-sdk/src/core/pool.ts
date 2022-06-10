import * as anchor from '@project-serum/anchor';
import { getAccount, getMint, TOKEN_PROGRAM_ID } from '@solana/spl-token';
import { Connection, PublicKey, SystemProgram } from '@solana/web3.js';
import {
	LiquidityTickInfo,
	PoolAccount,
	PoolInformation,
	TokenPool,
	TokenPoolStatistics,
} from 'src/types';
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
	 * DEPRECATE: too slow
	 *
	 * @returns Pool accounts
	 */
	async getPoolsInformation(): Promise<PoolInformation[]> {
		const surePools = await this.getPoolsPDA();
		const pools = await this.program.account.surePools.fetch(surePools);
		const poolsInformation: PoolInformation[] = [];
		let pool: PoolAccount;
		let tokenPool: TokenPool;
		let liquidityBitmapPDA: PublicKey;
		let liquidityBitmap: Bitmap;
		for (const poolPDA of pools.pools) {
			console.log('> poolPDA: ', poolPDA);
			pool = await this.program.account.poolAccount.fetch(poolPDA);
			console.log('pool: ', pool);
			for (const tokenPoolPDA of pool.tokenPools) {
				console.log('> tokenPoolPDA: ', tokenPoolPDA);
				tokenPool = await this.program.account.tokenPool.fetch(tokenPoolPDA);
				console.log('> tokenPool: ', tokenPool);
				// Get liquidity bitmap
				liquidityBitmapPDA = await this.getPoolLiquidityTickBitmapPDA(
					poolPDA,
					tokenPool.tokenMint
				);

				liquidityBitmap = Bitmap.new(
					await this.program.account.bitMap.fetch(liquidityBitmapPDA)
				);
				console.log('liquidityBitmap: ', liquidityBitmap.getLowestTick());
				console.log('> tokenMint: ', tokenPool.tokenMint.toBase58());
				try {
					poolsInformation.push({
						name: pool.name,
						tokenMint: tokenPool.tokenMint,
						insuranceFee: pool.insuranceFee,
						smartContract: pool.smartContract,
						liquidity: await this.convertBNFromDecimals(
							tokenPool.liquidity,
							tokenPool.tokenMint
						),
						usedLiquidity: await this.convertBNFromDecimals(
							tokenPool.usedLiquidity,
							tokenPool.tokenMint
						),
						lowestPremium: liquidityBitmap.getLowestTick(),
						locked: pool.locked,
					});
				} catch {
					console.log('could not fetch pool');
				}
			}
		}

		return poolsInformation;
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

	async initializeTokenPool(pool: PublicKey, tokenMint: PublicKey) {
		console.log('Initialize token pool: ');
		const liquidityVaultPDA = await this.getPoolVaultPDA(pool, tokenMint);
		const premiumVaultPDA = await this.getPremiumVaultPDA(pool, tokenMint);
		const poolLiquidityTickBitmapPDA = await this.getPoolLiquidityTickBitmapPDA(
			pool,
			tokenMint
		);
		const tokenPoolPDA = await this.getTokenPoolPDA(pool, tokenMint);

		console.log('> liquidityVaultPDA: ', liquidityVaultPDA.toBase58());
		console.log('> premiumVaultPDA: ', premiumVaultPDA.toBase58());
		console.log(
			'> poolLiquidityTickBitmapPDA: ',
			poolLiquidityTickBitmapPDA.toBase58()
		);
		console.log('tokenPoolPDA: ', tokenPoolPDA.toBase58());
		try {
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
				.rpc();
			console.log('> success');
		} catch (err) {
			if (err.logs) {
				console.log(err.logs);
			}

			throw new Error('sure.pool.initializeTokenPool.error. Cause: ' + err);
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
