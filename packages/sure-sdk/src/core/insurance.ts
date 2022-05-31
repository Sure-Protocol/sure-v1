import * as anchor from '@project-serum/anchor';

import { Connection, PublicKey } from '@solana/web3.js';
import {
	SURE_PREMIUM_POOL_SEED,
	SURE_INSURANCE_CONTRACT,
	SURE_INSURANCE_CONTRACTS,
} from './seeds';

import { Bitmap } from '../utils/bitmap';
import {
	getOrCreateAssociatedTokenAccount,
	TOKEN_PROGRAM_ID,
} from '@solana/spl-token';
import NodeWallet from '@project-serum/anchor/dist/cjs/nodewallet';
import { min } from 'bn.js';

import { SurePool } from './../anchor/types/sure_pool';
import { Common } from './commont';

export class Insurance extends Common {
	constructor(
		readonly program: anchor.Program<SurePool>,
		readonly connection: Connection,
		readonly wallet: anchor.Wallet
	) {
		super(program, connection, wallet);
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

	/**
	 * Get the Insurance Contract PDA
	 * The insurance contract is per user, per tick
	 *
	 * @param tickAccount     The tick account
	 * @param owner The user that owns the insurance contract
	 */
	public async getInsuranceContractPDA(
		tickAccount: PublicKey
	): Promise<PublicKey> {
		const [insuranceContractPDA, insuranceContractBump] =
			await PublicKey.findProgramAddress(
				[
					SURE_INSURANCE_CONTRACT,
					this.wallet.publicKey.toBytes(),
					tickAccount.toBytes(),
				],
				this.program.programId
			);

		return insuranceContractPDA;
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
	async getInsuranceContractsBitmapPDA(
		pool: PublicKey,
		tokenMint: PublicKey
	): Promise<PublicKey> {
		const [insuranceContractsPDA, insuranceContractsBump] =
			await PublicKey.findProgramAddress(
				[
					SURE_INSURANCE_CONTRACTS,
					this.wallet.publicKey.toBytes(),
					pool.toBytes(),
					tokenMint.toBytes(),
				],
				this.program.programId
			);
		return insuranceContractsPDA;
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
		owner: PublicKey,
		pool: PublicKey,
		tokenMint: PublicKey,
		tickAccount: PublicKey
	): Promise<PublicKey> => {
		const insuranceContractsBitmapPDA =
			await this.getInsuranceContractsBitmapPDA(pool, tokenMint);
		// Get insurance contract with pool
		const insuranceContractPDA = await this.getInsuranceContractPDA(
			tickAccount
		);

		try {
			await this.program.methods
				.initializeInsuranceContract()
				.accounts({
					owner: owner,
					pool: pool,
					tokenMint: tokenMint,
					tickAccount: tickAccount,
					insuranceContract: insuranceContractPDA,
					insuranceContracts: insuranceContractsBitmapPDA,
					systemProgram: this.program.programId,
				})
				.rpc();

			await this.program.account.insuranceContract.fetch(insuranceContractPDA);
		} catch (err) {
			throw new Error('could not create insurance contract. Cause: ' + err);
		}

		return insuranceContractPDA;
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
		const poolAccount = await this.program.account.poolAccount.fetch(pool);
		const insuranceFee = poolAccount.insuranceFee;

		/// Estimate premium
		let bitmapPDA = await this.getLiquidityPositionBitmapPDA(pool, tokenMint);
		const liquidityPositions = await this.program.account.bitMap.fetch(
			bitmapPDA
		);
		const bitmap = Bitmap.new(liquidityPositions);

		// Check if there is enough
		let totalPremium = new anchor.BN(0);
		let amountToPay = new anchor.BN(0);
		let amountToCover = new anchor.BN(amount);
		let amountCovered = new anchor.BN(0);
		let tick = bitmap.getLowestTick();

		// Get tick account
		let tickAccountPDA = await this.getTickAccountPDA(pool, tokenMint, tick);
		let tickAccount = await this.program.account.tick.fetch(tickAccountPDA);
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

			bitmapPDA = await this.getLiquidityPositionBitmapPDA(pool, tokenMint);
			tick = bitmap.getNextTick(tick);
			tickAccountPDA = await this.getTickAccountPDA(pool, tokenMint, tick);
			tickAccount = await this.program.account.tick.fetch(tickAccountPDA);
			availableLiquidity = tickAccount.liquidity.sub(tickAccount.usedLiquidity);

			amountCovered = amountCovered.add(amountToPay);
		}

		// Add insurance fee
		const sureFee = totalPremium.muln(insuranceFee / 10000);
		const insurancePrice = totalPremium.add(sureFee);
		return [amountCovered, insurancePrice];
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
		tickAccount: PublicKey
	): Promise<PublicKey> {
		const insuranceContractPDA = await this.getInsuranceContractPDA(
			tickAccount
		);

		try {
			const insuranceContract =
				await this.program.account.insuranceContract.getAccountInfo(
					insuranceContractPDA
				);
			if (insuranceContract !== null) {
				return insuranceContractPDA;
			}
			throw new Error();
		} catch (_) {
			// Insurance contract does not exist. Create it
			await this.createInsuranceContractForTick(
				owner,
				pool,
				tokenMint,
				tickAccount
			);
		}

		return insuranceContractPDA;
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
		const userInsuranceContractsPDA = await this.getInsuranceContractsBitmapPDA(
			pool,
			tokenMint
		);
		try {
			const userInsuranceContracts = await this.program.account.bitMap.fetch(
				userInsuranceContractsPDA
			);
			// Create insurance contract bitmap
			const insuranceContractBitmap = Bitmap.new(userInsuranceContracts);

			// Start from right and reduce position
			let currentTick = insuranceContractBitmap.getHighestTick();
			let amount = new anchor.BN(0);

			while (currentTick !== -1) {
				const tickAccountPDA = await this.getTickAccountPDA(
					pool,
					tokenMint,
					currentTick
				);
				const insuranceContractForTickPDA = await this.getInsuranceContractPDA(
					tickAccountPDA
				);
				const insuranceContractForTick =
					await this.program.account.insuranceContract.fetch(
						insuranceContractForTickPDA
					);
				amount = amount.add(insuranceContractForTick.insuredAmount);
				currentTick = insuranceContractBitmap.getPreviousTick(currentTick);
			}
			return amount;
		} catch (err) {
			console.log('Could not calculate insured amount. cause: ', err);
			return new anchor.BN(0);
		}
	}

	async createUserInsuranceContracts(
		pool: PublicKey,
		tokenMint: PublicKey
	): Promise<PublicKey> {
		try {
			const insuranceContractPDA = await this.getInsuranceContractsBitmapPDA(
				pool,
				tokenMint
			);
			await this.program.methods
				.initializeUserInsuranceContracts()
				.accounts({
					signer: this.wallet.publicKey,
					pool: pool,
					insuranceContracts: insuranceContractPDA,
					systemProgram: this.program.programId,
				})
				.rpc();

			return insuranceContractPDA;
		} catch (err) {
			throw new Error(
				'Could not initialize Insurance Contracts. Cause: ' + err
			);
		}
	}

	async getOrCreateUserInsuranceContracts(
		pool: PublicKey,
		tokenMint: PublicKey
	): Promise<PublicKey> {
		const insuranceContractPDA = await this.getInsuranceContractsBitmapPDA(
			pool,
			tokenMint
		);
		try {
			const res = await this.program.account.bitMap.getAccountInfo(
				insuranceContractPDA
			);
			if (res !== null) {
				return insuranceContractPDA;
			}
			throw new Error();
		} catch (_) {
			return this.createUserInsuranceContracts(pool, tokenMint);
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
		console.log('// Buy insurance ');
		const newInsuredAmountBN = new anchor.BN(newInsuredAmount);

		// Create insurance overview
		const insuredAmount = await this.getInsuredAmount(pool, tokenMint);
		let amountChange = new anchor.BN(0);
		// Check if amount is changed
		if (newInsuredAmountBN.gte(insuredAmount)) {
			const insuranceContractsPDA =
				await this.getOrCreateUserInsuranceContracts(pool, tokenMint);
			const liquidityPositionsPDA = await this.getLiquidityPositionBitmapPDA(
				pool,
				tokenMint
			);
			const liquidityPositions = await this.program.account.bitMap.fetch(
				liquidityPositionsPDA
			);
			const liquidityPositionsBitmap = Bitmap.new(liquidityPositions);

			amountChange = newInsuredAmountBN.sub(insuredAmount);
			await this.increaseInsurancePosition(
				pool,
				tokenMint,
				amountChange,
				liquidityPositionsBitmap,
				endTimestamp
			);
		} else {
			const insuranceContractsPDA =
				await this.getOrCreateUserInsuranceContracts(pool, tokenMint);
			const insuranceContracts = await this.program.account.bitMap.fetch(
				insuranceContractsPDA
			);
			const insuranceContractsBitmap = Bitmap.new(insuranceContracts);

			amountChange = insuredAmount.sub(newInsuredAmountBN);
			await this.reduceInsurancePositon(
				pool,
				tokenMint,
				amountChange,
				insuranceContractsBitmap,
				endTimestamp
			);
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
		const insuranceContractsPDA = await this.getOrCreateUserInsuranceContracts(
			pool,
			tokenMint
		);
		const insuranceContracts = await this.program.account.bitMap.fetch(
			insuranceContractsPDA
		);
		const insuranceContractsBitmap = Bitmap.new(insuranceContracts);
		const tokenAccount = await getOrCreateAssociatedTokenAccount(
			this.connection,
			(this.wallet as NodeWallet).payer,
			tokenMint,
			this.wallet.publicKey
		);
		const premiumVaultPDA = await this.getPremiumVaultPDA(pool, tokenMint);

		// Step through all insurance positions and update the end data
		let currentTick = insuranceContractsBitmap.getLowestTick();
		let insuredAmountConst;
		let insuranceContractForTickPDA;
		let insuranceContract;
		let tickAccountPDA;
		let txs = new anchor.web3.Transaction();

		while (currentTick !== -1) {
			// Fetch insurance contract for current tick
			tickAccountPDA = await this.getTickAccountPDA(
				pool,
				tokenMint,
				currentTick
			);
			insuranceContractForTickPDA =
				await this.getOrCreateInsuranceContractForTick(
					this.wallet.publicKey,
					pool,
					tokenMint,
					tickAccountPDA
				);
			insuranceContract = await this.program.account.insuranceContract.fetch(
				insuranceContractForTickPDA
			);
			insuredAmountConst = insuranceContract.insuredAmount;

			txs.add(
				this.program.instruction.buyInsuranceForTick(
					insuredAmountConst,
					new anchor.BN(endTimestamp),
					{
						accounts: {
							buyer: this.wallet.publicKey,
							pool: pool,
							tickAccount: tickAccountPDA,
							tokenAccount: tokenAccount.address,
							premiumVault: premiumVaultPDA,
							insuranceContract: insuranceContractForTickPDA,
							tokenProgram: TOKEN_PROGRAM_ID,
							systemProgram: this.program.programId,
						},
					}
				)
			);

			currentTick = insuranceContractsBitmap.getNextTick(currentTick);
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
		bitmap: Bitmap,
		endTs: number
	) {
		console.log('// increaseInsurancePosition');
		// Start from left and increase position
		let currentTick = bitmap.getLowestTick();

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
		let tickAccount;
		let tickAccountPDA;
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
			tickAccountPDA = await tickAccount.getTickAccountPDA(
				pool,
				tokenMint,
				currentTick
			);
			tickAccount = await this.program.account.tick.fetch(tickAccountPDA);

			insuranceContractForTickPDA =
				await this.getOrCreateInsuranceContractForTick(
					this.wallet.publicKey,
					pool,
					tokenMint,
					tickAccountPDA
				);

			insuranceContract = await this.program.account.insuranceContract.fetch(
				insuranceContractForTickPDA
			);

			availableLiquidity = tickAccount.liquidity.sub(tickAccount.usedLiquidity);
			console.log('> availableLiquidity: ', availableLiquidity.toString());
			amountToBuyForTick = min(availableLiquidity, amountChange);
			insureAmountForTick =
				insuranceContract.insuredAmount.add(amountToBuyForTick);
			console.log('> amountToInsureForTick: ', insureAmountForTick.toString());
			txs.add(
				this.program.instruction.buyInsuranceForTick(
					insureAmountForTick,
					new anchor.BN(endTs),
					{
						accounts: {
							buyer: this.wallet.publicKey,
							pool: pool,
							tickAccount: tickAccountPDA,
							tokenAccount: tokenAccount.address,
							premiumVault: premiumVaultPDA,
							insuranceContract: insuranceContractForTickPDA,
							tokenProgram: TOKEN_PROGRAM_ID,
							systemProgram: this.program.programId,
						},
					}
				)
			);

			amountChange = amountChange.sub(amountToBuyForTick);

			// Get the previous tick in the bitmap
			currentTick = bitmap.getNextTick(currentTick);
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
		let tickAccountPDA;
		let insuranceContractForTickPDA;
		let insuranceContract;
		let amountToReduceForTick;
		let amountToInsureForTick = new anchor.BN(0);

		// Reduce position tick for tick
		while (amountChange.gt(new anchor.BN(0))) {
			tickAccountPDA = await this.getTickAccountPDA(
				pool,
				tokenMint,
				currentTick
			);
			insuranceContractForTickPDA = await this.getInsuranceContractPDA(
				tickAccountPDA
			);
			insuranceContract = await this.program.account.insuranceContract.fetch(
				insuranceContractForTickPDA
			);
			amountToReduceForTick = min(
				insuranceContract.insuredAmount,
				amountChange
			);
			amountToInsureForTick = insuranceContract.insuredAmount.sub(
				amountToReduceForTick
			);

			txs.add(
				this.program.instruction.buyInsuranceForTick(
					amountToInsureForTick,
					new anchor.BN(endTs),
					{
						accounts: {
							buyer: this.wallet.publicKey,
							pool: pool,
							tickAccount: tickAccountPDA,
							tokenAccount: tokenAccount.address,
							premiumVault: premiumVaultPDA,
							insuranceContract: insuranceContractForTickPDA,
							tokenProgram: TOKEN_PROGRAM_ID,
							systemProgram: this.program.programId,
						},
					}
				)
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
