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

import { Program } from '@project-serum/anchor';

import { SurePool } from '../target/types/sure_pool';
import {
	PublicKey,
	LAMPORTS_PER_SOL,
	TokenAccountsFilter,
	TokenAmount,
} from '@solana/web3.js';
import NodeWallet from '@project-serum/anchor/dist/cjs/nodewallet';
import { findProgramAddressSync } from '@project-serum/anchor/dist/cjs/utils/pubkey';
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

// PDAs
let protcolToInsure0: anchor.web3.Keypair;

/// ============== TESTS ===========================

const toX32 = (num: anchor.BN): anchor.BN => {
	return num.mul(new anchor.BN(2).pow(new anchor.BN(32)));
};

const fromX32 = (num: anchor.BN): anchor.BN => {
	return num.div(new anchor.BN(2).pow(new anchor.BN(32)));
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
		const tickSpacing = 20;
		const initialSqrtPrice = new anchor.BN(Math.sqrt(25));
		const initialSqrtPriceX32 = toX32(initialSqrtPrice);

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
				.initializePool(productId, tickSpacing, initialSqrtPriceX32, 'my pool')
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
		assert.equal(pool.sqrtPriceX32.toString(), initialSqrtPriceX32.toString());

		// Initialize Tick array for pool
		const startSqrtPrice = new anchor.BN(Math.sqrt(4));
		const startSqrtPricex32 = toX32(startSqrtPrice);
	});
	it('');
});
