import * as anchor from '@project-serum/anchor';
import * as solana_contrib from '@saberhq/solana-contrib';
import * as token_utils from '@saberhq/token-utils';
import * as web3 from '@solana/web3.js';
import * as spl from '@solana/spl-token';
import { Transaction } from '@solana/web3.js';
import { OracleIDL } from '../packages/idls/oracle';
import { Provider, SureOracleSDK } from '../packages/oracle-sdk/src';
import {
	createAssociatedTokenAccount,
	transfer,
	createMint,
	mintTo,
} from '@solana/spl-token';
import NodeWallet from '@project-serum/anchor/dist/cjs/nodewallet';
import { Keypair } from '@solana/web3.js';
import { LAMPORTS_PER_SOL } from '@solana/web3.js';

const program = anchor.workspace.Oracle as anchor.Program<OracleIDL>;

describe('Test Sure Oracle', () => {
	const provider = anchor.AnchorProvider.env();
	const { wallet } = program.provider as anchor.AnchorProvider;
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
	let mint: web3.PublicKey;
	let walletATA: web3.PublicKey;
	before(async () => {
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
	}),
		it('Create a new proposal ', async () => {
			const proposedStake = new anchor.BN(100_000_000);
			const createProposal = await oracleSdk.proposal().proposeVote({
				name: 'proposal 1',
				description: 'how many eggs are in the basket',
				stake: proposedStake,
				mint,
			});
			try {
				await createProposal.confirm();
			} catch (err) {
				const error = err as web3.SendTransactionError;
				console.log('error: ', error);
			}
		});
});
