import { assert } from 'chai';
import * as anchor from '@project-serum/anchor';
import { Program } from '@project-serum/anchor';
import { Money, SurePool, SureSdk } from '@sure/sdk';
import {
	PublicKey,
	LAMPORTS_PER_SOL,
	TokenAccountsFilter,
} from '@solana/web3.js';
import {
	createAssociatedTokenAccount,
	createMint,
	getAccount,
	getMint,
	mintTo,
} from '@solana/spl-token';
import NodeWallet from '@project-serum/anchor/dist/cjs/nodewallet';

const program = anchor.workspace.SurePool as Program<SurePool>;

let tokenMint: PublicKey;

describe('Provide Liquidity', () => {
	const provider = anchor.AnchorProvider.env();
	const { wallet } = program.provider as anchor.AnchorProvider;
	const { connection } = provider;
	anchor.setProvider(provider);
	const sureSdk = SureSdk.init(connection, wallet);
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
		const tokenAccountAta = await createAssociatedTokenAccount(
			connection,
			(wallet as NodeWallet).payer,
			tokenMint,
			wallet.publicKey
		);

		console.log('tokenMintAccount.decimals: ', tokenMintAccount.decimals);
		const mintAmount = Money.new(tokenMintAccount.decimals, 100);
		await mintTo(
			connection,
			(wallet as NodeWallet).payer,
			tokenMint,
			tokenAccountAta,
			(wallet as NodeWallet).payer,
			mintAmount.convertToDecimals()
		);

		// Assert the correct amount
		const account = await getAccount(connection, tokenAccountAta);
		console.log('account amount: ', account.amount);
		const amount = new anchor.BN(account.amount);
		assert(new anchor.BN(mintAmount.convertToDecimals()).eq(amount));

		// Create protocol owner
		try {
			await sureSdk.protocol.initializeProtocol();
		} catch (err) {
			throw new Error('sure.test. create protocol owner. Cause: ' + err);
		}

		// Create Sure pool
		const insuranceFee = 0;
		const smartContract = PublicKey.default;

		const poolPDA = await sureSdk?.pool.getPoolPDA(smartContract);
		await sureSdk.pool.createPool(smartContract, insuranceFee, 'sure-test');
		const newPool = await program.account.poolAccount.fetch(poolPDA);
		assert.isAbove(newPool.bump, 0);

		// Create pool Vault for mint
		await sureSdk.pool.createPoolVault(tokenMint, smartContract);

		/// Deposit liquidity in range
		const liquidityAmount = 100; // amount to draw from account
		const tickStart = 210; // 300bp tick
		const tickEnd = 300;

		// TODO: Deposit some more liquidity from other LPs
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
	});
});
