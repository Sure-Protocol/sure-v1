import { findProgramAddressSync } from '@project-serum/anchor/dist/cjs/utils/pubkey';
import SHA3, { SHAKE } from 'sha3';
import * as web3 from '@solana/web3.js';
import * as spl from '@solana/spl-token';
import { AnchorError } from '@project-serum/anchor';
import { Oracle } from '../../../target/types/oracle';
import * as anchor from '@project-serum/anchor';
import { assert } from 'chai';
import * as tribeca from '@tribecahq/tribeca-sdk';
import * as solana_contrib from '@saberhq/solana-contrib';
import { Idl } from '@project-serum/anchor/dist/esm';

const SURE_ORACLE_VOTE_SEED = 'sure-oracle-vote';
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

export const topUpSure = async ({
	connection,
	mint,
	minterWallet,
	to,
	amount,
}: {
	connection: web3.Connection;
	mint: web3.PublicKey;
	minterWallet: web3.Signer;
	to: web3.PublicKey;
	amount: number;
}) => {
	try {
		const toAta = await spl.getOrCreateAssociatedTokenAccount(
			connection,
			minterWallet,
			mint,
			to
		);

		const transferAmount = await convertSureTokensToDecimals({
			connection,
			tokenMint: mint,
			amount,
		});
		const minterWalletAta = await spl.getAssociatedTokenAddress(
			mint,
			minterWallet.publicKey
		);

		const res = await spl.transfer(
			connection,
			minterWallet,
			minterWalletAta,
			toAta.address,
			minterWallet,
			BigInt(transferAmount.toString())
		);
		return res;
	} catch (err) {
		throw new Error(`[topUpSure] error. Cause: ${err}`);
	}
};

/**
 * topUpVeSure allows users to lock their tokens
 * @param param0
 */
export const topUpVeSure = async <T extends anchor.Idl>({
	program,
	tribecaSDK,
	sureLocker,
	governor,
	mint,
	voter,
	amount,
}: {
	program: anchor.Program<T>;
	tribecaSDK: tribeca.TribecaSDK;
	sureLocker: web3.PublicKey;
	governor: web3.PublicKey;
	mint: web3.PublicKey;
	voter: web3.Keypair;
	amount: number;
}) => {
	try {
		const lockerWrapper = await tribeca.LockerWrapper.load(
			tribecaSDK,
			sureLocker,
			governor
		);
		const sureToLock = await convertSureTokensToDecimals({
			connection: program.provider.connection,
			tokenMint: mint,
			amount,
		});
		const voterAccount = await spl.getAssociatedTokenAddress(
			mint,
			voter.publicKey
		);
		const voter1SureBalance = new anchor.BN(
			(
				await spl.getAccount(program.provider.connection, voterAccount)
			).amount.toString()
		);
		console.log('Voter1 Sure balance: ', voter1SureBalance.toString());
		console.log(
			'Voter1 wants to lock: ',
			sureToLock.toString(),
			'sures , which leaves her with ',
			voter1SureBalance.sub(sureToLock).toString()
		);
		const escrowRes = await lockerWrapper.getOrCreateEscrow(voter.publicKey);
		const transactionEnvelope = await lockerWrapper.lockTokens({
			amount: sureToLock,
			duration: new anchor.BN(365 * 60 * 60 * 24),
			authority: voter.publicKey,
		});
		await transactionEnvelope.addSigners(voter).confirm();
	} catch (err) {
		console.log('err: ', err);
		throw new Error(`[topUpVeSure] Failed to escrow tokens. Cause: ${err}`);
	}
};

export const createVoteHash = ({
	vote,
	salt,
}: {
	vote: anchor.BN;
	salt: Buffer;
}): Buffer => {
	const hash = new SHA3(256);
	const voteCandidate = vote.toString() + salt.toString('utf8');
	hash.update(voteCandidate);
	return hash.digest();
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
	const mint = await spl.getMint(connection, tokenMint);
	return new anchor.BN(amount).mul(
		new anchor.BN(10).pow(new anchor.BN(mint.decimals))
	);
};

export const findConfigPDA = (
	tokenMint: web3.PublicKey,
	programId: web3.PublicKey
) => {
	return findProgramAddressSync(
		[Buffer.from('sure-oracle-config'), tokenMint.toBytes()],
		programId
	);
};

export const findProposalPDA = (id: Buffer, programId: web3.PublicKey) => {
	return findProgramAddressSync([Buffer.from('sure-oracle'), id], programId);
};

export const findRevealVoteArrayPDA = (
	id: Buffer,
	programId: web3.PublicKey
) => {
	return findProgramAddressSync(
		[Buffer.from('sure-oracle-reveal-array'), id],
		programId
	);
};

export const findProposalVaultPDA = (id: Buffer, programId: web3.PublicKey) => {
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

export const findVoteAccount = ({
	proposal,
	voter,
	programId,
}: {
	proposal: web3.PublicKey;
	voter: web3.PublicKey;
	programId: web3.PublicKey;
}) => {
	return findProgramAddressSync(
		[Buffer.from(SURE_ORACLE_VOTE_SEED), proposal.toBuffer(), voter.toBuffer()],
		programId
	);
};

/**
 * createProposal
 * @param param0
 */
export const createProposal = async ({
	id,
	sureMint,
	program,
	proposer,
}: {
	id: Buffer;
	sureMint: web3.PublicKey;
	program: anchor.Program<Oracle>;
	proposer: web3.Signer;
}) => {
	// inputs

	const name = 'test123';
	const description = 'This is a test proposal';
	const stake = new anchor.BN(10).mul(new anchor.BN(1000000));

	// get necessary accounts
	try {
		const proposerAta = await spl.getAssociatedTokenAddress(
			sureMint,
			proposer.publicKey
		);
		const [configPda] = findConfigPDA(sureMint, program.programId);
		const [proposalPda] = findProposalPDA(id, program.programId);
		const [revealVoteArray] = findRevealVoteArrayPDA(id, program.programId);
		const [proposalVault] = findProposalVaultPDA(id, program.programId);
		let tx = new web3.Transaction();
		const instruction = await program.methods
			.proposeVote(id, name, description, stake)
			.accounts({
				proposer: proposer.publicKey,
				config: configPda,
				proposal: proposalPda,
				revealVoteArray: revealVoteArray,
				proposalVault,
				proposerAccount: proposerAta,
				proposalVaultMint: sureMint,
			})
			.instruction();
		tx.add(instruction);
		const signature = await web3.sendAndConfirmTransaction(
			program.provider.connection,
			tx,
			[proposer]
		);

		// check proposal
		const proposal = await program.account.proposal.fetch(proposalPda);
		// start vote automatically
		assert.equal(proposal.status, 2);
	} catch (err) {
		console.log('err: ', err);
		throw new Error('Could not create proposal. Cause ' + err);
	}
};

/**
 * voteOnProposal
 * @param param0
 */
export const voteOnProposal = async ({
	voter,
	proposalId,
	program,
	escrow,
	mint,
	locker,
}: {
	voter: web3.Keypair;
	proposalId: Buffer;
	program: anchor.Program<Oracle>;
	escrow: web3.PublicKey;
	mint: web3.PublicKey;
	locker: web3.PublicKey;
}) => {
	const voteHash = createVoteHash({
		vote: new anchor.BN(1),
		salt: Buffer.from('0'),
	});
	const [proposalPda] = findProposalPDA(proposalId, program.programId);
	const [proposalVault] = findProposalVaultPDA(proposalId, program.programId);
	const [voteAccount] = findVoteAccount({
		proposal: proposalPda,
		voter: voter.publicKey,
		programId: program.programId,
	});
	const voterAccount = await spl.getAssociatedTokenAddress(
		mint,
		voter.publicKey
	);
	const tx = new web3.Transaction();
	const ix = await program.methods
		.submitVote(voteHash)
		.accounts({
			voter: voter.publicKey,
			voterAccount,
			locker,
			userEscrow: escrow,
			proposal: proposalPda,
			proposalVault: proposalVault,
			proposalVaultMint: mint,
			voteAccount: voteAccount,
		})
		.instruction();
	tx.add(ix);
	const voteResult = await web3.sendAndConfirmTransaction(
		program.provider.connection,
		tx,
		[voter]
	);
};
