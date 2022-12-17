//! Integration tests directly against the Sure oracle / prediction market
import * as anchor from '@project-serum/anchor';
import * as tribeca from '@tribecahq/tribeca-sdk';
import * as solana_contrib from '@saberhq/solana-contrib';
import * as goki from '@gokiprotocol/client';
import * as web3 from '@solana/web3.js';
import * as spl from '@solana/spl-token';
import { createMint, mintTo, getMint } from '@solana/spl-token';
import { Oracle } from '../target/types/oracle';
import { findProgramAddressSync } from '@project-serum/anchor/dist/cjs/utils/pubkey';
import { SHAKE } from 'sha3';
import NodeWallet from '@project-serum/anchor/dist/cjs/nodewallet';

const findConfigPDA = (
	tokenMint: web3.PublicKey,
	programId: web3.PublicKey
) => {
	return findProgramAddressSync(
		[Buffer.from('sure-oracle-config'), tokenMint.toBytes()],
		programId
	);
};

const findProposalPDA = (id: Buffer, programId: web3.PublicKey) => {
	return findProgramAddressSync([Buffer.from('sure-oracle'), id], programId);
};

const findRevealVoteArrayPDA = (id: Buffer, programId: web3.PublicKey) => {
	return findProgramAddressSync(
		[Buffer.from('sure-oracle-reveal-array'), id],
		programId
	);
};

const findProposalVaultPDA = (id: Buffer, programId: web3.PublicKey) => {
	return findProgramAddressSync(
		[Buffer.from('sure-oracle-propsal-vault'), id],
		programId
	);
};

export const createProposalHash = ({ name }: { name: string }): Buffer => {
	const hash = new SHAKE(128);
	hash.update(name);
	return hash.digest();
};

export const topUpAccount = async ({
	connection,
	pk,
}: {
	connection: web3.Connection;
	pk: web3.PublicKey;
}) => {
	const airdrop = await connection.requestAirdrop(
		pk,
		10 * web3.LAMPORTS_PER_SOL
	);
	await connection.confirmTransaction(airdrop);
};

export const convertSureTokensToDecimals = async ({
	connection,
	tokenMint,
	amount,
}: {
	connection: web3.Connection;
	amount: number;
	tokenMint: web3.PublicKey;
}) => {
	const mint = await getMint(connection, tokenMint);
	return new anchor.BN(amount).mul(
		new anchor.BN(10).pow(new anchor.BN(mint.decimals))
	);
};

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
			const governor = await tribeca.findGovernorAddress(base.publicKey);
			const owners = [governor[0]];
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

			// create a sure locker
			const createLocker = await tribecaSDK.createLocker({
				governor: govern.wrapper.governorKey,
				govTokenMint: sureMint,
				baseKP: base,
			});
			await createLocker.tx.confirm();
			const sureLockerPK = createLocker.locker;
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
		const id = createProposalHash({ name: '1' });
		console.log('id lenght: ', id.byteLength);
		const name = 'test123';
		const description = 'This is a test proposal';
		const stake = new anchor.BN(10).mul(new anchor.BN(1000000));

		// get necessary accounts
		try {
			const proposer1 = web3.Keypair.generate();
			await topUpAccount({ connection, pk: proposer1.publicKey });
			const proposer1Ata = await spl.createAssociatedTokenAccount(
				connection,
				proposer1,
				sureMint,
				proposer1.publicKey
			);
			console.log('proposer1Ata: ', proposer1Ata);
			// mint sure tokens to wallet
			const transferAmount = await convertSureTokensToDecimals({
				connection,
				tokenMint: sureMint,
				amount: 100,
			});
			console.log('transferAmount: ', transferAmount);
			const minterTokenAccount = await spl.getAccount(
				connection,
				minterWalletSureATA
			);
			console.log('minterTokenAccount: ', minterTokenAccount.amount.toString());
			const res = await spl.transfer(
				connection,
				minterWallet,
				minterWalletSureATA,
				proposer1Ata,
				minterWallet,
				BigInt(transferAmount.toString())
			);
			const tokenAccount = await spl.getAccount(connection, proposer1Ata);
			console.log('token balence: ', tokenAccount.amount.toString());
			const [configPda] = findConfigPDA(sureMint, program.programId);
			const [proposalPda] = findProposalPDA(id, program.programId);
			const [revealVoteArray] = findRevealVoteArrayPDA(id, program.programId);
			const [proposalVault] = findProposalVaultPDA(id, program.programId);
			await program.methods
				.proposeVote(id, name, description, stake)
				.accounts({
					config: configPda,
					proposal: proposalPda,
					revealVoteArray: revealVoteArray,
					proposalVault,
					proposerAccount: proposer1Ata,
					proposalVaultMint: sureMint,
				})
				.rpc();
		} catch (err) {
			console.log('err: ', err);
			throw new Error('Could not create proposal. Cause ' + err);
		}
	});
});
