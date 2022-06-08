import * as anchor from '@project-serum/anchor';

import { Connection, PublicKey, SystemProgram } from '@solana/web3.js';
import {
	SURE_PREMIUM_POOL_SEED,
	SURE_INSURANCE_CONTRACTS_BITMAP,
	SURE_INSURANCE_CONTRACTS_INFO,
	SURE_INSURANCE_CONTRACT,
	SURE_INSURANCE_CONTRACTS,
} from './seeds';

import {
	getAccount,
	getMint,
	getOrCreateAssociatedTokenAccount,
	TOKEN_PROGRAM_ID,
} from '@solana/spl-token';
import NodeWallet from '@project-serum/anchor/dist/cjs/nodewallet';
import { BN, min } from 'bn.js';

import { SurePool } from './../anchor/types/sure_pool';
import { Common } from './commont';
import { InsuranceContractsInfo, PoolInsuranceContract } from 'src/types';
import { Money, Bitmap } from './../utils';

export class Insurance extends Common {
	constructor(
		readonly program: anchor.Program<SurePool>,
		readonly connection: Connection,
		readonly wallet: anchor.Wallet
	) {
		super(program, connection, wallet);
	}

	/**
	 * Get the Insurance Contract PDA
	 * The insurance contract is per user, per tick
	 *
	 * @param tickAccount     The tick account
	 */
	public async getInsuranceTickContractPDA(
		liquidityTickInfo: PublicKey
	): Promise<PublicKey> {
		const [insuranceTickContractPDA, insuranceTickContractBump] =
			await PublicKey.findProgramAddress(
				[
					SURE_INSURANCE_CONTRACT,
					this.wallet.publicKey.toBytes(),
					liquidityTickInfo.toBytes(),
				],
				this.program.programId
			);
		return insuranceTickContractPDA;
	}
	/**
	 * Get the Insurance Contracts bitmap PDA
	 * The insurance contracts held for each user per pool is represented
	 * as a bitmap [..].
	 *
	 * @param owner    The owner of the insurance contracts
	 * @param pool 		The user that owns the insurance contract
	 * @param tokenMint The mint of the token used in the pool
	 */
	async getPoolInsuranceContractBitmapPDA(
		pool: PublicKey,
		tokenMint: PublicKey
	): Promise<PublicKey> {
		const [poolInsuranceContractBitmapPDA, poolInsuranceContractBitmapBump] =
			await PublicKey.findProgramAddress(
				[
					SURE_INSURANCE_CONTRACTS_BITMAP,
					this.wallet.publicKey.toBytes(),
					pool.toBytes(),
					tokenMint.toBytes(),
				],
				this.program.programId
			);
		return poolInsuranceContractBitmapPDA;
	}

	async getPoolInsuranceContractInfoPDA(
		pool: PublicKey,
		tokenMint: PublicKey
	): Promise<PublicKey> {
		const [poolInsuranceContractInfoPDA, poolInsuranceContractInfoBump] =
			await PublicKey.findProgramAddress(
				[
					SURE_INSURANCE_CONTRACTS_INFO,
					this.wallet.publicKey.toBytes(),
					pool.toBytes(),
					tokenMint.toBytes(),
				],
				this.program.programId
			);
		return poolInsuranceContractInfoPDA;
	}

	async getPoolInsuranceContractInfo(
		pool: PublicKey,
		tokenMint: PublicKey
	): Promise<PoolInsuranceContract> {
		try {
			const poolInsuranceContractPDA =
				await this.getPoolInsuranceContractInfoPDA(pool, tokenMint);
			const poolInsuranceContract =
				await this.program.account.poolInsuranceContract.fetch(
					poolInsuranceContractPDA
				);
			return poolInsuranceContract;
		} catch (err) {
			throw new Error(
				'sure-sdk.insurance.getPoolInsuranceContractInfo.error. Cause: ' + err
			);
		}
	}

	/**
	 * Get Insurance Contracts PDA
	 *
	 * Method gets the PDA for the insurance contract held by
	 * a user/policy holder
	 */
	async getInsuranceContractsPDA(): Promise<PublicKey> {
		const [insuranceContractsPDA, insuranceContractsBump] =
			await PublicKey.findProgramAddress(
				[SURE_INSURANCE_CONTRACTS, this.wallet.publicKey.toBytes()],
				this.program.programId
			);
		return insuranceContractsPDA;
	}

	/**
	 * Create a New Policy Holder
	 * The insurance contract holds information about the positions
	 * in a given pool
	 *
	 * @param pool: The pool to buy insurance from. Ex: Ray - USDC
	 * @param tokenMint: The mint for the token used in the pool
	 *
	 */
	async createPolicyHolder(): Promise<PublicKey> {
		try {
			const insuranceContractsPDA = await this.getInsuranceContractsPDA();
			await this.program.methods
				.initializePolicyHolder()
				.accounts({
					signer: this.wallet.publicKey,
					insuranceContracts: insuranceContractsPDA,
					systemProgram: SystemProgram.programId,
				})
				.rpc();
			return insuranceContractsPDA;
		} catch (err) {
			throw new Error(
				'sure-sdk.insurance.createPolicyHolder.error. Cause: ' + err
			);
		}
	}

	/**
	 * Create insurance contract for given POOL
	 * The insurance contract holds information about the positions
	 * in a given pool
	 *
	 * @param pool: The pool to buy insurance from. Ex: Ray - USDC
	 * @param tokenMint: The mint for the token used in the pool
	 *
	 */
	async createUserPoolInsuranceContract(
		pool: PublicKey,
		tokenMint: PublicKey
	): Promise<
		[
			poolInsuranceContractInfo: PublicKey,
			poolInsuranceContractBitmap: PublicKey
		]
	> {
		try {
			const poolInsuranceContractBitmapPDA =
				await this.getPoolInsuranceContractBitmapPDA(pool, tokenMint);
			const poolInsuranceContractInfoPDA =
				await this.getPoolInsuranceContractInfoPDA(pool, tokenMint);
			const insuranceContracts = await this.getInsuranceContractsPDA();
			await this.program.methods
				.initializeUserPoolInsuranceContract()
				.accounts({
					signer: this.wallet.publicKey,
					pool: pool,
					tokenMint: tokenMint,
					insuranceContracts: insuranceContracts,
					poolInsuranceContractBitmap: poolInsuranceContractBitmapPDA,
					poolInsuranceContractInfo: poolInsuranceContractInfoPDA,
					systemProgram: SystemProgram.programId,
				})
				.rpc();

			return [poolInsuranceContractInfoPDA, poolInsuranceContractBitmapPDA];
		} catch (err) {
			throw new Error(
				'Could not initialize Pool Insurance Contracts. Cause: ' + err
			);
		}
	}

	/**
	 * Create insurance contract for given tick
	 * The insurance contract holds information about
	 *
	 * @param owner<publickey>: Owner of insurance contract
	 * @param tickAccount<publickey>: The tick to buy insurance from
	 * @param pool: The pool to buy insurance from. Ex: Ray - USDC
	 * @param tokenMint: The mint for the token used in the pool
	 *
	 */
	createInsuranceContractForTick = async (
		pool: PublicKey,
		tokenMint: PublicKey,
		liquidityTickInfo: PublicKey
	): Promise<PublicKey> => {
		const poolInsuranceContractInfoPDA =
			await this.getPoolInsuranceContractInfoPDA(pool, tokenMint);
		const poolInsuranceContractBitmapPDA =
			await this.getPoolInsuranceContractBitmapPDA(pool, tokenMint);

		// Get insurance contract with pool
		const insuranceTickContractPDA = await this.getInsuranceTickContractPDA(
			liquidityTickInfo
		);

		try {
			await this.program.methods
				.initializeInsuranceContract()
				.accounts({
					owner: this.wallet.publicKey,
					pool: pool,
					tokenMint: tokenMint,
					liquidityTickInfo: liquidityTickInfo,
					insuranceTickContract: insuranceTickContractPDA,
					poolInsuranceContractInfo: poolInsuranceContractInfoPDA,
					poolInsuranceContractBitmap: poolInsuranceContractBitmapPDA,
					systemProgram: SystemProgram.programId,
				})
				.rpc();

			await this.program.account.insuranceTickContract.fetch(
				insuranceTickContractPDA
			);
		} catch (err) {
			throw new Error('could not create insurance contract. Cause: ' + err);
		}

		return insuranceTickContractPDA;
	};

	/**
	 * Helper function to estimate the yearly premium
	 * and the amount that can be insured
	 *
	 * @param amount     The amount to be insured
	 * @param tokenMint The mint of the current pool. Ex. USDC
	 * @param pool The pool to buy insurance from
	 */
	async estimateYearlyPremium(
		amount: number,
		tokenMint: PublicKey,
		pool: PublicKey
	): Promise<[amountCovered: anchor.BN, insurancePrice: anchor.BN]> {
		try {
			let poolAccount;
			try {
				poolAccount = await this.program.account.poolAccount.fetch(pool);
			} catch (err) {
				throw new Error("couldn't get pool account. " + err);
			}

			const insuranceFee = poolAccount.insuranceFee;
			const tokenDecimals = (await getMint(this.connection, tokenMint))
				.decimals;
			const amountInDecimals = new Money(
				tokenDecimals,
				amount
			).convertToDecimals();

			/// Estimate premium
			let bitmapPDA = await this.getPoolLiquidityTickBitmapPDA(pool, tokenMint);
			let liquidityPositions;
			try {
				liquidityPositions = await this.program.account.bitMap.fetch(bitmapPDA);
			} catch (err) {
				throw new Error('could not get liquidity position bitmap. ' + err);
			}

			const bitmap = Bitmap.new(liquidityPositions);

			// Check if there is enough
			let totalPremium = new anchor.BN(0);
			let amountToPay = new anchor.BN(0);
			let amountToCover = new anchor.BN(amountInDecimals);
			let amountCovered = new anchor.BN(0);
			let tick = bitmap.getLowestTick();
			if (tick === -1) {
				throw new Error('no available liquidity');
			}

			// Get tick account
			let tickAccountPDA = await this.getLiquidityTickInfoPDA(
				pool,
				tokenMint,
				tick
			);
			let tickAccount;
			try {
				tickAccount = await this.program.account.tick.fetch(tickAccountPDA);
			} catch (err) {
				throw new Error('could not get liquidity tick account. ' + err);
			}

			let availableLiquidity = tickAccount.liquidity.sub(
				tickAccount.usedLiquidity
			);

			while (amountToCover.gt(new anchor.BN(0)) && tick !== -1) {
				if (availableLiquidity.gte(new anchor.BN(amountToCover))) {
					// Enough liquidity for one tick
					amountToPay = amountToCover;
				} else {
					// Buy all the liquidity for one tick
					amountToPay = availableLiquidity;
				}

				// Buy insurance for tick
				totalPremium = totalPremium.add(amountToPay.muln(tick / 10000));

				amountToCover = amountToCover.sub(amountToPay);

				// find next liquidity
				tick = bitmap.getNextTick(tick);
				tickAccountPDA = await this.getLiquidityTickInfoPDA(
					pool,
					tokenMint,
					tick
				);
				tickAccount = await this.program.account.tick.fetch(tickAccountPDA);
				availableLiquidity = tickAccount.liquidity.sub(
					tickAccount.usedLiquidity
				);

				amountCovered = amountCovered.add(amountToPay);
			}

			// Add insurance fee
			const sureFee = totalPremium.muln(insuranceFee / 10000);
			const insurancePrice = totalPremium.add(sureFee);
			return [amountCovered, insurancePrice];
		} catch (err) {
			throw new Error(
				'sure-sdk.insurance.estimateYearlyPremium.error. Cause: ' + err
			);
		}
	}

	/**
	 * Get or create insurance contract for given tick
	 * The insurance contract holds information about
	 *
	 * @param owner<publickey>: Owner of insurance contract
	 * @param tickAccount<publickey>: The tick to buy insurance from
	 *
	 */
	async getOrCreateInsuranceContractForTick(
		owner: PublicKey,
		pool: PublicKey,
		tokenMint: PublicKey,
		liquidityTickInfo: PublicKey
	): Promise<PublicKey> {
		const insuranceTickContractPDA = await this.getInsuranceTickContractPDA(
			liquidityTickInfo
		);

		try {
			const insuranceContract =
				await this.program.account.insuranceTickContract.getAccountInfo(
					insuranceTickContractPDA
				);
			if (insuranceContract !== null) {
				return insuranceTickContractPDA;
			}
			throw new Error(
				'insurance contract does not exist: ' + insuranceContract
			);
		} catch (_) {
			// Insurance contract does not exist. Create it
			await this.createInsuranceContractForTick(
				pool,
				tokenMint,
				liquidityTickInfo
			);
		}

		return insuranceTickContractPDA;
	}

	/**
	 * Calculate the amount insured by user
	 *
	 * @param owner<publickey>: Owner of insurance contract
	 * @param tokenMint<publickey>: the mint account publickkey
	 * @param pool<PublicKey>: the pool to buy insurance from
	 *
	 * @returns Promise for a Big Number - BN
	 */
	async getInsuredAmount(
		pool: PublicKey,
		tokenMint: PublicKey
	): Promise<anchor.BN> {
		try {
			const userPoolInsuranceContractBitmapPDA =
				await this.getPoolInsuranceContractBitmapPDA(pool, tokenMint);

			let userPoolInsuranceContractBitmap;
			try {
				userPoolInsuranceContractBitmap =
					await this.program.account.bitMap.fetch(
						userPoolInsuranceContractBitmapPDA
					);
			} catch (err) {
				throw new Error(' insurance contracts:' + err);
			}
			// Create insurance contract bitmap
			const insuranceContractBitmap = Bitmap.new(
				userPoolInsuranceContractBitmap
			);

			// Start from right and reduce position
			let currentTick = insuranceContractBitmap.getHighestTick();
			let amount = new anchor.BN(0);

			while (currentTick !== -1) {
				const tickAccountPDA = await this.getLiquidityTickInfoPDA(
					pool,
					tokenMint,
					currentTick
				);
				const insuranceContractForTickPDA =
					await this.getInsuranceTickContractPDA(tickAccountPDA);
				let insuranceContractForTick;
				try {
					insuranceContractForTick =
						await this.program.account.insuranceTickContract.fetch(
							insuranceContractForTickPDA
						);
				} catch (err) {
					throw new Error('InsuranceContract:' + err);
				}
				console.log(
					'> getInsuredAmount insuranceContractForTick: ',
					insuranceContractForTick
				);
				amount = amount.add(insuranceContractForTick.insuredAmount);
				currentTick = insuranceContractBitmap.getPreviousTick(currentTick);
			}
			return amount;
		} catch (err) {
			console.log('sure.insurance.getInsuredAmount.warning. Cause: ' + err);
			return new BN(0);
		}
	}

	/**
	 * Get Pool Insurance Contracts Info
	 *
	 * Get information about a user's insurance positions in a given
	 * pool for a given mint.
	 *
	 * @param pool<publickey>: Pool
	 * @param tokenMint<publickey>: mint of the deposited token
	 *
	 * @returns InsuranceContractsInfo
	 */
	async getPoolInsuranceContractsInfo(
		pool: PublicKey,
		tokenMint: PublicKey
	): Promise<InsuranceContractsInfo> {
		const insuranceContractsInfoPDA =
			await this.getPoolInsuranceContractInfoPDA(pool, tokenMint);
		const insuranceContractsInfo =
			await this.program.account.poolInsuranceContract.fetch(
				insuranceContractsInfoPDA
			);
		return {
			insuredAmount: insuranceContractsInfo.insuredAmount,
			expiryTs: insuranceContractsInfo.expiryTs,
		};
	}

	async getOrCreateUserPoolInsuranceContract(
		pool: PublicKey,
		tokenMint: PublicKey
	): Promise<
		[
			poolInsuranceContractInfo: PublicKey,
			poolInsuranceContractBitmap: PublicKey
		]
	> {
		const poolInsuranceContractBitmapPDA =
			await this.getPoolInsuranceContractBitmapPDA(pool, tokenMint);
		const poolInsuranceContractInfoPDA =
			await this.getPoolInsuranceContractInfoPDA(pool, tokenMint);
		try {
			const res = await this.program.account.bitMap.getAccountInfo(
				poolInsuranceContractBitmapPDA
			);
			const res2 =
				await this.program.account.poolInsuranceContract.getAccountInfo(
					poolInsuranceContractInfoPDA
				);
			if (res !== null || res !== res2) {
				return [poolInsuranceContractInfoPDA, poolInsuranceContractBitmapPDA];
			}
			throw new Error();
		} catch (_) {
			return this.createUserPoolInsuranceContract(pool, tokenMint);
		}
	}

	/**
	 * Buy Insurance from the Liquidity pool
	 * It's important that we can buy in as few transactions
	 * as possible
	 * This means we have to do optimistic buying of insurance i.e. find how many
	 * ticks needs to be filled and then prepare the correct instructions
	 *
	 * @param amount<number>: the amount of insurance to buy
	 * @param pool<publickey>: the pool to buy from
	 *
	 */
	async buyInsurance(
		pool: PublicKey,
		tokenMint: PublicKey,
		newInsuredAmount: number,
		endTimestamp: number
	) {
		try {
			const newInsuredAmountBN = new anchor.BN(newInsuredAmount);

			// Create insurance overview
			const insuredAmount = await this.getInsuredAmount(pool, tokenMint);
			let amountChange = new anchor.BN(0);
			// Check if amount is changed
			const [poolInsuranceContractInfoPDA, poolInsuranceContractBitmapPDA] =
				await this.getOrCreateUserPoolInsuranceContract(pool, tokenMint);

			if (newInsuredAmountBN.gte(insuredAmount)) {
				const tickAccountPDA = await this.getPoolLiquidityTickBitmapPDA(
					pool,
					tokenMint
				);
				const tickAccount = await this.program.account.bitMap.fetch(
					tickAccountPDA
				);
				const tickAccountBitmap = Bitmap.new(tickAccount);

				amountChange = newInsuredAmountBN.sub(insuredAmount);
				await this.increaseInsurancePosition(
					pool,
					tokenMint,
					amountChange,
					tickAccountBitmap,
					endTimestamp
				);
			} else {
				amountChange = insuredAmount.sub(newInsuredAmountBN);

				const poolInsuranceContract = await this.program.account.bitMap.fetch(
					poolInsuranceContractBitmapPDA
				);
				const poolInsuranceContractBitmap = Bitmap.new(poolInsuranceContract);
				await this.reduceInsurancePositon(
					pool,
					tokenMint,
					amountChange,
					poolInsuranceContractBitmap,
					endTimestamp
				);
			}
		} catch (err) {
			throw new Error('sure.insurance.buyInsurance.error. Cause: ' + err);
		}
	}

	/**
	 * Changes the end date / contract expiry for all the held contracts
	 *
	 * @param connection<Connection>: rpc connection
	 * @param endTimestamp<number>: the timestamp of the contract expiry
	 * @param pool<Pubkey>: the liquidity pool used to buy the insurance from
	 * @param tokenMint<Pubkey>: the mint of the token used to deposit liquidity
	 *
	 * @returns none
	 */
	async changeContractExpiry(
		pool: PublicKey,
		tokenMint: PublicKey,
		endTimestamp: number
	) {
		const [poolInsuranceContractInfoPDA, poolInsuranceContractBitmapPDA] =
			await this.getOrCreateUserPoolInsuranceContract(pool, tokenMint);

		const poolInsuranceContract = await this.program.account.bitMap.fetch(
			poolInsuranceContractBitmapPDA
		);
		const poolInsuranceContractBitmap = Bitmap.new(poolInsuranceContract);
		const tokenAccount = await getOrCreateAssociatedTokenAccount(
			this.connection,
			(this.wallet as NodeWallet).payer,
			tokenMint,
			this.wallet.publicKey
		);
		const premiumVaultPDA = await this.getPremiumVaultPDA(pool, tokenMint);

		// Step through all insurance positions and update the end data
		let currentTick = poolInsuranceContractBitmap.getLowestTick();
		let insuredAmountConst;
		let insuranceContractForTickPDA;
		let insuranceContract;
		let liquidityTickInfoPDA;
		let txs = new anchor.web3.Transaction();

		while (currentTick !== -1) {
			// Fetch insurance contract for current tick
			liquidityTickInfoPDA = await this.getLiquidityTickInfoPDA(
				pool,
				tokenMint,
				currentTick
			);
			insuranceContractForTickPDA =
				await this.getOrCreateInsuranceContractForTick(
					this.wallet.publicKey,
					pool,
					tokenMint,
					liquidityTickInfoPDA
				);
			insuranceContract =
				await this.program.account.insuranceTickContract.fetch(
					insuranceContractForTickPDA
				);
			insuredAmountConst = insuranceContract.insuredAmount;

			txs.add(
				await this.program.methods
					.updateInsuranceForTick(
						insuredAmountConst,
						new anchor.BN(endTimestamp)
					)
					.accounts({
						buyer: this.wallet.publicKey,
						pool: pool,
						liquidityTickInfo: liquidityTickInfoPDA,
						tokenAccount: tokenAccount.address,
						premiumVault: premiumVaultPDA,
						insuranceTickContract: insuranceContractForTickPDA,
						poolInsuranceContractInfo: poolInsuranceContractInfoPDA,
						tokenProgram: TOKEN_PROGRAM_ID,
						systemProgram: SystemProgram.programId,
					})
					.instruction()
			);

			currentTick = poolInsuranceContractBitmap.getNextTick(currentTick);
		}
		txs.recentBlockhash = (
			await this.connection.getLatestBlockhash()
		).blockhash;
		txs.feePayer = this.wallet.publicKey;

		try {
			const provider = await anchor.getProvider();
			await provider.send?.(txs);
		} catch (err) {
			console.log('logs?: ', err?.logs);
			throw new Error(
				'Sure.buyInsurance. Could not buy insurance. Cause: ' + err
			);
		}
	}

	async increaseInsurancePosition(
		pool: PublicKey,
		tokenMint: PublicKey,
		amountChange: anchor.BN,
		liquidityPositions: Bitmap,
		endTs: number
	) {
		// Start from left and increase position
		let currentTick = liquidityPositions.getLowestTick();

		const tokenAccount = await getOrCreateAssociatedTokenAccount(
			this.connection,
			(this.wallet as NodeWallet).payer,
			tokenMint,
			this.wallet.publicKey
		);
		const premiumVaultPDA = await this.getPremiumVaultPDA(pool, tokenMint);

		const poolInsuranceContractInfoPDA =
			await this.getPoolInsuranceContractInfoPDA(pool, tokenMint);

		// Create Anchor Transaction
		let tx = new anchor.web3.Transaction();
		// Initialize parameters
		let liquidityTickInfo;
		let liquidityTickInfoPDA;
		let availableLiquidity;
		let insuranceContractForTickPDA;
		let insuranceContract;
		let amountToBuyForTick;
		let insureAmountForTick = new anchor.BN(0);

		// Reduce position tick for tick
		while (amountChange.gt(new anchor.BN(0)) && currentTick !== -1) {
			console.log('> amountChange: ', amountChange.toString());
			console.log('> current_tick: ', currentTick);

			// Get tick account
			liquidityTickInfoPDA = await this.getLiquidityTickInfoPDA(
				pool,
				tokenMint,
				currentTick
			);
			liquidityTickInfo = await this.program.account.tick.fetch(
				liquidityTickInfoPDA
			);
			insuranceContractForTickPDA =
				await this.getOrCreateInsuranceContractForTick(
					this.wallet.publicKey,
					pool,
					tokenMint,
					liquidityTickInfoPDA
				);

			insuranceContract =
				await this.program.account.insuranceTickContract.fetch(
					insuranceContractForTickPDA
				);

			availableLiquidity = liquidityTickInfo.liquidity.sub(
				liquidityTickInfo.usedLiquidity
			);
			amountToBuyForTick = min(availableLiquidity, amountChange);
			insureAmountForTick =
				insuranceContract.insuredAmount.add(amountToBuyForTick);
			console.log(
				'> increaseInsurancePosition.insureAmountForTick: ',
				insureAmountForTick.toString()
			);
			tx.add(
				await this.program.methods
					.updateInsuranceForTick(insureAmountForTick, new anchor.BN(endTs))
					.accounts({
						buyer: this.wallet.publicKey,
						pool: pool,
						liquidityTickInfo: liquidityTickInfoPDA,
						tokenAccount: tokenAccount.address,
						premiumVault: premiumVaultPDA,
						insuranceTickContract: insuranceContractForTickPDA,
						poolInsuranceContractInfo: poolInsuranceContractInfoPDA,
						tokenProgram: TOKEN_PROGRAM_ID,
						systemProgram: SystemProgram.programId,
					})
					.instruction()
			);

			amountChange = amountChange.sub(amountToBuyForTick);

			// Get the previous tick in the bitmap
			currentTick = liquidityPositions.getNextTick(currentTick);
		}
		tx.recentBlockhash = (await this.connection.getLatestBlockhash()).blockhash;
		tx.feePayer = this.wallet.publicKey;

		try {
			await this.program.provider.sendAndConfirm?.(tx);
		} catch (err) {
			console.log('logs?: ', err?.logs);
			throw new Error(
				'Sure.buyInsurance. Could not buy insurance. Cause: ' + err
			);
		}
	}

	async reduceInsurancePositon(
		pool: PublicKey,
		tokenMint: PublicKey,
		amountChange: anchor.BN,
		bitmap: Bitmap,
		endTs: number
	) {
		// Start from right and reduce position
		let currentTick = bitmap.getHighestTick();
		const tokenAccount = await getOrCreateAssociatedTokenAccount(
			this.connection,
			(this.wallet as NodeWallet).payer,
			tokenMint,
			this.wallet.publicKey
		);
		const premiumVaultPDA = await this.getPremiumVaultPDA(pool, tokenMint);

		// Create Anchor Transaction
		let txs = new anchor.web3.Transaction();
		// Initialize parameters
		let liquidityTickInfoPDA;
		let insuranceTickContractPDA;
		let insuranceTickContract;
		let amountToReduceForTick;
		let amountToInsureForTick = new anchor.BN(0);

		// Reduce position tick for tick
		while (amountChange.gt(new anchor.BN(0))) {
			liquidityTickInfoPDA = await this.getLiquidityTickInfoPDA(
				pool,
				tokenMint,
				currentTick
			);
			insuranceTickContractPDA = await this.getInsuranceTickContractPDA(
				liquidityTickInfoPDA
			);
			insuranceTickContract =
				await this.program.account.insuranceTickContract.fetch(
					insuranceTickContractPDA
				);
			amountToReduceForTick = min(
				insuranceTickContract.insuredAmount,
				amountChange
			);
			amountToInsureForTick = insuranceTickContract.insuredAmount.sub(
				amountToReduceForTick
			);

			txs.add(
				await this.program.methods
					.updateInsuranceForTick(amountToInsureForTick, new anchor.BN(endTs))
					.accounts({
						buyer: this.wallet.publicKey,
						pool: pool,
						liquidityTickInfo: liquidityTickInfoPDA,
						tokenAccount: tokenAccount.address,
						premiumVault: premiumVaultPDA,
						insuranceTickContract: insuranceTickContractPDA,
						tokenProgram: TOKEN_PROGRAM_ID,
						systemProgram: SystemProgram.programId,
					})
					.instruction()
			);

			amountChange = amountChange.sub(amountToReduceForTick);
			// Get the previous tick in the bitmap
			currentTick = bitmap.getPreviousTick(currentTick);
		}
		txs.recentBlockhash = (
			await this.connection.getLatestBlockhash()
		).blockhash;
		txs.feePayer = this.wallet.publicKey;

		try {
			const provider = await anchor.getProvider();
			await provider.send?.(txs);
		} catch (err) {
			console.log('logs?: ', err?.logs);
			throw new Error(
				'Sure.buyInsurance. Could not buy insurance. Cause: ' + err
			);
		}
	}
}
