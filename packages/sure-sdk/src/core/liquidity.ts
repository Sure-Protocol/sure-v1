import * as anchor from '@project-serum/anchor';
import {
	ASSOCIATED_TOKEN_PROGRAM_ID,
	getAccount,
	TOKEN_PROGRAM_ID,
} from '@solana/spl-token';
import { PublicKey } from '@solana/web3.js';
import * as pool from './pool';
import * as protocol from './protocol';
import * as Tick from './tickAccount';
import { TokenMetadataProgram } from '@metaplex-foundation/js-next';
const { SystemProgram } = anchor.web3;
import {
	SURE_MP_METADATA_SEED,
	SURE_NFT_MINT_SEED,
	SURE_BITMAP,
	SURE_VAULT_POOL_SEED,
	SURE_TOKEN_ACCOUNT_SEED,
	SURE_LIQUIDITY_POSITION,
} from './seeds';

import { Program } from '@project-serum/anchor';
import { SurePool } from './../anchor/types/sure_pool';

export const getMetaplexMetadataPDA = async (
	nftMintPDA: PublicKey
): Promise<PublicKey> => {
	const [mpMetadataPDA, mpMetadataBump] = await PublicKey.findProgramAddress(
		[
			SURE_MP_METADATA_SEED,
			TokenMetadataProgram.publicKey.toBuffer(),
			nftMintPDA.toBytes(),
		],
		TokenMetadataProgram.publicKey
	);
	return mpMetadataPDA;
};

export const getNFTMintPDA = async (
	program: Program<SurePool>,
	nftAccountPDA: PublicKey
): Promise<PublicKey> => {
	const [nftMintPDA, nftMintBump] = await PublicKey.findProgramAddress(
		[SURE_NFT_MINT_SEED, nftAccountPDA.toBytes()],
		program.programId
	);
	return nftMintPDA;
};

export const getLiquidityPositionPDA = async (
	program: Program<SurePool>,
	nftAccountPDA: PublicKey
): Promise<PublicKey> => {
	const [liquidityPositionPDA, liquidityPositionBump] =
		await PublicKey.findProgramAddress(
			[SURE_LIQUIDITY_POSITION, nftAccountPDA.toBytes()],
			program.programId
		);
	return liquidityPositionPDA;
};

export const getLiquidityProviderTokenAccountPDA = async (
	program: Program<SurePool>,
	poolPDA: PublicKey,
	vaultPDA: PublicKey,
	tickBN: anchor.BN,
	nextTickPositionBN: anchor.BN
): Promise<PublicKey> => {
	const [nftAccountPDA, nftAccountBump] = await PublicKey.findProgramAddress(
		[
			SURE_TOKEN_ACCOUNT_SEED,
			poolPDA.toBytes(),
			vaultPDA.toBytes(),
			tickBN.toBuffer('le', 2),
			nextTickPositionBN.toBuffer('le', 8),
		],
		program.programId
	);
	return nftAccountPDA;
};

export const getLiquidityVaultPDA = async (
	program: Program<SurePool>,
	pool: PublicKey,
	tokenMint: PublicKey
): Promise<PublicKey> => {
	const [liquidityVaultPDA, liquidityVaultBump] =
		await PublicKey.findProgramAddress(
			[SURE_VAULT_POOL_SEED, pool.toBytes(), tokenMint.toBytes()],
			program.programId
		);
	return liquidityVaultPDA;
};

export const getNFTMetadataPDA = async (
	nftMintPDA: PublicKey
): Promise<PublicKey> => {
	const [mpMetadataPDA, mpMetadataBump] = await PublicKey.findProgramAddress(
		[
			SURE_MP_METADATA_SEED,
			TokenMetadataProgram.publicKey.toBuffer(),
			nftMintPDA.toBytes(),
		],
		TokenMetadataProgram.publicKey
	);
	return mpMetadataPDA;
};

export const getLiquidityProviderMintPDA = async (
	program: Program<SurePool>,
	nftAccountPDA: PublicKey
): Promise<PublicKey> => {
	const [nftMintPDA, nftMintBump] = await PublicKey.findProgramAddress(
		[SURE_NFT_MINT_SEED, nftAccountPDA.toBytes()],
		program.programId
	);
	return nftMintPDA;
};

export const getLiquidityPositionBitmapPDA = async (
	program: Program<SurePool>,
	pool: PublicKey,
	tokenMint: PublicKey
): Promise<anchor.web3.PublicKey> => {
	const [bitmapPDA, bitmapBump] = await PublicKey.findProgramAddress(
		[SURE_BITMAP, pool.toBytes(), tokenMint.toBytes()],
		program.programId
	);
	return bitmapPDA;
};

/**
 * Deposit liquidity into a Sure pool
 *
 * @param liquidityAmount Amount of liquidity to be transferred
 * @param tick Tick in basis points to supply liquidity to
 * @param liquidityProvider The signer of the transaction
 * @param liquidityProviderATA Associated Token Account for the tokens to be supplied to the pool
 * @param protocolToInsure The Public Key of the sureProgram to insure
 * @param tokenMint The mint of the token to be supplied to the pool. This could be USDC
 * @return Nothing
 */
export const depositLiquidity = async (
	connection: anchor.web3.Connection,
	program: Program<SurePool>,
	liquidityAmount: number,
	tick: number,
	liquidityProvider: PublicKey,
	liquidityProviderATA: PublicKey,
	protocolToInsure: PublicKey,
	tokenMint: PublicKey
) => {
	// Liquidity Pool PDA
	const poolPDA = await pool.getPoolPDA(program, protocolToInsure);
	try {
		await program.account.poolAccount.fetch(poolPDA);
	} catch (err) {
		throw new Error('Pool does not exist. Cause: ' + err);
	}

	// Protocol Owner
	let [protocolOwnerPDA, _] = await protocol.getProtocolOwner(
		program.programId
	);
	try {
		await program.account.protocolOwner.fetch(protocolOwnerPDA);
	} catch (err) {
		throw new Error('Protocol owner does not exist. Cause: ' + err);
	}
	// Liquidity Pool Vault
	const vaultPDA = await getLiquidityVaultPDA(program, poolPDA, tokenMint);
	try {
		await getAccount(connection, vaultPDA);
	} catch (err) {
		throw new Error('Vault does not exist. Cause: ' + err);
	}

	// Get tick account
	const tickAccountPDA = await Tick.createTickAccount(
		program,
		poolPDA,
		tokenMint,
		tick,
		liquidityProvider
	);
	try {
		await program.account.tick.fetch(tickAccountPDA);
	} catch (err) {
		throw new Error('Tick account does not exist. Cause: ' + err);
	}
	//  Generate tick

	const tickBN = new anchor.BN(tick);
	const tickPosition = await Tick.getCurrentTickPosition(
		program,
		poolPDA,
		tokenMint,
		tick
	);
	const nextTickPositionBN = new anchor.BN(tickPosition + 1);

	// Generate nft accounts
	const nftAccount = await getLiquidityProviderTokenAccountPDA(
		program,
		poolPDA,
		vaultPDA,
		tickBN,
		nextTickPositionBN
	);
	const nftMint = await getNFTMintPDA(program, nftAccount);

	let liquidityPositionPDA = await getLiquidityPositionPDA(program, nftAccount);

	// Get bitmap
	const bitmapPDA = await getLiquidityPositionBitmapPDA(
		program,
		poolPDA,
		tokenMint
	);
	try {
		await program.account.bitMap.fetch(bitmapPDA);
	} catch (err) {
		throw new Error('Bitmap does not exist. Cause: ' + err);
	}

	const mpMetadataAccountPDA = await getMetaplexMetadataPDA(nftMint);

	/// Deposit liquidity Instruction
	try {
		const amountBN = new anchor.BN(liquidityAmount);
		await program.methods
			.depositLiquidity(tick, nextTickPositionBN, amountBN)
			.accounts({
				liquidityProvider: liquidityProvider,
				protocolOwner: protocolOwnerPDA,
				liquidityProviderAccount: liquidityProviderATA,
				pool: poolPDA,
				vault: vaultPDA,
				nftMint: nftMint,
				metadataAccount: mpMetadataAccountPDA,
				metadataProgram: TokenMetadataProgram.publicKey,
				liquidityPosition: liquidityPositionPDA,
				nftAccount: nftAccount,
				bitmap: bitmapPDA,
				tickAccount: tickAccountPDA,
				rent: anchor.web3.SYSVAR_RENT_PUBKEY,
				tokenProgram: TOKEN_PROGRAM_ID,
				systemProgram: SystemProgram.programId,
				associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
			})
			.rpc();
	} catch (e) {
		console.log(e);
		throw new Error('sure.error! Could not deposit liqudity. Cause: ' + e);
	}
};

/**
 * Redeem liquidity based on ownership of NFT
 *
 * @param Wallet: the publickey of the signer and payer
 * @param walletATA: Associated token account for the token to be redeemed
 * @param nftAccount: The NFT (account) that should be used to redeem
 *
 */
export const redeemLiquidity = async (
	wallet: PublicKey,
	program: Program<SurePool>,
	walletATA: PublicKey,
	nftAccount: PublicKey,
	insuredTokenAccount: PublicKey
) => {
	const liquidityPositionPDA = await getLiquidityPositionPDA(
		program,
		nftAccount
	);
	let liquidityPosition;
	try {
		liquidityPosition = await program.account.liquidityPosition.fetch(
			liquidityPositionPDA
		);
	} catch (e) {
		throw new Error('could not get liquidity position: ' + e);
	}

	const poolPDA = liquidityPosition.pool;
	const pool = await program.account.poolAccount.fetch(poolPDA);
	const tokenMint = liquidityPosition.tokenMint;
	const tick = liquidityPosition.tick;
	const nftMint = liquidityPosition.nftMint;

	// Protocol Owner
	let [protocolOwnerPDA, _] = await protocol.getProtocolOwner(
		program.programId
	);

	let vaultAccountPDA = await getLiquidityVaultPDA(program, poolPDA, tokenMint);
	let tickAccount = await Tick.getTickAccountPDA(
		program,
		poolPDA,
		tokenMint,
		tick
	);
	let metadataAccountPDA = await getMetaplexMetadataPDA(nftMint);
	try {
		await program.rpc.redeemLiquidity({
			accounts: {
				nftHolder: wallet,
				nftAccount: nftAccount,
				protocolOwner: protocolOwnerPDA,
				liquidityPosition: liquidityPositionPDA,
				tokenAccount: walletATA,
				vault: vaultAccountPDA,
				tickAccount: tickAccount,
				metadataAccount: metadataAccountPDA,
				metadataProgram: TokenMetadataProgram.publicKey,
				pool: poolPDA,
				tokenProgram: TOKEN_PROGRAM_ID,
				systemProgram: SystemProgram.programId,
			},
		});
	} catch (err) {
		throw new Error('sure.reedemLiquidity.error. cause: ' + err);
	}
};
