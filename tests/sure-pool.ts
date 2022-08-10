import { assert } from 'chai';
import * as anchor from '@project-serum/anchor';
import {
	getNearestValidTickIndex,
	getNextValidTickIndex,
	priceToTickIndex,
	tickIndexToPrice,
} from '@orca-so/whirlpool-sdk/dist/index';

import {
	createMint,
	transfer,
	Mint,
	mintTo,
	getMint,
	createAssociatedTokenAccount,
	TOKEN_PROGRAM_ID,
	ASSOCIATED_TOKEN_PROGRAM_ID,
	getAccount,
	getAssociatedTokenAddress,
} from '../node_modules/@solana/spl-token';

import { SurePool } from '../target/types/sure_pool';
import {
	PublicKey,
	LAMPORTS_PER_SOL,
	TokenAccountsFilter,
	TokenAmount,
} from '@solana/web3.js';
import NodeWallet from '@project-serum/anchor/dist/cjs/nodewallet';
import { findProgramAddressSync } from '@project-serum/anchor/dist/cjs/utils/pubkey';
import Decimal from 'decimal.js';
import {
	MetadataAccount,
	TokenMetadataProgram,
} from '@metaplex-foundation/js-next';
import { getUnixTime, SureDate } from '@surec/sdk';
const { SystemProgram } = anchor.web3;

/// =============== Variables ==================

// PDA seeds
const program = anchor.workspace.SurePool as anchor.Program<SurePool>;
const TICK_ARRAY_SIZE = 64;
/// Token for Sure Pool
let tokenMintAccount: Mint;
let minterWallet: anchor.web3.Keypair;
let liqudityProviderWallet: anchor.web3.Keypair;

/// Token mints
let tokenMint0: PublicKey;
let tokenMint1: PublicKey;
let originAccountA: PublicKey;
let originAccountB: PublicKey;

let metadataUpdateAuthority = new PublicKey(
	'rYhoVCsVF8dahDpAYUZ9sDygLbhoVgRcczMxnQhWWjg'
);
let vault0: PublicKey;

// global test vars
let tickSpacing = 20;

// PDAs
let protcolToInsure0: anchor.web3.Keypair;

/// ============== TESTS ===========================

const toX64 = (num: anchor.BN): anchor.BN => {
	return num.mul(new anchor.BN(2).pow(new anchor.BN(64)));
};

const fromX64 = (num: anchor.BN): anchor.BN => {
	return num.div(new anchor.BN(2).pow(new anchor.BN(64)));
};

const getMetaplexMetadataPDA = async (
	nftMintPDA: PublicKey,
	test?: boolean
): Promise<PublicKey> => {
	const metadataProgramId = test
		? new PublicKey('5F4dJcMHuNp5qYe3JjPY9CK8G3ePR9dZCJ98aZD9Mxgi')
		: TokenMetadataProgram.publicKey;
	const [mpMetadataPDA, mpMetadataBump] = await PublicKey.findProgramAddress(
		[
			Buffer.from('metadata'),
			metadataProgramId.toBuffer(),
			nftMintPDA.toBytes(),
		],
		metadataProgramId
	);
	return mpMetadataPDA;
};

const getPoolPDA = (
	productId: number,
	tokenMintA: PublicKey,
	tokenMintB: PublicKey,
	tickSpacing: number
): PublicKey => {
	const [poolPDA, _poolPDABump] = findProgramAddressSync(
		[
			Buffer.from('sure-pool'),
			new anchor.BN(productId).toBuffer('le', 1),
			tokenMintA.toBytes(),
			tokenMintB.toBytes(),
			new anchor.BN(tickSpacing).toBuffer('le', 2),
		],
		program.programId
	);
	return poolPDA;
};

describe('Initialize Sure Pool', () => {
	const provider = anchor.AnchorProvider.env();
	const { wallet } = program.provider as anchor.AnchorProvider;
	const { connection } = provider;
	anchor.setProvider(provider);

	it('initialize', async () => {
		minterWallet = anchor.web3.Keypair.generate();
		liqudityProviderWallet = anchor.web3.Keypair.generate();

		// Airdrop 1 SOL into each wallet
		let sig = await connection.requestAirdrop(
			minterWallet.publicKey,
			10 * LAMPORTS_PER_SOL
		);
		await connection.confirmTransaction(sig);

		sig = await connection.requestAirdrop(
			wallet.publicKey,
			100 * LAMPORTS_PER_SOL
		);
		await connection.confirmTransaction(sig);
		sig = await connection.requestAirdrop(
			liqudityProviderWallet.publicKey,
			10 * LAMPORTS_PER_SOL
		);
		await connection.confirmTransaction(sig);

		const minterWalletAccount = await connection.getBalance(
			minterWallet.publicKey
		);
		protcolToInsure0 = anchor.web3.Keypair.generate();
		// Create a random mint for testing
		// TODO: The mint should have the same pubkey as USDC
		tokenMint0 = await createMint(
			connection,
			minterWallet,
			minterWallet.publicKey,
			minterWallet.publicKey,
			8
		);

		tokenMint1 = await createMint(
			connection,
			minterWallet,
			minterWallet.publicKey,
			minterWallet.publicKey,
			8
		);
		// If tokenMint0 is larger than tokenMint1, reverse the pubkey
		// so that tokenMint0 < tokenMint1
		if (Buffer.compare(tokenMint0.toBuffer(), tokenMint1.toBuffer()) > 0) {
			const tempToken = tokenMint1;
			tokenMint1 = tokenMint0;
			tokenMint0 = tempToken;
		}

		tokenMintAccount = await getMint(connection, tokenMint0);

		// Create associated token accounts for each wallet for the tokenMint mint
		const minterWalletATA0 = await createAssociatedTokenAccount(
			connection,
			minterWallet,
			tokenMint0,
			minterWallet.publicKey
		);

		const minterWalletATA1 = await createAssociatedTokenAccount(
			connection,
			minterWallet,
			tokenMint1,
			minterWallet.publicKey
		);

		originAccountA = await createAssociatedTokenAccount(
			connection,
			(wallet as NodeWallet).payer,
			tokenMint0,
			wallet.publicKey
		);

		originAccountB = await createAssociatedTokenAccount(
			connection,
			(wallet as NodeWallet).payer,
			tokenMint1,
			wallet.publicKey
		);

		// Mint initial supply to mint authority associated wallet account
		const mintAmount = 1000000 * Math.pow(10, tokenMintAccount.decimals);
		await mintTo(
			connection,
			minterWallet,
			tokenMint0,
			minterWalletATA0,
			minterWallet,
			mintAmount
		);

		await mintTo(
			connection,
			minterWallet,
			tokenMint1,
			minterWalletATA1,
			minterWallet,
			mintAmount
		);

		// Transfer tokens to liqudity provider ATA from Minter
		const tranferAmount = 10 * Math.pow(10, tokenMintAccount.decimals);
		await transfer(
			connection,
			minterWallet,
			minterWalletATA0,
			originAccountA,
			minterWallet,
			tranferAmount
		);

		// Mint to liquidity provider
		await transfer(
			connection,
			minterWallet,
			minterWalletATA1,
			originAccountB,
			minterWallet,
			tranferAmount
		);

		// Validate transfer
		const originAccountAData = await getAccount(connection, originAccountA);
		assert.equal(
			originAccountAData.owner.toBase58(),
			wallet.publicKey.toBase58()
		);
		assert.equal(
			originAccountAData.amount.toString(),
			tranferAmount.toString()
		);
	});
	it('Launch a pool', async () => {
		// create fee package
		const feeRate = 100;
		const protocolFeeRate = 50;
		const foundersFeeRate = 50;
		const productId = 1;
		const initialSqrtPrice = new anchor.BN(Math.sqrt(25));
		const initialSqrtPriceX64 = toX64(initialSqrtPrice);

		const [feePackagePDA, _] = findProgramAddressSync(
			[anchor.utils.bytes.utf8.encode('sure-pool')],
			program.programId
		);
		// initialize fee package
		const txId = await program.methods
			.initializeFeePackage(feeRate, protocolFeeRate, foundersFeeRate)
			.accounts({
				owner: wallet.publicKey,
				feePackage: feePackagePDA,
				systemProgram: SystemProgram.programId,
			})
			.rpc();

		// Lowest
		let tokenMintA = tokenMint0;
		let tokenMintB = tokenMint1;
		if (Buffer.compare(tokenMintB.toBuffer(), tokenMintA.toBuffer()) < 0) {
			tokenMintA = tokenMint1;
			tokenMintB = tokenMint0;
		}

		//pool PDA
		const poolPDA = getPoolPDA(1, tokenMintA, tokenMintB, tickSpacing);

		// Token vault 0 PDA
		const [tokenVaultAPDA, _tokenVaultXBump] = findProgramAddressSync(
			[Buffer.from('sure-pool'), poolPDA.toBytes(), tokenMintA.toBytes()],
			program.programId
		);
		// Token vault 1 PDA
		const [tokenVaultBPDA, _tokenVaultYBump] = findProgramAddressSync(
			[Buffer.from('sure-pool'), poolPDA.toBytes(), tokenMintB.toBytes()],
			program.programId
		);

		try {
			const initPoolTxId = await program.methods
				.initializePool(productId, tickSpacing, initialSqrtPriceX64, 'my pool')
				.accounts({
					creator: wallet.publicKey,
					pool: poolPDA,
					tokenMintA: tokenMintA,
					tokenMintB: tokenMintB,
					poolVaultA: tokenVaultAPDA,
					poolVaultB: tokenVaultBPDA,
					feePackage: feePackagePDA,
					rent: anchor.web3.SYSVAR_RENT_PUBKEY,
					tokenProgram: TOKEN_PROGRAM_ID,
					systemProgram: SystemProgram.programId,
				})

				.rpc();
		} catch (err) {
			console.log('Error: ', err);
		}

		// get info on pool
		const pool = await program.account.pool.fetch(poolPDA);
		assert.equal(pool.sqrtPriceX64.toString(), initialSqrtPriceX64.toString());

		// ================= Initialize Liquidity Position ========================
		// Initialize Tick array for pool
		const tickIndex = getNearestValidTickIndex(
			new Decimal(16),
			tokenMintAccount.decimals,
			tokenMintAccount.decimals,
			tickSpacing
		);

		const pool_data = await program.account.pool.fetch(poolPDA);
		const [tickArray0PDA, _tickArray0Bump] = findProgramAddressSync(
			[
				Buffer.from('sure-pool'),
				tokenMintA.toBytes(),
				tokenMintB.toBytes(),
				new anchor.BN(pool_data.feeRate).toBuffer('le', 2),
				new anchor.BN(tickIndex).toBuffer('le', 2),
			],
			program.programId
		);
		try {
			await program.methods
				.initializeTickArray(tickIndex)
				.accounts({
					creator: wallet.publicKey,
					pool: poolPDA,
					tickArray: tickArray0PDA,
					systemProgram: SystemProgram.programId,
				})
				.rpc();
		} catch (err) {
			console.log('err: ', err);
		}

		//const nextTickIndex = tickIndex + tickSpacing * TICK_ARRAY_SIZE;
		const nextTickIndex = getNearestValidTickIndex(
			new Decimal(25),
			tokenMintAccount.decimals,
			tokenMintAccount.decimals,
			tickSpacing
		);
		console.log('nextTickIndex: ', nextTickIndex);
		console.log(
			'next tick index price: ',
			tickIndexToPrice(
				nextTickIndex,
				tokenMintAccount.decimals,
				tokenMintAccount.decimals
			)
		);
		const [tickArray1PDA, _tickArray1Bump] = findProgramAddressSync(
			[
				Buffer.from('sure-pool'),
				tokenMintA.toBytes(),
				tokenMintB.toBytes(),
				new anchor.BN(pool_data.feeRate).toBuffer('le', 2),
				new anchor.BN(nextTickIndex).toBuffer('le', 2),
			],
			program.programId
		);
		try {
			await program.methods
				.initializeTickArray(nextTickIndex)
				.accounts({
					creator: wallet.publicKey,
					pool: poolPDA,
					tickArray: tickArray1PDA,
					systemProgram: SystemProgram.programId,
				})
				.rpc();
		} catch (err) {
			console.log('err: ', err);
		}

		const tickIndexLower = getNearestValidTickIndex(
			new Decimal(16),
			tokenMintAccount.decimals,
			tokenMintAccount.decimals,
			tickSpacing
		);
		const tickIndexUpper = getNearestValidTickIndex(
			new Decimal(25),
			tokenMintAccount.decimals,
			tokenMintAccount.decimals,
			tickSpacing
		);

		const [positionMintPDA, positionMintBump] = findProgramAddressSync(
			[
				Buffer.from('sure-pool'),
				new anchor.BN(tickIndexLower).toBuffer('le', 4),
				new anchor.BN(tickIndexUpper).toBuffer('le', 4),
				poolPDA.toBytes(),
			],
			program.programId
		);
		const [positionTokenAccountPDA, positionTokenAccountBump] =
			await findProgramAddressSync(
				[Buffer.from('sure-token-account'), positionMintPDA.toBytes()],
				program.programId
			);
		const [liquidityPositionPDA, liquidityPositionBump] =
			findProgramAddressSync(
				[Buffer.from('sure-pool'), positionMintPDA.toBytes()],
				program.programId
			);
		const metadataAccount = await getMetaplexMetadataPDA(
			positionMintPDA,
			false
		);

		console.log('metadataAccount: ', metadataAccount.toString());
		console.log('metadataProgram: ', TokenMetadataProgram.publicKey.toString());
		console.log(
			'metadataUpdateAuthority: ',
			metadataUpdateAuthority.toString()
		);
		try {
			await program.methods
				.initializeLiquidityPosition(tickIndexUpper, tickIndexLower)
				.accounts({
					liquidityProvider: wallet.publicKey,
					pool: poolPDA,
					liquidityPosition: liquidityPositionPDA,
					positionMint: positionMintPDA,
					positionTokenAccount: positionTokenAccountPDA,
					metadataAccount: metadataAccount,
					metadataProgram: TokenMetadataProgram.publicKey,
					metadataUpdateAuthority: metadataUpdateAuthority,
					associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
					tokenProgram: TOKEN_PROGRAM_ID,
					rent: anchor.web3.SYSVAR_RENT_PUBKEY,
					systemProgram: SystemProgram.programId,
				})
				.rpc();
		} catch (err) {
			console.log('err: ', err);
		}

		// ==================== Increase Liquidity Position ====================

		const liquidityAmount = new anchor.BN(1_000_000);
		const positionTokenAccount = await getAccount(
			connection,
			positionTokenAccountPDA
		);

		console.log('positionTokenAccount amount: ', positionTokenAccount);
		console.log(
			'positionTokenAccount owner: ',
			positionTokenAccount.owner.toString()
		);
		console.log('wallet owner: ', wallet.publicKey.toString());
		try {
			await program.methods
				.increaseLiquidityPosition(
					liquidityAmount,
					liquidityAmount,
					liquidityAmount
				)
				.accounts({
					liquidityProvider: wallet.publicKey,
					liquidityPosition: liquidityPositionPDA,
					positionTokenAccount: positionTokenAccountPDA,
					pool: poolPDA,
					originAccountA: originAccountA,
					originAccountB: originAccountB,
					vaultA: tokenVaultAPDA,
					vaultB: tokenVaultBPDA,
					tickArrayLower: tickArray0PDA,
					tickArrayUpper: tickArray1PDA,
					tokenProgram: TOKEN_PROGRAM_ID,
				})
				.rpc();
		} catch (err) {
			console.log('err: ', err);
		}

		// Check amount in each vault
		const vaultAAccount = await getAccount(connection, tokenVaultAPDA);
		const vaultBAccount = await getAccount(connection, tokenVaultBPDA);
		console.log('vaultAAccount amount: ', vaultAAccount.amount);
		console.log('vaultBAccount amount: ', vaultBAccount.amount);

		const poolAccount = await program.account.pool.fetch(poolPDA);
		console.log('poolAccount liq: ', poolAccount.liquidity);

		// ============= Decrease liquidity Position ======================
		const liquidityAmountReduction = new anchor.BN(250_000);
		try {
			await program.methods
				.decreaseLiquidityPosition(
					liquidityAmountReduction,
					liquidityAmountReduction,
					liquidityAmountReduction
				)
				.accounts({
					liquidityProvider: wallet.publicKey,
					liquidityPosition: liquidityPositionPDA,
					positionTokenAccount: positionTokenAccountPDA,
					pool: poolPDA,
					originAccountA: originAccountA,
					originAccountB: originAccountB,
					vaultA: tokenVaultAPDA,
					vaultB: tokenVaultBPDA,
					tickArrayLower: tickArray0PDA,
					tickArrayUpper: tickArray1PDA,
					tokenProgram: TOKEN_PROGRAM_ID,
				})
				.rpc();
		} catch (err) {
			console.log('decreaseLiquidityPosition err: ', err);
			throw new Error('failure ');
		}
		console.log(
			'token vault A amount: ',
			await (
				await getAccount(connection, tokenVaultAPDA)
			).amount
		);
		const updatedLiquidityPostion =
			await program.account.liquidityPosition.fetch(liquidityPositionPDA);
		console.log(
			'updatedLiquidityPostion: ',
			updatedLiquidityPostion.liquidity.toString()
		);

		// =============== Initilize Coverage Position ==============

		// PDA coverage position
		const [coveragePositionMintPDA, coveragePositionMintBump] =
			findProgramAddressSync(
				[
					Buffer.from('sure-coverage'),
					poolPDA.toBuffer(),
					new anchor.BN(tickIndexLower).toBuffer('le', 4),
				],
				program.programId
			);
		const coveragePositionTokenAccount = await getAssociatedTokenAddress(
			coveragePositionMintPDA,
			wallet.publicKey
		);
		const coverageMetadataAccount = await getMetaplexMetadataPDA(
			coveragePositionMintPDA,
			false
		);
		try {
			await program.methods
				.initializeCoveragePosition(tickIndexLower)
				.accounts({
					user: wallet.publicKey,
					pool: poolPDA,
					positionMint: coveragePositionMintPDA,
					positionTokenAccount: coveragePositionTokenAccount,
					metadataAccount: coverageMetadataAccount,
					metadataProgram: TokenMetadataProgram.publicKey,
					metadataUpdateAuthority: metadataUpdateAuthority,
					associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
					tokenProgram: TOKEN_PROGRAM_ID,
					rent: anchor.web3.SYSVAR_RENT_PUBKEY,
					systemProgram: SystemProgram.programId,
				})
				.rpc();
		} catch (err) {
			console.log('initializeCoveragePosition err: ', err);
			throw new Error('initializeCoveragePosition fail');
		}

		/// ================= Purchase Cover ===================
		const converageAmount = 250_000;
		const expiryTs = SureDate.new(getUnixTime())
			.addHours(1000)
			.getTimeInSeconds();
		try {
			await program.methods.increaseCoveragePosition();
		} catch (err) {
			console.log('');
		}
	});
});
