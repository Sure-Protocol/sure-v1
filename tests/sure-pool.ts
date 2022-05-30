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
} from '@solana/web3.js';
import NodeWallet from '@project-serum/anchor/dist/cjs/nodewallet';
import { u64 } from '@solana/buffer-layout-utils';
const { SystemProgram } = anchor.web3;

import * as sureSdk from '@sure/sdk';
import { Money } from '@sure/sdk/src';

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

const nftMint: anchor.web3.Keypair = new anchor.web3.Keypair();

// PDAs
let protcolToInsure0: anchor.web3.Keypair;

/// ============== TESTS ===========================

describe('Initialize Sure Pool', () => {
	const provider = anchor.Provider.env();
	const { connection, wallet } = anchor.getProvider();
	anchor.setProvider(provider);

	it('initialize', async () => {
		minterWallet = anchor.web3.Keypair.generate();
		liqudityProviderWallet = anchor.web3.Keypair.generate();

		// Airdrop 1 SOL into each wallet
		const fromAirdropSig = await connection.requestAirdrop(
			minterWallet.publicKey,
			10 * LAMPORTS_PER_SOL
		);
		await connection.confirmTransaction(fromAirdropSig);
		const airdropLP = await connection.requestAirdrop(
			wallet.publicKey,
			10 * LAMPORTS_PER_SOL
		);
		await connection.confirmTransaction(airdropLP);
		const lpAirdrop = await connection.requestAirdrop(
			liqudityProviderWallet.publicKey,
			10 * LAMPORTS_PER_SOL
		);
		await connection.confirmTransaction(lpAirdrop);
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
		const mintAmount = Money.new(tokenMintAccount.decimals, 1000000);
		await mintTo(
			connection,
			minterWallet,
			tokenMint,
			minterWalletATA,
			minterWallet,
			mintAmount.convertToDecimals()
		);

		// Transfer tokens to liqudity provider ATA from Minter
		await transfer(
			connection,
			minterWallet,
			minterWalletATA,
			walletATAPubkey,
			minterWallet,
			mintAmount.setAmount(100000).convertToDecimals()
		);

		await transfer(
			connection,
			minterWallet,
			minterWalletATA,
			liquidityProviderWalletATA,
			minterWallet,
			mintAmount.convertToDecimals()
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
			liquidityProvidertokenMintATA.amount,
			mintAmount.convertToDecimals()
		);
	});

	it('create protocol owner ', async () => {
		let [protocolOwnerPDA, _] = await sureSdk.getProtocolOwner();
		let surePools = await sureSdk.getSurePools();
		await program.methods
			.initializeProtocol()
			.accounts({
				owner: provider.wallet.publicKey,
				protocolOwner: protocolOwnerPDA,
				pools: surePools,
				systemProgram: SystemProgram.programId,
			})
			.rpc();
	});

	it('create Sure pool manager', async () => {
		const [managerPDA, _] = await PublicKey.findProgramAddress(
			[anchor.utils.bytes.utf8.encode('sure-pool-manager')],
			program.programId
		);

		// Create Pool Manager PDA
		await program.rpc.initializePoolManager({
			accounts: {
				manager: managerPDA,
				initialManager: provider.wallet.publicKey,
				systemProgram: SystemProgram.programId,
			},
		});

		const onChainManager = await program.account.poolManager.fetch(managerPDA);
		assert.equal(
			onChainManager.owner.toBase58(),
			provider.wallet.publicKey.toBase58()
		);
	}),
		it('create sure pool', async () => {
			const insuranceFee = 0;
			const tick_spacing = 10; // tick size in basispoints
			const name = 'my awesome sure pool';

			// Generate PDA for Sure Pool
			const poolPDA = await sureSdk.getPoolPDA(protcolToInsure0.publicKey);

			// Generate PDA for token vault
			vault0 = await sureSdk.getLiquidityVaultPDA(poolPDA, tokenMint);

			let [protocolOwnerPDA, _] = await sureSdk.getProtocolOwner();

			let surePoolsList = await sureSdk.getSurePools();

			// Create Poool
			try {
				await program.methods
					.createPool(insuranceFee, name)
					.accounts({
						poolCreator: wallet.publicKey,
						protocolOwner: protocolOwnerPDA,
						pool: poolPDA,
						surePools: surePoolsList,
						insuredTokenAccount: protcolToInsure0.publicKey,
						rent: anchor.web3.SYSVAR_RENT_PUBKEY,
						systemProgram: SystemProgram.programId,
					})
					.rpc();
			} catch (err) {
				throw new Error('Could not create pool. Cause: ' + err);
			}

			const newPool = await program.account.poolAccount.fetch(poolPDA);
			assert.isAbove(newPool.bump, 0);
		}),
		it('create pool vaults -> For a given mint the isolated ', async () => {
			// Smart contract that sure should insure.

			// Generate PDA for Sure Pool
			const pool = await sureSdk.getPoolPDA(protcolToInsure0.publicKey);
			const liqudityPositionBitmap =
				await sureSdk.getLiquidityPositionBitmapPDA(pool, tokenMint);
			const liquidityVault = await sureSdk.getLiquidityVaultPDA(
				pool,
				tokenMint
			);
			const premiumVault = await sureSdk.getPremiumVaultPDA(pool, tokenMint);

			try {
				await program.methods
					.createPoolVaults()
					.accounts({
						creator: wallet.publicKey,
						pool: pool,
						tokenMint: tokenMint,
						liquidityVault: liquidityVault,
						premiumVault: premiumVault,
						bitmap: liqudityPositionBitmap,
						rent: anchor.web3.SYSVAR_RENT_PUBKEY,
						tokenProgram: TOKEN_PROGRAM_ID,
						systemProgram: SystemProgram.programId,
					})
					.rpc();
			} catch (err) {
				throw new Error('could not create Pool vaults. cause: ' + err);
			}

			const bitmapAccount = await program.account.bitMap.fetch(
				liqudityPositionBitmap
			);
			assert.equal(bitmapAccount.spacing, 10);
		}),
		it('get list of existing pools', async () => {
			/// the full list of pools should be returned
			const surePoolsPDA = await sureSdk.getSurePools();

			try {
				const surePools = await program.account.surePools.fetch(surePoolsPDA);

				assert.equal(surePools.pools.length, 1);
				const firstContractInsured = surePools.pools[0];

				// get the first pool
				const poolPDA = await sureSdk.getPoolPDA(firstContractInsured);

				try {
					const pool = await program.account.poolAccount.fetch(poolPDA);
				} catch (err) {
					throw new Error('Pool does not exist. Cause: ' + err);
				}
			} catch (err) {
				throw new Error('Could not get Sure Pools. Cause: ' + err);
			}
		}),
		it('create tick account for pool', async () => {
			const tick = 440;
			const poolPDA = await sureSdk.getPoolPDA(protcolToInsure0.publicKey);
			const tickPDA = await sureSdk.getTickAccountPDA(poolPDA, tokenMint, tick);
			try {
				await program.methods
					.initializeTick(poolPDA, tokenMint, tick)
					.accounts({
						creator: wallet.publicKey,
						tickAccount: tickPDA,
						systemProgram: SystemProgram.programId,
					})
					.rpc();
			} catch (err) {
				throw new Error('Could not initialize tick. Cause: ' + err);
			}

			const createdTickAccount = await program.account.tick.fetch(tickPDA);
			assert.equal(createdTickAccount.active, true);
			assert.equal(createdTickAccount.liquidity.toString(), '0');
			assert.equal(createdTickAccount.usedLiquidity.toString(), '0');
			assert.equal(createdTickAccount.tick.toString(), tick.toString());
			assert.equal(createdTickAccount.lastLiquidityPositionIdx, 0);
		}),
		it('deposit liquidity into pool at a given tick', async () => {
			let amount = await Money.new(tokenMintAccount.decimals, 1500); // amount to draw from account
			let tick = 210; // 300bp tick

			// TODO: Deposit some more liquidity from other LPs

			try {
				await sureSdk.depositLiquidity(
					connection,
					amount.convertToDecimals(),
					tick,
					wallet.publicKey,
					walletATAPubkey,
					protcolToInsure0.publicKey,
					tokenMint
				);
			} catch (err) {
				console.log('logs?: ', err?.logs);
				throw new Error('Deposit liquidity error. Cause:' + err);
			}

			const poolPDA = await sureSdk.getPoolPDA(protcolToInsure0.publicKey);
			const vaultPDA = await sureSdk.getLiquidityVaultPDA(poolPDA, tokenMint);
			const tickPosition = await sureSdk.getCurrentTickPosition(
				poolPDA,
				tokenMint,
				tick
			);
			const tickAccountPDA = await sureSdk.getTickAccountPDA(
				poolPDA,
				tokenMint,
				tick
			);
			const tickAccount = await program.account.tick.fetch(tickAccountPDA);

			const nftAccountPDA = await sureSdk.getLPTokenAccountPDA(
				poolPDA,
				vaultPDA,
				new anchor.BN(tick),
				new anchor.BN(tickPosition)
			);
			let nftAccount = await getAccount(connection, nftAccountPDA);
			assert.equal(nftAccount.amount, 1);
			/// Get liquidity position
			const liquidityPositionPDA = await sureSdk.getLiquidityPositionPDA(
				nftAccountPDA
			);
			let liquidityPosition = await program.account.liquidityPosition.fetch(
				liquidityPositionPDA
			);
			assert.equal(
				liquidityPosition.nftAccount.toBase58(),
				nftAccountPDA.toBase58(),
				'nft account not equal to expected address'
			);
		}),
		it('redeem liquidity based on NFT', async () => {
			//  Allow user to provide only the NFT to get the
			// liquidity position and redeem it.
			const sureNfts = await sureSdk.getSureNfts(connection, wallet.publicKey);
			/// Select one NFT to redeem
			const reedemableNFT = sureNfts[0];

			// Redeem liquidity
			try {
				await sureSdk.redeemLiquidity(
					wallet.publicKey,
					walletATAPubkey,
					reedemableNFT.pubkey,
					protcolToInsure0.publicKey
				);
			} catch (err) {
				throw new Error(err);
			}
		});
	it('buy insurance from smart contract pool', async () => {
		/// Variables
		let positionSize = await Money.new(tokenMintAccount.decimals, 15000);
		const liquidity = await Money.new(tokenMintAccount.decimals, 14000);
		const tick = 120;
		const dateNow = new sureSdk.SureDate();
		let hours = 10;
		let contractExpiry = dateNow.addHours(hours);
		let contractExpiryInSeconds = contractExpiry.getTimeInSeconds();

		// deposit liquidity
		try {
			await sureSdk.depositLiquidity(
				connection,
				liquidity.convertToDecimals(),
				tick,
				wallet.publicKey,
				walletATAPubkey,
				protcolToInsure0.publicKey,
				tokenMint
			);
		} catch (err) {
			throw new Error('deposit liquidity error. Cause:' + err);
		}

		try {
			await sureSdk.depositLiquidity(
				connection,
				liquidity.setAmount(1000).convertToDecimals(),
				150,
				wallet.publicKey,
				walletATAPubkey,
				protcolToInsure0.publicKey,
				tokenMint
			);
		} catch (err) {
			throw new Error('deposit liquidity error. Cause:' + err);
		}

		// Find pool to target
		const poolPDA = await sureSdk.getPoolPDA(protcolToInsure0.publicKey);

		// Calculate cost of insurance
		//const [potentialAmountCovered,price] = await sureSdk.estimateYearlyPremium(positionSize,tokenMint,poolPDA,wallet.publicKey)

		await sureSdk.buyInsurance(
			connection,
			positionSize.convertToDecimals(),
			contractExpiryInSeconds,
			tokenMint,
			poolPDA,
			wallet
		);

		// Check the user positions
		console.log('Buy insurance > getInsured amount');
		let insuredAmount = await sureSdk.getInsuredAmount(
			wallet.publicKey,
			tokenMint,
			poolPDA
		);
		assert.isTrue(
			insuredAmount.eq(new anchor.BN(positionSize.convertToDecimals()))
		);
		console.log('insurance: ', insuredAmount.toString());

		// Buy more insurance
		const insurancePosition = Money.new(tokenMintAccount.decimals, 17000);
		await sureSdk.buyInsurance(
			connection,
			insurancePosition.convertToDecimals(),
			contractExpiryInSeconds,
			tokenMint,
			poolPDA,
			wallet
		);
		insuredAmount = await sureSdk.getInsuredAmount(
			wallet.publicKey,
			tokenMint,
			poolPDA
		);
		assert.isTrue(
			insuredAmount.eq(new anchor.BN(positionSize.convertToDecimals()))
		);
		console.log('insurance: ', insuredAmount.toString());

		// reduce position
		await sureSdk.buyInsurance(
			connection,
			positionSize.setAmount(15000).convertToDecimals(),
			contractExpiryInSeconds,
			tokenMint,
			poolPDA,
			wallet
		);
		insuredAmount = await sureSdk.getInsuredAmount(
			wallet.publicKey,
			tokenMint,
			poolPDA
		);
		assert.isTrue(
			insuredAmount.eq(new anchor.BN(positionSize.convertToDecimals()))
		);
		console.log('insurance: ', insuredAmount.toString());

		// Change contract expiry
		hours = 20;
		const contractExpiryD = dateNow.addHours(hours);
		await sureSdk.changeContractExpiry(
			wallet,
			connection,
			contractExpiryD.getTimeInSeconds(),
			poolPDA,
			tokenMint
		);

		// fetch a contract
		const insuranceContractsPDA = await sureSdk.getInsuranceContractsBitmapPDA(
			wallet.publicKey,
			poolPDA
		);
		const insuranceContracts = await program.account.bitMap.fetch(
			insuranceContractsPDA
		);
		const insuranceContractsBitmap = sureSdk.Bitmap.new(insuranceContracts);
		const firstTick = insuranceContractsBitmap.getLowestTick();
		const tickAccountPDA = await sureSdk.getTickAccountPDA(
			poolPDA,
			tokenMint,
			firstTick
		);
		const insuranceContractPDA = await sureSdk.getInsuranceContractPDA(
			tickAccountPDA,
			wallet.publicKey
		);
		const insuranceContract = await program.account.insuranceContract.fetch(
			insuranceContractPDA
		);
		const insuranceContractExpiry = insuranceContract.endTs.toString();
		assert.isTrue(
			new anchor.BN(contractExpiryD.getTimeInSeconds()).eq(
				insuranceContract.endTs
			)
		);

		/// Test for different hours

		/// TODO: check premium calculations
		const insuranceContractPremium = insuranceContract.premium.toString();
		console.log('insuranceContractPremium: ', insuranceContractPremium);
	});
});
