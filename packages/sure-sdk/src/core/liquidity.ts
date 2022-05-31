import * as anchor from '@project-serum/anchor';
import {
	ASSOCIATED_TOKEN_PROGRAM_ID,
	getAccount,
	TOKEN_PROGRAM_ID,
} from '@solana/spl-token';

import { Connection, PublicKey } from '@solana/web3.js';

import * as mp from '@metaplex-foundation/mpl-token-metadata';
const { SystemProgram } = anchor.web3;
import {
	SURE_MP_METADATA_SEED,
	SURE_NFT_MINT_SEED,
	SURE_VAULT_POOL_SEED,
	SURE_TOKEN_ACCOUNT_SEED,
	SURE_LIQUIDITY_POSITION,
} from './seeds';

import { Program } from '@project-serum/anchor';
import { SurePool } from './../anchor/types/sure_pool';
import { Common } from './commont';

export class Liquidity extends Common {
	constructor(
		readonly program: anchor.Program<SurePool>,
		readonly connection: Connection,
		readonly wallet: anchor.Wallet
	) {
		super(program, connection, wallet);
	}

	async getMetaplexMetadataPDA(nftMintPDA: PublicKey): Promise<PublicKey> {
		const [mpMetadataPDA, mpMetadataBump] = await PublicKey.findProgramAddress(
			[SURE_MP_METADATA_SEED, mp.PROGRAM_ID.toBuffer(), nftMintPDA.toBytes()],
			mp.PROGRAM_ID
		);
		return mpMetadataPDA;
	}

	async getLiquidityVaultPDA(
		pool: PublicKey,
		tokenMint: PublicKey
	): Promise<PublicKey> {
		const [liquidityVaultPDA, liquidityVaultBump] =
			await PublicKey.findProgramAddress(
				[SURE_VAULT_POOL_SEED, pool.toBytes(), tokenMint.toBytes()],
				this.program.programId
			);
		return liquidityVaultPDA;
	}

	async getLiquidityPositionPDA(nftAccountPDA: PublicKey): Promise<PublicKey> {
		const [liquidityPositionPDA, liquidityPositionBump] =
			await PublicKey.findProgramAddress(
				[SURE_LIQUIDITY_POSITION, nftAccountPDA.toBytes()],
				this.program.programId
			);
		return liquidityPositionPDA;
	}

	async getLiquidityPositionTokenAccountPDA(
		poolPDA: PublicKey,
		vaultPDA: PublicKey,
		tickBN: anchor.BN,
		nextTickPositionBN: anchor.BN
	): Promise<PublicKey> {
		const [nftAccountPDA, nftAccountBump] = await PublicKey.findProgramAddress(
			[
				SURE_TOKEN_ACCOUNT_SEED,
				poolPDA.toBytes(),
				vaultPDA.toBytes(),
				tickBN.toBuffer('le', 2),
				nextTickPositionBN.toBuffer('le', 8),
			],
			this.program.programId
		);
		return nftAccountPDA;
	}

	async getLiquidityPositionMetadataPDA(
		nftMintPDA: PublicKey
	): Promise<PublicKey> {
		const [mpMetadataPDA, mpMetadataBump] = await PublicKey.findProgramAddress(
			[SURE_MP_METADATA_SEED, mp.PROGRAM_ID.toBuffer(), nftMintPDA.toBytes()],
			mp.PROGRAM_ID
		);
		return mpMetadataPDA;
	}

	async getLiquidityPositionMintPDA(
		nftAccountPDA: PublicKey
	): Promise<PublicKey> {
		const [nftMintPDA, nftMintBump] = await PublicKey.findProgramAddress(
			[SURE_NFT_MINT_SEED, nftAccountPDA.toBytes()],
			this.program.programId
		);
		return nftMintPDA;
	}

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
	depositLiquidity = async (
		liquidityProvider: PublicKey,
		liquidityProviderATA: PublicKey,
		protocolToInsure: PublicKey,
		tokenMint: PublicKey,
		liquidityAmount: number,
		tick: number
	) => {
		// Liquidity Pool PDA
		const poolPDA = await this.getPoolPDA(protocolToInsure);
		try {
			await this.program.account.poolAccount.fetch(poolPDA);
		} catch (err) {
			throw new Error('Pool does not exist. Cause: ' + err);
		}

		// Protocol Owner
		let [protocolOwnerPDA, _] = await this.getProtocolOwner();
		try {
			await this.program.account.protocolOwner.fetch(protocolOwnerPDA);
		} catch (err) {
			throw new Error('Protocol owner does not exist. Cause: ' + err);
		}
		// Liquidity Pool Vault
		const vaultPDA = await this.getLiquidityVaultPDA(poolPDA, tokenMint);
		try {
			await getAccount(this.connection, vaultPDA);
		} catch (err) {
			throw new Error('Vault does not exist. Cause: ' + err);
		}

		// Get tick account
		const tickAccountPDA = await this.getOrCreateTickAccount(
			liquidityProvider,
			poolPDA,
			tokenMint,
			tick
		);
		try {
			await this.program.account.tick.fetch(tickAccountPDA);
		} catch (err) {
			throw new Error('Tick account does not exist. Cause: ' + err);
		}
		//  Generate tick

		const tickBN = new anchor.BN(tick);
		const tickPosition = await this.getCurrentTickPosition(
			poolPDA,
			tokenMint,
			tick
		);
		const nextTickPositionBN = new anchor.BN(tickPosition + 1);

		// Generate nft accounts
		const nftAccount = await this.getLiquidityPositionTokenAccountPDA(
			poolPDA,
			vaultPDA,
			tickBN,
			nextTickPositionBN
		);
		const nftMint = await this.getLiquidityPositionMintPDA(nftAccount);

		let liquidityPositionPDA = await this.getLiquidityPositionPDA(nftAccount);

		// Get bitmap
		const bitmapPDA = await this.getLiquidityPositionBitmapPDA(
			poolPDA,
			tokenMint
		);
		try {
			await this.program.account.bitMap.fetch(bitmapPDA);
		} catch (err) {
			throw new Error('Bitmap does not exist. Cause: ' + err);
		}

		const mpMetadataAccountPDA = await this.getMetaplexMetadataPDA(nftMint);

		/// Deposit liquidity Instruction
		try {
			const amountBN = new anchor.BN(liquidityAmount);
			await this.program.methods
				.depositLiquidity(tick, nextTickPositionBN, amountBN)
				.accounts({
					liquidityProvider: liquidityProvider,
					protocolOwner: protocolOwnerPDA,
					liquidityProviderAccount: liquidityProviderATA,
					pool: poolPDA,
					vault: vaultPDA,
					nftMint: nftMint,
					metadataAccount: mpMetadataAccountPDA,
					metadataProgram: mp.PROGRAM_ID,
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
	async redeemLiquidity(
		wallet: PublicKey,
		walletATA: PublicKey,
		nftAccount: PublicKey,
		insuredTokenAccount: PublicKey
	) {
		const liquidityPositionPDA = await this.getLiquidityPositionPDA(nftAccount);
		let liquidityPosition;
		try {
			liquidityPosition = await this.program.account.liquidityPosition.fetch(
				liquidityPositionPDA
			);
		} catch (e) {
			throw new Error('could not get liquidity position: ' + e);
		}

		const poolPDA = liquidityPosition.pool;
		const pool = await this.program.account.poolAccount.fetch(poolPDA);
		const tokenMint = liquidityPosition.tokenMint;
		const tick = liquidityPosition.tick;
		const nftMint = liquidityPosition.nftMint;

		// Protocol Owner
		let [protocolOwnerPDA, _] = await this.getProtocolOwner();

		let vaultAccountPDA = await this.getLiquidityVaultPDA(poolPDA, tokenMint);
		let tickAccount = await this.getTickAccountPDA(poolPDA, tokenMint, tick);
		let metadataAccountPDA = await this.getMetaplexMetadataPDA(nftMint);
		try {
			await this.program.rpc.redeemLiquidity({
				accounts: {
					nftHolder: wallet,
					nftAccount: nftAccount,
					protocolOwner: protocolOwnerPDA,
					liquidityPosition: liquidityPositionPDA,
					tokenAccount: walletATA,
					vault: vaultAccountPDA,
					tickAccount: tickAccount,
					metadataAccount: metadataAccountPDA,
					metadataProgram: mp.PROGRAM_ID,
					pool: poolPDA,
					tokenProgram: TOKEN_PROGRAM_ID,
					systemProgram: SystemProgram.programId,
				},
			});
		} catch (err) {
			throw new Error('sure.reedemLiquidity.error. cause: ' + err);
		}
	}
}
