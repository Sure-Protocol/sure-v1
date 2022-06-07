import * as anchor from '@project-serum/anchor';
import {
	ASSOCIATED_TOKEN_PROGRAM_ID,
	createAssociatedTokenAccount,
	getAccount,
	getMint,
	getOrCreateAssociatedTokenAccount,
	TOKEN_PROGRAM_ID,
} from '@solana/spl-token';

import { Connection, PublicKey } from '@solana/web3.js';

import * as mp from '@metaplex-foundation/mpl-token-metadata';
const { SystemProgram } = anchor.web3;
import {
	SURE_MP_METADATA_SEED,
	SURE_NFT_MINT_SEED,
	SURE_TOKEN_ACCOUNT_SEED,
	SURE_LIQUIDITY_POSITION,
} from './seeds';

import { Program } from '@project-serum/anchor';
import { SurePool } from './../anchor/types/sure_pool';
import { Common } from './commont';
import NodeWallet from '@project-serum/anchor/dist/cjs/nodewallet';
import { Bitmap, Money } from 'src/utils';

export class Liquidity extends Common {
	constructor(
		readonly program: anchor.Program<SurePool>,
		readonly connection: Connection,
		readonly wallet: anchor.Wallet
	) {
		super(program, connection, wallet);
	}

	async getMetaplexMetadataPDA(
		nftMintPDA: PublicKey,
		test?: boolean
	): Promise<PublicKey> {
		const metadataProgramId = test
			? new PublicKey('5F4dJcMHuNp5qYe3JjPY9CK8G3ePR9dZCJ98aZD9Mxgi')
			: mp.PROGRAM_ID;
		const [mpMetadataPDA, mpMetadataBump] = await PublicKey.findProgramAddress(
			[
				SURE_MP_METADATA_SEED,
				metadataProgramId.toBuffer(),
				nftMintPDA.toBytes(),
			],
			metadataProgramId
		);
		return mpMetadataPDA;
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
	 * Deposit liquidity into a Sure pool in percentage range
	 *
	 * @param pool The pool to deposit liquidity into
	 * @param tokenMint The mint of the tokens deposited
	 * @param amount Amount in usual denominations
	 * @param rangeStart The start of the range in percentage
	 * @param rangeEnd The Public Key of the sureProgram to insure
	 * @return Nothing
	 */
	depositLiquidityInPercentage = async (
		pool: PublicKey,
		tokenMint: PublicKey,
		amount: number,
		rangeStart: number,
		rangeEnd: number
	) => {
		const rangeStartBP = Math.round(rangeStart * 100);
		const rangeEndBP = Math.round(rangeEnd * 100);

		await this.depositLiquidity(
			pool,
			tokenMint,
			amount,
			rangeStartBP,
			rangeEndBP
		);
	};

	/**
	 * Deposit liquidity into a Sure pool in basis points range
	 *
	 * @param pool The pool to deposit liquidity into
	 * @param tokenMint The mint of the tokens deposited
	 * @param amount Amount in usual denominations
	 * @param rangeStartBP The start of the liquidity range in basis points
	 * @param rangeEndBP The end of the liquidity range in basis points
	 * @return Nothing
	 */
	depositLiquidity = async (
		pool: PublicKey,
		tokenMint: PublicKey,
		amount: number,
		rangeStartBP: number,
		rangeEndBP: number,
		test?: boolean
	) => {
		try {
			// Convert amount to amountDecimals
			const tokenDecimals = (await getMint(this.connection, tokenMint))
				.decimals;
			const amountInDecimals = new Money(
				tokenDecimals,
				amount
			).convertToDecimals();

			// Convert ranges to basis points

			// Get bitmap
			let bitmapPDA = await this.getPoolLiquidityTickBitmapPDA(pool, tokenMint);
			let liquidityPositions;
			try {
				liquidityPositions = await this.program.account.bitMap.fetch(bitmapPDA);
			} catch (err) {
				throw new Error('could not get liquidity position bitmap. ' + err);
			}
			const bitmap = Bitmap.new(liquidityPositions);

			const rangeStart = rangeStartBP - (rangeStartBP % bitmap.spacing);
			const rangeEnd = rangeEndBP - (rangeEndBP % bitmap.spacing);

			const amountPerTick =
				(amountInDecimals * (rangeEnd - rangeStart)) / bitmap.spacing;

			let liquidityLeft = amountInDecimals;
			let tick = rangeStart;
			while (liquidityLeft > 0) {
				console.log('tick: ', tick);
				await this.depositLiquidityAtTick(
					pool,
					tokenMint,
					amountPerTick,
					tick,
					test
				);
				liquidityLeft = liquidityLeft - amountPerTick;
				tick = tick + bitmap.spacing;
			}
		} catch (err) {
			throw new Error(
				'sure-sdk.liquidity.depositLiquidity.error. Cause: ' + err
			);
		}
	};

	/**
	 * Deposit liquidity at Tick into a Sure pool
	 *
	 * @param liquidityAmount Amount of liquidity to be transferred
	 * @param tick Tick in basis points to supply liquidity to
	 * @param liquidityProvider The signer of the transaction
	 * @param liquidityProviderATA Associated Token Account for the tokens to be supplied to the pool
	 * @param protocolToInsure The Public Key of the sureProgram to insure
	 * @param tokenMint The mint of the token to be supplied to the pool. This could be USDC
	 * @return Nothing
	 */
	depositLiquidityAtTick = async (
		poolPDA: PublicKey,
		tokenMint: PublicKey,
		liquidityAmount: number,
		tick: number,
		test?: boolean
	) => {
		// Liquidity Pool PDA
		const liquidityProviderAtaAccount = await getOrCreateAssociatedTokenAccount(
			this.connection,
			(this.wallet as NodeWallet).payer,
			tokenMint,
			this.wallet.publicKey
		);

		// Protocol Owner
		let [protocolOwnerPDA, _] = await this.getProtocolOwner();
		try {
			await this.program.account.protocolOwner.fetch(protocolOwnerPDA);
		} catch (err) {
			throw new Error('Protocol owner does not exist. Cause: ' + err);
		}

		// Liquidity Pool Vault
		const poolVaultPDA = await this.getPoolVaultPDA(poolPDA, tokenMint);
		try {
			await getAccount(this.connection, poolVaultPDA);
		} catch (err) {
			throw new Error('Vault does not exist. Cause: ' + err);
		}

		// Get tick account
		const liquidityTickInfoPDA = await this.getOrCreateLiquidityTickInfo(
			poolPDA,
			tokenMint,
			tick
		);
		try {
			await this.program.account.tick.getAccountInfo(liquidityTickInfoPDA);
		} catch (err) {
			throw new Error(
				'Liquidity Tick Info account does not exist. Cause: ' + err
			);
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
		const nftAccountPDA = await this.getLiquidityPositionTokenAccountPDA(
			poolPDA,
			poolVaultPDA,
			tickBN,
			nextTickPositionBN
		);
		const nftMint = await this.getLiquidityPositionMintPDA(nftAccountPDA);

		let liquidityPositionPDA = await this.getLiquidityPositionPDA(
			nftAccountPDA
		);

		// Get bitmap
		const poolLiquidityTickBitmapPDA = await this.getPoolLiquidityTickBitmapPDA(
			poolPDA,
			tokenMint
		);
		try {
			await this.program.account.bitMap.fetch(poolLiquidityTickBitmapPDA);
		} catch (err) {
			throw new Error('Bitmap does not exist. Cause: ' + err);
		}

		const mpMetadataAccountPDA = await this.getMetaplexMetadataPDA(
			nftMint,
			test
		);
		const metadataProgramId = test
			? new PublicKey('5F4dJcMHuNp5qYe3JjPY9CK8G3ePR9dZCJ98aZD9Mxgi')
			: mp.PROGRAM_ID;
		/// Deposit liquidity Instruction
		try {
			const amountBN = new anchor.BN(liquidityAmount);
			await this.program.methods
				.depositLiquidity(tick, nextTickPositionBN, amountBN)
				.accounts({
					liquidityProvider: this.wallet.publicKey,
					protocolOwner: protocolOwnerPDA,
					liquidityProviderAta: liquidityProviderAtaAccount.address,
					pool: poolPDA,
					poolVault: poolVaultPDA,
					liquidityPosition: liquidityPositionPDA,
					liquidityPositionNftMint: nftMint,
					metadataAccount: mpMetadataAccountPDA,
					metadataProgram: metadataProgramId,
					liquidityPositionNftAccount: nftAccountPDA,
					poolLiquidityTickBitmap: poolLiquidityTickBitmapPDA,
					liquidityTickInfo: liquidityTickInfoPDA,
					rent: anchor.web3.SYSVAR_RENT_PUBKEY,
					tokenProgram: TOKEN_PROGRAM_ID,
					systemProgram: SystemProgram.programId,
					associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
				})
				.rpc();
		} catch (e) {
			console.log(e?.logs);
			throw new Error(
				'sure.liquidity.depositLiquidity.error. Could not deposit liqudity. Cause: ' +
					e
			);
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

		let poolVaultPDA = await this.getPoolVaultPDA(poolPDA, tokenMint);
		let liqudityTickInfoPDA = await this.getLiquidityTickInfoPDA(
			poolPDA,
			tokenMint,
			tick
		);
		let metadataAccountPDA = await this.getMetaplexMetadataPDA(nftMint);
		try {
			await this.program.methods.redeemLiquidity().accounts({
				nftHolder: wallet,
				liquidityPositionNftAccount: nftAccount,
				protocolOwner: protocolOwnerPDA,
				liquidityPosition: liquidityPositionPDA,
				liquidityProviderAta: walletATA,
				poolVault: poolVaultPDA,
				liquidityTickInfo: liqudityTickInfoPDA,
				metadataAccount: metadataAccountPDA,
				metadataProgram: mp.PROGRAM_ID,
				pool: poolPDA,
				tokenProgram: TOKEN_PROGRAM_ID,
				systemProgram: SystemProgram.programId,
			});
		} catch (err) {
			throw new Error('sure.reedemLiquidity.error. cause: ' + err);
		}
	}
}
