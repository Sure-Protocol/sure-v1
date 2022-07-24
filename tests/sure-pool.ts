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
} from '@solana/spl-token';

import { Program } from '@project-serum/anchor';

import { SurePool } from '../target/types/sure_pool';
import {
	PublicKey,
	LAMPORTS_PER_SOL,
	TokenAccountsFilter,
	TokenAmount,
} from '@solana/web3.js';
import NodeWallet from '@project-serum/anchor/dist/cjs/nodewallet';
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

let vault0: PublicKey;

// PDAs
let protcolToInsure0: anchor.web3.Keypair;

/// ============== TESTS ===========================

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
});
