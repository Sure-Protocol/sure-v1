import { assert } from 'chai';
import * as anchor from '@project-serum/anchor';
import { Program } from '@project-serum/anchor';
import { Money, SureDate, SurePool, SureSdk } from '@surec/sdk';
import {
	PublicKey,
	LAMPORTS_PER_SOL,
	TokenAccountsFilter,
	Keypair,
} from '@solana/web3.js';
import {
	createAssociatedTokenAccount,
	createMint,
	getAccount,
	getMint,
	mintTo,
} from '@solana/spl-token';
import NodeWallet from '@project-serum/anchor/dist/cjs/nodewallet';
import { keypairIdentity } from '@metaplex-foundation/js-next';

const program = anchor.workspace.SurePool as Program<SurePool>;

let tokenMint: PublicKey;
let poolPDA: PublicKey;

describe('Provide Liquidity', () => {
	const provider = anchor.AnchorProvider.env();
	const { wallet } = program.provider as anchor.AnchorProvider;
	const { connection } = provider;
	anchor.setProvider(provider);
	const sureSdk = SureSdk.init(connection, wallet, program.programId);
	let tokenAccountAtaPK;
	let tokenAccountAta;
	let tokenAccountAtaAmount;
	it('Initialize test', async () => {
		await connection.requestAirdrop(wallet.publicKey, 10 * LAMPORTS_PER_SOL);

		// Mint Liquidity token
		tokenMint = await createMint(
			connection,
			(wallet as NodeWallet).payer,
			wallet.publicKey,
			wallet.publicKey,
			8
		);
		const tokenMintAccount = await getMint(connection, tokenMint);

		// Create associated token account
		tokenAccountAtaPK = await createAssociatedTokenAccount(
			connection,
			(wallet as NodeWallet).payer,
			tokenMint,
			wallet.publicKey
		);

		const mintAmount = Money.new(tokenMintAccount.decimals, 110);
		await mintTo(
			connection,
			(wallet as NodeWallet).payer,
			tokenMint,
			tokenAccountAtaPK,
			(wallet as NodeWallet).payer,
			mintAmount.convertToDecimals().toNumber()
		);

		// Assert the correct amount
		tokenAccountAta = await getAccount(connection, tokenAccountAtaPK);
		console.log('account amount: ', tokenAccountAta.amount);
		tokenAccountAtaAmount = new anchor.BN(tokenAccountAta.amount);
		assert(
			new anchor.BN(mintAmount.convertToDecimals()).eq(tokenAccountAtaAmount)
		);

		// Create protocol owner
		try {
			await sureSdk.protocol.initializeProtocol();
		} catch (err) {
			throw new Error('sure.test. create protocol owner. Cause: ' + err);
		}
	});
	it('Deposit some liquidity into a pool', async () => {
		/// Deposit liquidity in range
		const liquidityAmount = 100; // amount to draw from account
		const tickStart = 210; // 300bp tick
		const tickEnd = 220;

		const insuranceFee = 0;
		const smartContract = Keypair.generate();

		poolPDA = await sureSdk?.pool.getPoolPDA(smartContract.publicKey);

		const txId = await sureSdk.pool.initializeTokenPool(
			smartContract.publicKey,
			tokenMint,
			insuranceFee,
			'Test pool'
		);

		try {
			await sureSdk.liquidity.depositLiquidity(
				poolPDA,
				tokenMint,
				liquidityAmount,
				tickStart,
				tickEnd
			);
		} catch (err) {
			console.log('logs?: ', err?.logs);
			throw new Error('Deposit liquidity error. Cause:' + err);
		}

		tokenAccountAta = await getAccount(connection, tokenAccountAtaPK);
		let expectedAmount = tokenAccountAtaAmount.sub(
			await Money.convertBNToDecimals(
				connection,
				new anchor.BN(liquidityAmount),
				tokenMint
			)
		);
		console.log('expectedAmount: ', expectedAmount.toString());
		tokenAccountAtaAmount = new anchor.BN(tokenAccountAta.amount);
		console.log('amount: ', tokenAccountAtaAmount.toString());
		assert.equal(tokenAccountAtaAmount.toString(), expectedAmount);
	});
	it('Test performance of getTokenPoolInformation', async () => {
		let smartContract: Keypair;
		const numPools = 5;
		const insuranceFee = 0;

		// Create a bunch of pools
		for (let i = 0; i < numPools; i++) {
			smartContract = Keypair.generate();
			await sureSdk.pool.initializeTokenPool(
				smartContract.publicKey,
				tokenMint,
				insuranceFee,
				'Test pool'
			);
		}

		// Fetch pools
		let startTimeMs = new Date().valueOf();
		const pools = await sureSdk.pool.getTokenPoolsInformation();
		let endTIme = (new Date().valueOf() - startTimeMs) / 1000;
		console.log('V1 Time to fetch: ', endTIme, 's');
		assert.equal(pools.length, numPools + 1);
		startTimeMs = new Date().valueOf();
		const poolsV2 = await sureSdk.pool.getTokenPoolsInformationV2();
		endTIme = (new Date().valueOf() - startTimeMs) / 1000;
		console.log('V2 Time to fetch: ', endTIme, 's');
		assert.equal(poolsV2.length, numPools + 1);
		console.log(
			'poolsV2: ',
			poolsV2.sort((a, b) =>
				a.address.toBase58().localeCompare(b.address.toBase58())
			)
		);
		console.log(
			'poolsV1: ',
			pools.sort((a, b) =>
				a.address.toBase58().localeCompare(b.address.toBase58())
			)
		);
		assert.equal(
			pools.sort((a, b) =>
				a.address.toBase58().localeCompare(b.address.toBase58())
			),
			poolsV2.sort((a, b) =>
				a.address.toBase58().localeCompare(b.address.toBase58())
			)
		);
	});
	it('Estimate coverage price, buy insurance and get an overview of positions', async () => {
		// Estimate insurance price
		const buyAmount = 99;
		const dateNow = new SureDate();
		let hours = 10;
		let contractExpiry = dateNow.addHours(hours);
		let contractExpiryInSeconds = contractExpiry.getTimeInSeconds();

		const estimate = await sureSdk.insurance.estimateYearlyPremium(
			buyAmount,
			tokenMint,
			poolPDA
		);
		console.log('estimate: ', estimate[0], estimate[1], estimate[2]);
		const expiryDateString = '2022-06-10';
		const expiryDateParsed = Date.parse(expiryDateString);
		const sureDate = SureDate.new(expiryDateParsed);
		console.log('expiryDateParsed: ', sureDate.getTimeInSeconds());
		// Buy insurance
		await sureSdk.insurance.buyInsurance(
			poolPDA,
			tokenMint,
			buyAmount,
			contractExpiryInSeconds
		);

		// Get an overview of positions 
	});
		
});
