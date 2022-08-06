import { assert } from 'chai';
import * as anchor from '@project-serum/anchor';
import {
	createMint,
	TOKEN_PROGRAM_ID,
	transfer,
	mintTo,
	getAccount,
	createAssociatedTokenAccount,
	getMint,
	Mint,
	getOrCreateAssociatedTokenAccount,
} from '@solana/spl-token';

import { priceToTickIndex } from '@orca-so/whirlpool-sdk';

import { Program, ProgramErrorStack } from '@project-serum/anchor';

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
import { MetadataAccount } from '@metaplex-foundation/js-next';
import { MethodsBuilder } from '@project-serum/anchor/dist/cjs/program/namespace/methods';
const { SystemProgram } = anchor.web3;

/// =============== Variables ==================

// PDA seeds
const program = anchor.workspace.SurePool as Program<SurePool>;

/// Token for Sure Pool
let tokenMint: PublicKey;
let tokenMintAccount: Mint;
let minterWallet: anchor.web3.Keypair;
let liqudityProviderWallet: anchor.web3.Keypair;
let walletATAPubkey: PublicKey;
let liquidityProviderWalletATA: PublicKey;

/// Token mints
let tokenMint0: PublicKey;
let tokenMint1: PublicKey;

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
			10 * LAMPORTS_PER_SOL
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
		tokenMint = await createMint(
			connection,
			minterWallet,
			minterWallet.publicKey,
			minterWallet.publicKey,
			8
		);

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

		tokenMintAccount = await getMint(connection, tokenMint);

		// Create associated token accounts for each wallet for the tokenMint mint
		const minterWalletATA = await createAssociatedTokenAccount(
			connection,
			minterWallet,
			tokenMint,
			minterWallet.publicKey
		);

		walletATAPubkey = await createAssociatedTokenAccount(
			connection,
			(wallet as NodeWallet).payer,
			tokenMint,
			wallet.publicKey
		);

		liquidityProviderWalletATA = await createAssociatedTokenAccount(
			connection,
			liqudityProviderWallet,
			tokenMint,
			liqudityProviderWallet.publicKey
		);

		// Mint initial supply to mint authority associated wallet account
		const mintAmount = 1000000 * Math.pow(10, tokenMintAccount.decimals);
		await mintTo(
			connection,
			minterWallet,
			tokenMint,
			minterWalletATA,
			minterWallet,
			mintAmount
		);

		// Transfer tokens to liqudity provider ATA from Minter
		const tranferAmount = 10 * Math.pow(10, tokenMintAccount.decimals);
		await transfer(
			connection,
			minterWallet,
			minterWalletATA,
			walletATAPubkey,
			minterWallet,
			tranferAmount
		);

		// Mint to liquidity provider
		await transfer(
			connection,
			minterWallet,
			minterWalletATA,
			liquidityProviderWalletATA,
			minterWallet,
			tranferAmount
		);

		// Validate transfer
		const liquidityProvidertokenMintATA = await getAccount(
			connection,
			walletATAPubkey
		);
		assert.equal(
			liquidityProvidertokenMintATA.owner.toBase58(),
			wallet.publicKey.toBase58()
		);
		assert.equal(
			liquidityProvidertokenMintATA.amount.toString(),
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

		// Initialize Tick array for pool
		const tickIndex = priceToTickIndex(
			new Decimal(4),
			tokenMintAccount.decimals,
			tokenMintAccount.decimals
		);
		console.log('tick index: ', tickIndex);
		const pool_data = await program.account.pool.fetch(poolPDA);
		const [tickArrayPDA, _tickArrayBump] = findProgramAddressSync(
			[
				Buffer.from('sure-pool'),
				tokenMintA.toBytes(),
				tokenMintB.toBytes(),
				new anchor.BN(pool_data.feeRate).toBuffer('le', 2),
				new anchor.BN(tickIndex).toBuffer('le', 2),
			],
			program.programId
		);
		const initTickArrayIx = await program.methods
			.initializeTickArray(tickIndex)
			.accounts({
				creator: wallet.publicKey,
				pool: poolPDA,
				tickArray: tickArrayPDA,
				systemProgram: SystemProgram.programId,
			});

		console.log('initTickArrayIx: ', initTickArrayIx);
	});
	it('Supply liquidity to Pool', async () => {
		const tickIndexLower = priceToTickIndex(
			new Decimal(4),
			tokenMintAccount.decimals,
			tokenMintAccount.decimals
		);
		const tickIndexUpper = priceToTickIndex(
			new Decimal(4),
			tokenMintAccount.decimals,
			tokenMintAccount.decimals
		);
		const poolPDA = getPoolPDA(1, tokenMint0, tokenMint1, tickSpacing);
		// await program.methods
		// 	.initializeLiquidityPosition(tickIndexLower, tickIndexUpper)
		// 	.accounts({
		// 		liquidityProvider: wallet.publicKey,
		// 		pool: poolPDA,
		// 	});
	});
});
