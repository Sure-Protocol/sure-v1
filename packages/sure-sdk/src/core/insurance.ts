import * as anchor from '@project-serum/anchor';

import { PublicKey } from '@solana/web3.js';
import * as Tick from './tickAccount';
import * as liquidity from './liquidity';
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
import { Wallet } from '@project-serum/anchor/dist/cjs/provider';
import { min } from 'bn.js';

import { Program } from '@project-serum/anchor';
import { SurePool } from './../anchor/types/sure_pool';

export const getPremiumVaultPDA = async (
	program: Program<SurePool>,
	pool: PublicKey,
	tokenMint: PublicKey
): Promise<PublicKey> => {
	const [premiumVaultPDA, premiumVaultBump] =
		await PublicKey.findProgramAddress(
			[SURE_PREMIUM_POOL_SEED, pool.toBytes(), tokenMint.toBytes()],
			program.programId
		);
	return premiumVaultPDA;
};

export const getInsuranceContractPDA = async (
	program: Program<SurePool>,
	tickAccount: PublicKey,
	owner: PublicKey
): Promise<PublicKey> => {
	const [insuranceContractPDA, insuranceContractBump] =
		await PublicKey.findProgramAddress(
			[SURE_INSURANCE_CONTRACT, owner.toBytes(), tickAccount.toBytes()],
			program.programId
		);

	return insuranceContractPDA;
};

export const estimateYearlyPremium = async (
	program: Program<SurePool>,
	amount: number,
	tokenMint: PublicKey,
	pool: PublicKey,
	owner: PublicKey
): Promise<[amountCovered: anchor.BN, insurancePrice: anchor.BN]> => {
	const poolAccount = await program.account.poolAccount.fetch(pool);
	const insuranceFee = poolAccount.insuranceFee;

	/// Estimate premium
	let bitmapPDA = await liquidity.getLiquidityPositionBitmapPDA(
		program,
		pool,
		tokenMint
	);
	const liquidityPositions = await program.account.bitMap.fetch(bitmapPDA);
	const bitmap = Bitmap.new(liquidityPositions);

	console.log('available liquidity in pool: ', poolAccount.liquidity);

	// Check if there is enough
	let totalPremium = new anchor.BN(0);
	let amountToPay = new anchor.BN(0);
	let amountToCover = new anchor.BN(amount);
	let amountCovered = new anchor.BN(0);
	let tick = bitmap.getLowestTick();

	// Get tick account
	let tickAccountPDA = await Tick.getTickAccountPDA(
		program,
		pool,
		tokenMint,
		tick
	);
	let tickAccount = await program.account.tick.fetch(tickAccountPDA);
	let availableLiquidity = tickAccount.liquidity.sub(tickAccount.usedLiquidity);

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

		bitmapPDA = await liquidity.getLiquidityPositionBitmapPDA(
			program,
			pool,
			tokenMint
		);
		tick = bitmap.getNextTick(tick);
		tickAccountPDA = await Tick.getTickAccountPDA(
			program,
			pool,
			tokenMint,
			tick
		);
		tickAccount = await program.account.tick.fetch(tickAccountPDA);
		availableLiquidity = tickAccount.liquidity.sub(tickAccount.usedLiquidity);

		amountCovered = amountCovered.add(amountToPay);
	}

	// Add insurance fee
	const sureFee = totalPremium.muln(insuranceFee / 10000);
	const insurancePrice = totalPremium.add(sureFee);
	return [amountCovered, insurancePrice];
};

/**
 * Create insurance contract for given tick
 * The insurance contract holds information about
 *
 * @param owner<publickey>: Owner of insurance contract
 * @param tickAccount<publickey>: The tick to buy insurance from
 *
 */
export const createInsuranceContractForTick = async (
	program: Program<SurePool>,
	owner: PublicKey,
	tickAccount: PublicKey,
	pool: PublicKey,
	tokenMint: PublicKey,
	userInsuranceContractsPDA: PublicKey
): Promise<PublicKey> => {
	// Get insurance contract with pool
	const insuranceContractPDA = await getInsuranceContractPDA(
		program,
		tickAccount,
		owner
	);

	try {
		await program.methods
			.initializeInsuranceContract()
			.accounts({
				owner: owner,
				pool: pool,
				tokenMint: tokenMint,
				tickAccount: tickAccount,
				insuranceContract: insuranceContractPDA,
				insuranceContracts: userInsuranceContractsPDA,
				systemProgram: program.programId,
			})
			.rpc();
		const insuranceContract = await program.account.insuranceContract.fetch(
			insuranceContractPDA
		);
	} catch (err) {
		throw new Error('could not create insurance contract. Cause: ' + err);
	}

	return insuranceContractPDA;
};

/**
 * Get or create insurance contract for given tick
 * The insurance contract holds information about
 *
 * @param owner<publickey>: Owner of insurance contract
 * @param tickAccount<publickey>: The tick to buy insurance from
 *
 */
export const getOrCreateInsuranceContractForTick = async (
	program: Program<SurePool>,
	owner: PublicKey,
	tickAccount: PublicKey,
	pool: PublicKey,
	tokenMint: PublicKey,
	userInsuranceContractsPDA: PublicKey
): Promise<PublicKey> => {
	const insuranceContractPDA = await getInsuranceContractPDA(
		program,
		tickAccount,
		owner
	);

	try {
		const insuranceContract =
			await program.account.insuranceContract.getAccountInfo(
				insuranceContractPDA
			);
		if (insuranceContract !== null) {
			return insuranceContractPDA;
		}
		throw new Error();
	} catch (_) {
		// Insurance contract does not exist. Create it
		await createInsuranceContractForTick(
			program,
			owner,
			tickAccount,
			pool,
			tokenMint,
			userInsuranceContractsPDA
		);
	}

	return insuranceContractPDA;
};

export const getInsuranceContractsBitmapPDA = async (
	program: Program<SurePool>,
	owner: PublicKey,
	pool: PublicKey
): Promise<PublicKey> => {
	const [insuranceContractsPDA, insuranceContractsBump] =
		await PublicKey.findProgramAddress(
			[SURE_INSURANCE_CONTRACTS, owner.toBytes(), pool.toBytes()],
			program.programId
		);
	return insuranceContractsPDA;
};

/**
 * Calculate the amount insured by user
 *
 * @param owner<publickey>: Owner of insurance contract
 * @param tokenMint<publickey>: the mint account publickkey
 * @param pool<PublicKey>: the pool to buy insurance from
 *
 * @returns Promise for a Big Number - BN
 */
export const getInsuredAmount = async (
	program: Program<SurePool>,
	owner: PublicKey,
	tokenMint: PublicKey,
	pool: PublicKey
): Promise<anchor.BN> => {
	const userInsuranceContractsPDA = await getInsuranceContractsBitmapPDA(
		program,
		owner,
		pool
	);
	try {
		const userInsuranceContracts = await program.account.bitMap.fetch(
			userInsuranceContractsPDA
		);
		// Create insurance contract bitmap
		const insuranceContractBitmap = Bitmap.new(userInsuranceContracts);

		// Start from right and reduce position
		let currentTick = insuranceContractBitmap.getHighestTick();
		let amount = new anchor.BN(0);

		while (currentTick !== -1) {
			const tickAccountPDA = await Tick.getTickAccountPDA(
				program,
				pool,
				tokenMint,
				currentTick
			);
			const insuranceContractForTickPDA = await getInsuranceContractPDA(
				program,
				tickAccountPDA,
				owner
			);
			const insuranceContractForTick =
				await program.account.insuranceContract.fetch(
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
};

export const createUserInsuranceContracts = async (
	program: Program<SurePool>,
	owner: PublicKey,
	pool: PublicKey
): Promise<PublicKey> => {
	try {
		const insuranceContractPDA = await getInsuranceContractsBitmapPDA(
			program,
			owner,
			pool
		);
		await program.methods
			.initializeUserInsuranceContracts()
			.accounts({
				signer: owner,
				pool: pool,
				insuranceContracts: insuranceContractPDA,
				systemProgram: program.programId,
			})
			.rpc();

		return insuranceContractPDA;
	} catch (err) {
		throw new Error('Could not initialize Insurance Contracts. Cause: ' + err);
	}
};

export const getOrCreateUserInsuranceContracts = async (
	program: Program<SurePool>,
	owner: PublicKey,
	pool: PublicKey
): Promise<PublicKey> => {
	const insuranceContractPDA = await getInsuranceContractsBitmapPDA(
		program,
		owner,
		pool
	);
	try {
		const res = await program.account.bitMap.getAccountInfo(
			insuranceContractPDA
		);
		if (res !== null) {
			return insuranceContractPDA;
		}
		throw new Error();
	} catch (_) {
		return createUserInsuranceContracts(program, owner, pool);
	}
};

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
export const buyInsurance = async (
	connection: anchor.web3.Connection,
	program: Program<SurePool>,
	newInsuredAmount: number,
	endTimestamp: number,
	tokenMint: PublicKey,
	pool: PublicKey,
	wallet: Wallet
) => {
	console.log('// Buy insurance ');
	const newInsuredAmountBN = new anchor.BN(newInsuredAmount);

	// Create insurance overview
	const insuredAmount = await getInsuredAmount(
		program,
		wallet.publicKey,
		tokenMint,
		pool
	);
	let amountChange = new anchor.BN(0);
	// Check if amount is changed
	if (newInsuredAmountBN.gte(insuredAmount)) {
		const insuranceContractsPDA = await getOrCreateUserInsuranceContracts(
			program,
			wallet.publicKey,
			pool
		);
		const liquidityPositionsPDA = await liquidity.getLiquidityPositionBitmapPDA(
			program,
			pool,
			tokenMint
		);
		const liquidityPositions = await program.account.bitMap.fetch(
			liquidityPositionsPDA
		);
		const liquidityPositionsBitmap = Bitmap.new(liquidityPositions);

		amountChange = newInsuredAmountBN.sub(insuredAmount);
		await increaseInsurancePosition(
			connection,
			program,
			amountChange,
			liquidityPositionsBitmap,
			endTimestamp,
			wallet,
			pool,
			tokenMint,
			insuranceContractsPDA
		);
	} else {
		const insuranceContractsPDA = await getOrCreateUserInsuranceContracts(
			program,
			wallet.publicKey,
			pool
		);
		const insuranceContracts = await program.account.bitMap.fetch(
			insuranceContractsPDA
		);
		const insuranceContractsBitmap = Bitmap.new(insuranceContracts);

		amountChange = insuredAmount.sub(newInsuredAmountBN);
		await reduceInsurancePositon(
			connection,
			program,
			amountChange,
			insuranceContractsBitmap,
			endTimestamp,
			wallet,
			pool,
			tokenMint
		);
	}
};

/**
 * Changes the end date / contract expiry for all the held contracts
 *
 * @param wallet<Wallet>: the wallet used to pay and sign transactions
 * @param connection<Connection>: rpc connection
 * @param endTimestamp<number>: the timestamp of the contract expiry
 * @param pool<Pubkey>: the liquidity pool used to buy the insurance from
 * @param tokenMint<Pubkey>: the mint of the token used to deposit liquidity
 *
 * @returns none
 */
export const changeContractExpiry = async (
	connection: anchor.web3.Connection,
	wallet: Wallet,
	program: Program<SurePool>,
	endTimestamp: number,
	pool: PublicKey,
	tokenMint: PublicKey
) => {
	const insuranceContractsPDA = await getOrCreateUserInsuranceContracts(
		program,
		wallet.publicKey,
		pool
	);
	const insuranceContracts = await program.account.bitMap.fetch(
		insuranceContractsPDA
	);
	const insuranceContractsBitmap = Bitmap.new(insuranceContracts);
	const tokenAccount = await getOrCreateAssociatedTokenAccount(
		connection,
		(wallet as NodeWallet).payer,
		tokenMint,
		wallet.publicKey
	);
	const premiumVaultPDA = await getPremiumVaultPDA(program, pool, tokenMint);

	// Step through all insurance positions and update the end data
	let currentTick = insuranceContractsBitmap.getLowestTick();
	let insuredAmountConst;
	let insuranceContractForTickPDA;
	let insuranceContract;
	let tickAccountPDA;
	let txs = new anchor.web3.Transaction();

	while (currentTick !== -1) {
		// Fetch insurance contract for current tick
		tickAccountPDA = await Tick.getTickAccountPDA(
			program,
			pool,
			tokenMint,
			currentTick
		);
		insuranceContractForTickPDA = await getOrCreateInsuranceContractForTick(
			program,
			wallet.publicKey,
			tickAccountPDA,
			pool,
			tokenMint,
			insuranceContractsPDA
		);
		insuranceContract = await program.account.insuranceContract.fetch(
			insuranceContractForTickPDA
		);
		insuredAmountConst = insuranceContract.insuredAmount;

		txs.add(
			program.instruction.buyInsuranceForTick(
				insuredAmountConst,
				new anchor.BN(endTimestamp),
				{
					accounts: {
						buyer: wallet.publicKey,
						pool: pool,
						tickAccount: tickAccountPDA,
						tokenAccount: tokenAccount.address,
						premiumVault: premiumVaultPDA,
						insuranceContract: insuranceContractForTickPDA,
						tokenProgram: TOKEN_PROGRAM_ID,
						systemProgram: program.programId,
					},
				}
			)
		);

		currentTick = insuranceContractsBitmap.getNextTick(currentTick);
	}
	txs.recentBlockhash = (await connection.getLatestBlockhash()).blockhash;
	txs.feePayer = wallet.publicKey;

	try {
		const provider = await anchor.getProvider();
		await provider.send?.(txs);
	} catch (err) {
		console.log('logs?: ', err?.logs);
		throw new Error(
			'Sure.buyInsurance. Could not buy insurance. Cause: ' + err
		);
	}
};

const increaseInsurancePosition = async (
	connection: anchor.web3.Connection,
	program: Program<SurePool>,
	amountChange: anchor.BN,
	bitmap: Bitmap,
	endTs: number,
	wallet: Wallet,
	pool: PublicKey,
	tokenMint: PublicKey,
	insuranceContractsPDA: PublicKey
) => {
	console.log('// increaseInsurancePosition');
	// Start from left and increase position
	let currentTick = bitmap.getLowestTick();

	const tokenAccount = await getOrCreateAssociatedTokenAccount(
		connection,
		(wallet as NodeWallet).payer,
		tokenMint,
		wallet.publicKey
	);
	const premiumVaultPDA = await getPremiumVaultPDA(program, pool, tokenMint);

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
		tickAccount = await program.account.tick.fetch(tickAccountPDA);

		insuranceContractForTickPDA = await getOrCreateInsuranceContractForTick(
			program,
			wallet.publicKey,
			tickAccountPDA,
			pool,
			tokenMint,
			insuranceContractsPDA
		);

		insuranceContract = await program.account.insuranceContract.fetch(
			insuranceContractForTickPDA
		);

		availableLiquidity = tickAccount.liquidity.sub(tickAccount.usedLiquidity);
		console.log('> availableLiquidity: ', availableLiquidity.toString());
		amountToBuyForTick = min(availableLiquidity, amountChange);
		insureAmountForTick =
			insuranceContract.insuredAmount.add(amountToBuyForTick);
		console.log('> amountToInsureForTick: ', insureAmountForTick.toString());
		txs.add(
			program.instruction.buyInsuranceForTick(
				insureAmountForTick,
				new anchor.BN(endTs),
				{
					accounts: {
						buyer: wallet.publicKey,
						pool: pool,
						tickAccount: tickAccountPDA,
						tokenAccount: tokenAccount.address,
						premiumVault: premiumVaultPDA,
						insuranceContract: insuranceContractForTickPDA,
						tokenProgram: TOKEN_PROGRAM_ID,
						systemProgram: program.programId,
					},
				}
			)
		);

		amountChange = amountChange.sub(amountToBuyForTick);

		// Get the previous tick in the bitmap
		currentTick = bitmap.getNextTick(currentTick);
	}
	txs.recentBlockhash = (await connection.getLatestBlockhash()).blockhash;
	txs.feePayer = wallet.publicKey;

	try {
		const provider = await anchor.getProvider();
		await provider.send?.(txs);
	} catch (err) {
		console.log('logs?: ', err?.logs);
		throw new Error(
			'Sure.buyInsurance. Could not buy insurance. Cause: ' + err
		);
	}
};

const reduceInsurancePositon = async (
	connection: anchor.web3.Connection,
	program: Program<SurePool>,
	amountChange: anchor.BN,
	bitmap: Bitmap,
	endTs: number,
	wallet: Wallet,
	pool: PublicKey,
	tokenMint: PublicKey
) => {
	// Start from right and reduce position
	let currentTick = bitmap.getHighestTick();
	const tokenAccount = await getOrCreateAssociatedTokenAccount(
		connection,
		(wallet as NodeWallet).payer,
		tokenMint,
		wallet.publicKey
	);
	const premiumVaultPDA = await getPremiumVaultPDA(program, pool, tokenMint);

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
		tickAccountPDA = await Tick.getTickAccountPDA(
			program,
			pool,
			tokenMint,
			currentTick
		);
		insuranceContractForTickPDA = await getInsuranceContractPDA(
			program,
			tickAccountPDA,
			wallet.publicKey
		);
		insuranceContract = await program.account.insuranceContract.fetch(
			insuranceContractForTickPDA
		);
		amountToReduceForTick = min(insuranceContract.insuredAmount, amountChange);
		amountToInsureForTick = insuranceContract.insuredAmount.sub(
			amountToReduceForTick
		);

		txs.add(
			program.instruction.buyInsuranceForTick(
				amountToInsureForTick,
				new anchor.BN(endTs),
				{
					accounts: {
						buyer: wallet.publicKey,
						pool: pool,
						tickAccount: tickAccountPDA,
						tokenAccount: tokenAccount.address,
						premiumVault: premiumVaultPDA,
						insuranceContract: insuranceContractForTickPDA,
						tokenProgram: TOKEN_PROGRAM_ID,
						systemProgram: program.programId,
					},
				}
			)
		);

		amountChange = amountChange.sub(amountToReduceForTick);
		// Get the previous tick in the bitmap
		currentTick = bitmap.getPreviousTick(currentTick);
	}
	txs.recentBlockhash = (await connection.getLatestBlockhash()).blockhash;
	txs.feePayer = wallet.publicKey;

	try {
		const provider = await anchor.getProvider();
		await provider.send?.(txs);
	} catch (err) {
		console.log('logs?: ', err?.logs);
		throw new Error(
			'Sure.buyInsurance. Could not buy insurance. Cause: ' + err
		);
	}
};
