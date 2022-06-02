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

import { Money } from '@sure/sdk/src';
import { SureSdk, SureDate, Bitmap, seeds, pool } from '@sure/sdk';
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
	const provider = anchor.AnchorProvider.env();
	const { wallet } = program.provider as anchor.AnchorProvider;
	const { connection } = provider;
	anchor.setProvider(provider);
	const sureSdk = SureSdk.init(connection, wallet);

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
		try {
			await sureSdk.protocol.initializeProtocol();
		} catch (err) {
			throw new Error('sure.test. create protocol owner. Cause: ' + err);
		}
	});

	it('create Sure pool manager', async () => {
		const [managerPDA, _] = await PublicKey.findProgramAddress(
			[seeds.SURE_POOL_MANAGER_SEED],
			program.programId
		);

		await sureSdk.pool.initializePoolManager();
		const onChainManager = await program.account.poolManager.fetch(managerPDA);
		assert.equal(
			onChainManager.owner.toBase58(),
			provider.wallet.publicKey.toBase58()
		);
	}),
		it('create sure pool', async () => {
			const insuranceFee = 0;
			const name = 'my awesome sure pool';

			// Generate PDA for Sure Pool
			const poolPDA = await sureSdk?.pool.getPoolPDA(
				protcolToInsure0.publicKey
			);

			await sureSdk.pool.createPool(
				tokenMint,
				protcolToInsure0.publicKey,
				insuranceFee
			);

			const newPool = await program.account.poolAccount.fetch(poolPDA);
			assert.isAbove(newPool.bump, 0);

			const surePoolsPDA = await sureSdk.pool.getSurePoolsPDA();
			const surePoolsAccount = await program.account.surePools.fetch(
				surePoolsPDA
			);
			const surePools = surePoolsAccount.pools;
			let isInPool = false;
			surePools.forEach((poolPDAItem) => {
				console.log(
					'poolPDA: ',
					poolPDA.toBase58(),
					' , poolPDAItem: ',
					poolPDAItem.toBase58()
				);
				if (poolPDA.toBase58() === poolPDAItem.toBase58()) {
					isInPool = true;
				}
			});

			assert.isTrue(isInPool);
		}),
		it('create pool vaults -> For a given mint the isolated ', async () => {
			// Smart contract that sure should insure.

			// Generate PDA for Sure Pool
			const pool = await sureSdk.pool.getPoolPDA(protcolToInsure0.publicKey);

			const poolLiquidityTickBitmap =
				await sureSdk.pool.getPoolLiquidityTickBitmapPDA(pool, tokenMint);

			await sureSdk.pool.createPoolVault(tokenMint, protcolToInsure0.publicKey);

			const bitmapAccount = await program.account.bitMap.fetch(
				poolLiquidityTickBitmap
			);
			assert.equal(bitmapAccount.spacing, 10);
		}),
		it('get list of existing pools', async () => {
			/// the full list of pools should be returned
			const surePoolsPDA = await sureSdk.pool.getSurePoolsPDA();

			try {
				const surePools = await program.account.surePools.fetch(surePoolsPDA);

				assert.equal(surePools.pools.length, 1);
				const firstPoolPDA = surePools.pools[0];
				console.log('surePools.pools: ', surePools.pools);

				try {
					const pool = await program.account.poolAccount.fetch(firstPoolPDA);
				} catch (err) {
					throw new Error('Pool does not exist. Cause: ' + err);
				}
			} catch (err) {
				throw new Error('Could not get Sure Pools. Cause: ' + err);
			}
		}),
		it('create tick account for pool', async () => {
			const tick = 440;
			const poolPDA = await sureSdk.pool.getPoolPDA(protcolToInsure0.publicKey);
			await sureSdk.tickAccount.createLiquidityTickInfo(
				poolPDA,
				tokenMint,
				tick
			);

			const tickPDA = await sureSdk.pool.getLiquidityTickInfoPDA(
				poolPDA,
				tokenMint,
				tick
			);

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
				await sureSdk.liquidity.depositLiquidity(
					wallet.publicKey,
					walletATAPubkey,
					protcolToInsure0.publicKey,
					tokenMint,
					amount.convertToDecimals(),
					tick
				);
			} catch (err) {
				console.log('logs?: ', err?.logs);
				throw new Error('Deposit liquidity error. Cause:' + err);
			}

			const poolPDA = await sureSdk.pool.getPoolPDA(protcolToInsure0.publicKey);
			const vaultPDA = await sureSdk.liquidity.getPoolVaultPDA(
				poolPDA,
				tokenMint
			);
			const tickPosition = await sureSdk.tickAccount.getCurrentTickPosition(
				poolPDA,
				tokenMint,
				tick
			);
			const tickAccountPDA = await sureSdk.tickAccount.getLiquidityTickInfoPDA(
				poolPDA,
				tokenMint,
				tick
			);
			const tickAccount = await program.account.tick.fetch(tickAccountPDA);

			const nftAccountPDA =
				await sureSdk.liquidity.getLiquidityPositionTokenAccountPDA(
					poolPDA,
					vaultPDA,
					new anchor.BN(tick),
					new anchor.BN(tickPosition)
				);
			let nftAccount = await getAccount(connection, nftAccountPDA);
			assert.equal(nftAccount.amount, 1);
			/// Get liquidity position
			const liquidityPositionPDA =
				await sureSdk.liquidity.getLiquidityPositionPDA(nftAccountPDA);
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
			const sureNfts = await sureSdk.nft.getSureNfts();
			/// Select one NFT to redeem
			const reedemableNFT = sureNfts[0];

			// Redeem liquidity
			try {
				await sureSdk.liquidity.redeemLiquidity(
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
		const dateNow = new SureDate();
		let hours = 10;
		let contractExpiry = dateNow.addHours(hours);
		let contractExpiryInSeconds = contractExpiry.getTimeInSeconds();

		// deposit liquidity
		try {
			await sureSdk.liquidity.depositLiquidity(
				wallet.publicKey,
				walletATAPubkey,
				protcolToInsure0.publicKey,
				tokenMint,
				liquidity.convertToDecimals(),
				tick
			);
		} catch (err) {
			throw new Error('deposit liquidity error. Cause:' + err);
		}

		try {
			await sureSdk.liquidity.depositLiquidity(
				wallet.publicKey,
				walletATAPubkey,
				protcolToInsure0.publicKey,
				tokenMint,
				liquidity.setAmount(1000).convertToDecimals(),
				150
			);
		} catch (err) {
			throw new Error('deposit liquidity error. Cause:' + err);
		}

		// Find pool to target
		const poolPDA = await sureSdk.pool.getPoolPDA(protcolToInsure0.publicKey);

		// Calculate cost of insurance
		//const [potentialAmountCovered,price] = await sureSdk.estimateYearlyPremium(positionSize,tokenMint,poolPDA,wallet.publicKey)

		await sureSdk.insurance.buyInsurance(
			poolPDA,
			tokenMint,
			positionSize.convertToDecimals(),
			contractExpiryInSeconds
		);
		const userInsuranceContractsPDA =
			await sureSdk.insurance.getInsurancePoolContractsPDA(poolPDA, tokenMint);
		const userInsuranceContracts =
			await program.account.insuranceTickContract.fetch(
				userInsuranceContractsPDA
			);
		console.log('userInsuranceContracts: ', userInsuranceContracts);
		// Check the user positions
		console.log('Buy insurance > getInsured amount');
		let insuredAmount = await sureSdk.insurance.getInsuredAmount(
			poolPDA,
			tokenMint
		);
		console.log('insuredAmount:: ', insuredAmount);
		console.log('positionSize: ', positionSize);
		assert.isTrue(
			insuredAmount.eq(new anchor.BN(positionSize.convertToDecimals()))
		);
		console.log('insurance: ', insuredAmount.toString());

		// // Buy more insurance
		// const insurancePosition = Money.new(tokenMintAccount.decimals, 17000);
		// await sureSdk.insurance.buyInsurance(
		// 	poolPDA,
		// 	tokenMint,
		// 	insurancePosition.convertToDecimals(),
		// 	contractExpiryInSeconds
		// );
		// insuredAmount = await sureSdk.insurance.getInsuredAmount(
		// 	poolPDA,
		// 	tokenMint
		// );
		// assert.isTrue(
		// 	insuredAmount.eq(new anchor.BN(positionSize.convertToDecimals()))
		// );
		// console.log('insurance: ', insuredAmount.toString());

		// // reduce position
		// await sureSdk.insurance.buyInsurance(
		// 	poolPDA,
		// 	tokenMint,
		// 	positionSize.setAmount(15000).convertToDecimals(),
		// 	contractExpiryInSeconds
		// );
		// insuredAmount = await sureSdk.insurance.getInsuredAmount(
		// 	poolPDA,
		// 	tokenMint
		// );
		// assert.isTrue(
		// 	insuredAmount.eq(new anchor.BN(positionSize.convertToDecimals()))
		// );
		// console.log('insurance: ', insuredAmount.toString());

		// // Change contract expiry
		// hours = 20;
		// const contractExpiryD = dateNow.addHours(hours);
		// await sureSdk.insurance.changeContractExpiry(
		// 	poolPDA,
		// 	tokenMint,
		// 	contractExpiryD.getTimeInSeconds()
		// );

		// // fetch a contract
		// const insuranceContractsPDA =
		// 	await sureSdk.insurance.getInsuranceContractsBitmapPDA(
		// 		poolPDA,
		// 		tokenMint
		// 	);
		// const insuranceContracts = await program.account.bitMap.fetch(
		// 	insuranceContractsPDA
		// );
		// const insuranceContractsBitmap = Bitmap.new(insuranceContracts);
		// const firstTick = insuranceContractsBitmap.getLowestTick();
		// const tickAccountPDA = await sureSdk.tickAccount.getTickAccountPDA(
		// 	poolPDA,
		// 	tokenMint,
		// 	firstTick
		// );
		// const insuranceContractPDA =
		// 	await sureSdk.insurance.getInsuranceContractPDA(tickAccountPDA);
		// const insuranceContract = await program.account.insuranceContract.fetch(
		// 	insuranceContractPDA
		// );
		// const insuranceContractExpiry = insuranceContract.endTs.toString();
		// assert.isTrue(
		// 	new anchor.BN(contractExpiryD.getTimeInSeconds()).eq(
		// 		insuranceContract.endTs
		// 	)
		// );

		// /// Test for different hours

		// /// TODO: check premium calculations
		// const insuranceContractPremium = insuranceContract.premium.toString();
		// console.log('insuranceContractPremium: ', insuranceContractPremium);
	});
});
