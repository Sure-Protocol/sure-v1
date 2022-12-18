//! Integration tests directly against the Sure oracle / prediction market
import * as anchor from '@project-serum/anchor';
import * as tribeca from '@tribecahq/tribeca-sdk';
import * as solana_contrib from '@saberhq/solana-contrib';
import * as goki from '@gokiprotocol/client';
import * as web3 from '@solana/web3.js';
import * as spl from '@solana/spl-token';
import { createMint, mintTo, getMint } from '@solana/spl-token';
import { Oracle } from '../../target/types/oracle';
import { findProgramAddressSync } from '@project-serum/anchor/dist/cjs/utils/pubkey';
import { SHAKE } from 'sha3';
import { assert } from 'chai';
import {
	convertSureTokensToDecimals,
	createProposal,
	createProposalHash,
	createVoteHash,
	findConfigPDA,
	findProposalPDA,
	findProposalVaultPDA,
	findVoteAccount,
	topUpAccount,
	topUpSure,
	topUpVeSure,
	voteOnProposal,
} from './utils';
import { buffer } from 'stream/consumers';

describe('Test Sure Prediction Market ', () => {
	const provider = anchor.AnchorProvider.env();
	const { connection } = provider;
	anchor.setProvider(provider);
	const program = anchor.workspace.Oracle as anchor.Program<Oracle>;

	// prepare tribeca ( veSure) and goki (smart wallet) sdks
	const solanaProvider = solana_contrib.SolanaProvider.init({
		connection,
		wallet: provider.wallet,
		opts: provider.opts,
	});
	const tribecaSDK = tribeca.TribecaSDK.load({ provider: solanaProvider });
	const gokiSDK = goki.GokiSDK.load({ provider: solanaProvider });
	const minterWallet = web3.Keypair.generate();
	let sureMint: web3.PublicKey;
	let minterWalletSureATA: web3.PublicKey;
	let sureLocker: web3.PublicKey;
	let governor: web3.PublicKey;
	before(async () => {
		const airdrop = await connection.requestAirdrop(
			minterWallet.publicKey,
			10 * web3.LAMPORTS_PER_SOL
		);
		await connection.confirmTransaction(airdrop);

		// create sure mint with 6 decimals
		sureMint = await createMint(
			connection,
			minterWallet,
			minterWallet.publicKey,
			minterWallet.publicKey,
			6
		);

		try {
			minterWalletSureATA = await spl.createAssociatedTokenAccount(
				connection,
				minterWallet,
				sureMint,
				minterWallet.publicKey
			);

			// genesis mint of sure tokens - 500,000,000
			const genesisMintAmount = new anchor.BN(500000000).mul(
				new anchor.BN(10).pow(new anchor.BN(6))
			);
			const mintPendingTransaction = await spl.mintTo(
				connection,
				minterWallet,
				sureMint,
				minterWalletSureATA,
				minterWallet,
				BigInt(genesisMintAmount.toString())
			);
			await connection.confirmTransaction(mintPendingTransaction);
		} catch (err) {
			throw new Error(`Failed to mint Sure tokens. Cause ${err}`);
		}

		try {
			// Setup Sure governance and token locking
			const base = web3.Keypair.generate();
			const governorAddress = await tribeca.findGovernorAddress(base.publicKey);
			const owners = [governorAddress[0]];
			const pendingSmartWallet = await await gokiSDK.newSmartWallet({
				owners,
				threshold: new anchor.BN(1),
				numOwners: 1,
				base: base,
			});
			await pendingSmartWallet.tx.confirm();
			const smartWallet = pendingSmartWallet.smartWalletWrapper;
			const governSDK = new tribeca.GovernWrapper(tribecaSDK);
			const lockerPK = tribeca.getLockerAddress(base.publicKey);

			// create governor
			const govern = await governSDK.createGovernor({
				electorate: lockerPK,
				smartWallet: smartWallet.key,
				baseKP: base,
			});
			await govern.tx.confirm();

			governor = govern.wrapper.governorKey;
			// create a sure locker
			const createLocker = await tribecaSDK.createLocker({
				governor: govern.wrapper.governorKey,
				govTokenMint: sureMint,
				baseKP: base,
			});
			await createLocker.tx.confirm();
			sureLocker = createLocker.locker;
		} catch (err) {
			throw new Error(`Failed to create Sure governance. Cause: ${err}`);
		}
	});

	it('Initialize config first time ->  ', async () => {
		// create a protocol authority whcih contrals the
		try {
			const protocolAuthority = web3.Keypair.generate();
			await spl.createAssociatedTokenAccount(
				connection,
				minterWallet,
				sureMint,
				protocolAuthority.publicKey
			);
			const configAccount = findConfigPDA(sureMint, program.programId);
			await program.methods
				.initializeConfig(protocolAuthority.publicKey)
				.accounts({
					config: configAccount[0],
					tokenMint: sureMint,
				})
				.rpc();
		} catch (err) {
			throw new Error('Failed to initialize oracle config. Cause ' + err);
		}
	});
	it('Propose vote with required params', async () => {
		const proposer = web3.Keypair.generate();
		const id = createProposalHash({ name: '1' });
		await topUpAccount({ connection, pk: proposer.publicKey });
		await topUpSure({
			connection,
			mint: sureMint,
			minterWallet,
			to: proposer.publicKey,
			amount: 100,
		});
		await createProposal({
			id,
			sureMint,
			program,
			proposer,
		});

		// VOTE ON PROPOSAL
		try {
			const voter1 = web3.Keypair.generate();
			await topUpAccount({ connection, pk: voter1.publicKey });
			await topUpSure({
				connection,
				mint: sureMint,
				minterWallet,
				to: voter1.publicKey,
				amount: 200,
			});

			const voterAccount = await spl.getAssociatedTokenAddress(
				sureMint,
				voter1.publicKey
			);

			// escrow some sure
			await topUpVeSure({
				program,
				tribecaSDK,
				sureLocker,
				governor,
				mint: sureMint,
				voter: voter1,
				amount: 100,
			});

			const lockerWrapper = await tribeca.LockerWrapper.load(
				tribecaSDK,
				sureLocker,
				governor
			);
			const escrowRes = await lockerWrapper.getOrCreateEscrow(voter1.publicKey);

			voteOnProposal({
				voter: voter1,
				proposalId: id,
				program,
				escrow: escrowRes.escrow,
				mint: sureMint,
				locker: sureLocker,
			});
		} catch (err) {
			console.log('err: ', err);
			throw new Error(`failed to cast vote. Cause: ${err}`);
		}
	});
});
