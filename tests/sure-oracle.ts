import * as anchor from '@project-serum/anchor';
import * as solana_contrib from '@saberhq/solana-contrib';
import * as tribeca from '@tribecahq/tribeca-sdk';
import * as goki from '@gokiprotocol/client';
import * as token_utils from '@saberhq/token-utils';
import * as web3 from '@solana/web3.js';
import * as spl from '@solana/spl-token';
import { Transaction } from '@solana/web3.js';
import {
	createVoteHash,
	Provider,
	revealVote,
	SureOracleSDK,
} from '../packages/oracle-sdk/src';
import {
	createAssociatedTokenAccount,
	transfer,
	createMint,
	mintTo,
} from '@solana/spl-token';
import NodeWallet from '@project-serum/anchor/dist/cjs/nodewallet';
import { Keypair } from '@solana/web3.js';
import { LAMPORTS_PER_SOL } from '@solana/web3.js';
import { createTestProposal } from './setup';
import { TransactionError } from '@solana/web3.js';
import { SendTransactionError } from '@solana/web3.js';
import { assert } from 'chai';

describe('Test Sure Oracle', () => {
	const provider = anchor.AnchorProvider.env();
	const { connection } = provider;
	anchor.setProvider(provider);
	// const anchorProvider = new anchor.AnchorProvider(
	// 	connection,
	// 	wallet,
	// 	provider.opts
	// );
	const oracleProvider = solana_contrib.SolanaProvider.init({
		connection,
		wallet: provider.wallet,
		opts: provider.opts,
	});
	const oracleSdk = SureOracleSDK.init({ provider: oracleProvider });
	const tribecaSDK = tribeca.TribecaSDK.load({ provider: oracleProvider });

	/// create mock smart wallet
	const gokiSDK = goki.GokiSDK.load({ provider: oracleProvider });

	let mint: web3.PublicKey;
	let walletATA: web3.PublicKey;
	let proposalName: string;
	let sureLocker: web3.PublicKey;
	let governorKey: web3.PublicKey;
	before(async () => {
		proposalName = 'test 1';
		// load up wallet
		const minterWallet = Keypair.generate();
		const airdrop = await connection.requestAirdrop(
			minterWallet.publicKey,
			10 * LAMPORTS_PER_SOL
		);
		await connection.confirmTransaction(airdrop);
		mint = await createMint(
			connection,
			minterWallet,
			minterWallet.publicKey,
			minterWallet.publicKey,
			8
		);

		walletATA = await createAssociatedTokenAccount(
			connection,
			minterWallet,
			mint,
			provider.wallet.publicKey
		);

		const res = await mintTo(
			connection,
			minterWallet,
			mint,
			walletATA,
			minterWallet,
			100_000_000
		);
		await connection.confirmTransaction(res);

		// create smart wallet
		try {
			const base = Keypair.generate();
			const [governor] = await tribeca.findGovernorAddress(base.publicKey);
			const owners = [governor];
			const smartWallet = await gokiSDK.newSmartWallet({
				owners,
				threshold: new anchor.BN(1),
				numOwners: 1,
				base: base,
			});
			await smartWallet.tx.confirm();

			// set up locker
			const governSDK = new tribeca.GovernWrapper(tribecaSDK);

			// generate a locker from a base keypair
			const locker = tribeca.getLockerAddress(base.publicKey);
			// create governor controlled by the locker

			const govern = await governSDK.createGovernor({
				electorate: locker,
				smartWallet: smartWallet.smartWalletWrapper.key,
				baseKP: base,
			});
			await govern.tx.confirm();
			governorKey = govern.wrapper.governorKey;
			const createLockerRes = await tribecaSDK.createLocker({
				governor: governorKey,
				govTokenMint: mint,
				baseKP: base,
			});
			await createLockerRes.tx.confirm();
			sureLocker = createLockerRes.locker;
		} catch (e) {
			const error = e as SendTransactionError;
			console.log('error: ', error);
			throw new Error('before error');
		}

		// create locker associated with governor
	}),
		it('Create a new proposal ', async () => {
			try {
				const txRceipt = await createTestProposal(
					oracleSdk,
					mint,
					proposalName,
					new anchor.BN(10_000_000)
				);
			} catch (err) {
				throw new Error('failed to create proposal');
			}
		});
	it('Vote on proposal', async () => {
		try {
			const [proposal] = SureOracleSDK.pda().findProposalAddress(proposalName);
			const lockerWrapperSDK = new tribeca.LockerWrapper(
				tribecaSDK,
				sureLocker,
				governorKey
			);

			// lock tokens in escrow
			const lockTokensRes = await lockerWrapperSDK.lockTokens({
				amount: new anchor.BN(1_000_000),
				duration: tribeca.ONE_YEAR,
			});
			await lockTokensRes.confirm();

			const [userEscrow] = await tribeca.findEscrowAddress(
				sureLocker,
				provider.wallet.publicKey
			);
			const voteTx = await oracleSdk.vote().submitVote({
				vote: new anchor.BN(4.2),
				mint: mint,
				proposal,
				locker: sureLocker,
				userEscrow,
			});
			await voteTx.transactionEnvelope.confirm();
		} catch (e) {
			const error = e as SendTransactionError;
			console.log('error: ', error);
		}
	});
	it('Update vote ', async () => {
		try {
			const [proposal] = SureOracleSDK.pda().findProposalAddress(proposalName);
			const eggVote = new anchor.BN(4.3);

			const voteTx = await oracleSdk.vote().updateVote({
				vote: eggVote,
				proposal,
			});
			await voteTx.transactionEnvelope.confirm();
			const [voteAccountPDA] = await SureOracleSDK.pda().findVoteAccount({
				proposal,
				voter: provider.wallet.publicKey,
			});
			const voteAccount = await oracleSdk.program.account.voteAccount.fetch(
				voteAccountPDA
			);

			// validate vote hash
			const isCorrectVote = revealVote({
				expectedVoteHash: voteAccount.voteHash,
				vote: eggVote,
				salt: voteTx.salt,
			});
			assert.equal(isCorrectVote, true, 'vote was not valid');
		} catch (e) {
			const error = e as SendTransactionError;
			console.log(error);
		}
	});
	it('Cancel vote', async () => {
		try {
			const [proposal] = SureOracleSDK.pda().findProposalAddress(proposalName);
			const [voteAccountPDA] = await SureOracleSDK.pda().findVoteAccount({
				proposal,
				voter: provider.wallet.publicKey,
			});
			oracleSdk.vote().cancelVote({ voteAccount: voteAccountPDA });
		} catch (e) {
			const error = e as SendTransactionError;
			console.log(error);
		}
	});
});
