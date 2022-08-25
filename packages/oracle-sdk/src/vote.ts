import * as anchor from '@project-serum/anchor';
import { SHA3 } from 'sha3';
import { findProgramAddressSync } from '@project-serum/anchor/dist/cjs/utils/pubkey';
import * as solana_contrib from '@saberhq/solana-contrib';
import * as token_utils from '@saberhq/token-utils';
import {
	PublicKey,
	Transaction,
	TransactionInstruction,
} from '@solana/web3.js';
import * as oracleIDL from '../../idls/oracle';
import { randomBytes } from 'crypto';
import {
	SURE_ADDRESSES,
	SURE_ORACLE_REVEAL_ARRAY_SEED,
	SURE_ORACLE_VOTE_SEED,
	SURE_TOKEN,
} from './constants';
import { OracleProgram } from './program';
import { Provider, SureOracleSDK } from './sdk';
import { validateKeys } from './utils';
import { TransactionEnvelope } from '@saberhq/solana-contrib';

type SubmitVote = {
	vote: anchor.BN;
	mint?: PublicKey;
	locker: PublicKey;
	userEscrow: PublicKey;
	proposal: PublicKey;
};

type UpdateVote = {
	vote: anchor.BN;
	proposal: PublicKey;
};

type CancelVote = {
	voteAccount: PublicKey;
};

type RevealVote = {
	voteAccount: PublicKey;
	vote: anchor.BN;
	salt: Buffer;
};

type VoteTransactionEnvelope = {
	salt: Buffer;
	transactionEnvelope: TransactionEnvelope;
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

export const revealVote = ({
	expectedVoteHash,
	vote,
	salt,
}: {
	expectedVoteHash: number[];
	vote: anchor.BN;
	salt: Buffer;
}): Boolean => {
	const expectedVoteHashB = Buffer.from(expectedVoteHash);
	const voteHash = createVoteHash({ vote, salt });
	return voteHash.equals(expectedVoteHashB);
};

export class Vote {
	readonly program: anchor.Program<oracleIDL.Oracle>;
	constructor(readonly sdk: SureOracleSDK) {
		this.program = sdk.program;
	}

	/**
	 * submit a vote to a proposal
	 *
	 * @param mint - mint of proposal vault
	 * @param proposal - the proposal to vote on
	 * @param locker - locker used to lock tokens, see Tribeca
	 * @param userEscrow - escrow that holds the locked tokens
	 * @returns
	 */
	async submitVote({
		vote,
		mint,
		proposal,
		locker,
		userEscrow,
	}: SubmitVote): Promise<VoteTransactionEnvelope> {
		const tokenMint = mint ?? SURE_TOKEN;
		validateKeys([
			{ v: tokenMint, n: 'tokenMint' },
			{ v: proposal, n: 'proposal' },
			{ v: locker, n: 'lcoker' },
			{ v: userEscrow, n: 'escrow' },
		]);

		const salt = randomBytes(16);
		const voteHash = createVoteHash({ vote, salt });
		let ixs: TransactionInstruction[] = [];
		const createATA = await token_utils.getOrCreateATA({
			provider: this.sdk.provider,
			mint,
		});
		const [proposalVault] = await this.sdk.pda.findProposalVault(tokenMint);
		ixs.push(createATA.instruction);
		ixs.push(
			await this.program.methods
				.submitVote(voteHash)
				.accounts({
					voterAccount: createATA.address,
					locker,
					userEscrow,
					proposal,
					proposalVault: proposalVault,
					proposalVaultMint: tokenMint,
				})
				.instruction()
		);
		return {
			salt: salt,
			transactionEnvelope: this.sdk.provider.newTX(ixs),
		};
	}

	/**
	 * update vote
	 *
	 * @param mint - mint of proposal vault
	 * @param proposal - the proposal to vote on
	 * @returns
	 */
	async updateVote({
		vote,
		proposal,
	}: UpdateVote): Promise<VoteTransactionEnvelope> {
		validateKeys([{ v: proposal, n: 'proposal' }]);
		const salt = randomBytes(16);
		const voteHash = createVoteHash({ vote, salt });

		const voter = this.sdk.provider.wallet.publicKey;

		const [voteAccount] = await this.sdk.pda.findVoteAccount({
			proposal,
			voter,
		});
		let ixs: TransactionInstruction[] = [];
		ixs.push(
			await this.program.methods
				.updateVote(voteHash)
				.accounts({
					proposal,
					voteAccount,
				})
				.instruction()
		);
		return {
			salt: salt,
			transactionEnvelope: this.sdk.provider.newTX(ixs),
		};
	}

	/**
	 * cancel vote
	 *
	 * @param voteAccout - the account used to vote with
	 * @returns
	 */
	async cancelVote({ voteAccount }: CancelVote): Promise<TransactionEnvelope> {
		validateKeys([{ v: voteAccount, n: 'voteAccount' }]);

		const voter = this.sdk.provider.wallet.publicKey;
		const voteAccountLoaded = await this.program.account.voteAccount.fetch(
			voteAccount
		);
		const stakeMint = voteAccountLoaded.stakeMint;

		const [proposalVault] = await this.sdk.pda.findProposalVault(stakeMint);
		const voterAccount = await token_utils.getATAAddressSync({
			mint: stakeMint,
			owner: voter,
		});

		let ixs: TransactionInstruction[] = [];
		ixs.push(
			await this.program.methods
				.cancelVote()
				.accounts({
					voterAccount: voterAccount,
					proposalVault,
					proposalVaultMint: stakeMint,
					proposal: voteAccountLoaded.proposal,
					voteAccount,
				})
				.instruction()
		);
		return this.sdk.provider.newTX(ixs);
	}

	/**
	 * cancel vote
	 *
	 * @param voteAccout - the account used to vote with
	 * @returns
	 */
	async revealVote({
		voteAccount,
		vote,
		salt,
	}: RevealVote): Promise<TransactionEnvelope> {
		validateKeys([{ v: voteAccount, n: 'voteAccount' }]);

		console.log('voteAccount: ', voteAccount.toString());
		const voteAccountLoaded = await this.program.account.voteAccount.fetch(
			voteAccount
		);
		const proposal = voteAccountLoaded.proposal;

		const [voteArray] = await this.sdk.pda.findRevealVoteArrayAddress({
			proposal,
		});
		console.log('voteArray: ', voteArray);

		let ixs: TransactionInstruction[] = [];
		ixs.push(
			await this.program.methods
				.revealVote(salt.toString(), vote)
				.accounts({
					proposal,
					revealVoteArray: voteArray,
					voteAccount,
				})
				.instruction()
		);
		return this.sdk.provider.newTX(ixs);
	}
}
