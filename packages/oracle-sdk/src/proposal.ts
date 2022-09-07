import * as anchor from '@project-serum/anchor';
import { PublicKey, TransactionInstruction } from '@solana/web3.js';
import * as oracleIDL from './idls/oracle';
import { SURE_MINT } from './constants';
import { SureOracleSDK } from './sdk';
import { TransactionEnvelope } from '@saberhq/solana-contrib/dist/cjs';
import {
	createProposalHash,
	getOrCreateAssociatedTokenAccountIx,
	validateKeys,
} from './utils';
import { ProposalType } from './program';
import { ProgramAccount } from '@project-serum/anchor';
import NodeWallet from '@project-serum/anchor/dist/cjs/nodewallet';
import { getAssociatedTokenAddress } from '@solana/spl-token';

// ================== Types ==================
type ProposeVote = {
	name: string;
	description: string;
	stake: anchor.BN;
	mint?: PublicKey;
};

type FinalizeVoteResults = {
	proposal: PublicKey;
};

type CollectProposerReward = {
	proposal: PublicKey;
	tokenMint: PublicKey;
};

export class Proposal {
	readonly program: anchor.Program<oracleIDL.Oracle>;
	constructor(readonly sdk: SureOracleSDK) {
		this.program = sdk.program;
	}

	async fetchAllProposals(): Promise<ProgramAccount<ProposalType>[]> {
		return await this.program.account.proposal.all();
	}

	async fetchProposal({ name }: { name: string }): Promise<ProposalType> {
		const [proposalPDA] = this.sdk.pda.findProposalAddress({
			proposalName: name,
		});
		return await this.program.account.proposal.fetch(proposalPDA);
	}

	/**
	 * propose vote
	 *
	 * propose a vote
	 *
	 * @param name - name of vote
	 * @param description - description of vote
	 * @param stake - the amount of stake the user bets on the vote
	 * @param mint <optional> - SURE MINT if nothing specified
	 * @returns
	 */
	async proposeVote({
		name,
		description,
		stake,
		mint,
	}: ProposeVote): Promise<TransactionEnvelope> {
		const tokenMint = mint ?? SURE_MINT;
		validateKeys([{ v: tokenMint, n: 'mint' }]);
		if (name.length == 0) {
			throw new Error('proposal name cannot be empty');
		}

		if (description.length == 0) {
			throw new Error('proposal description cannot be empty');
		}

		const proposerAccount = await getOrCreateAssociatedTokenAccountIx({
			connection: this.sdk.provider.connection,
			payer: (this.sdk.provider.wallet as NodeWallet).payer,
			mint: tokenMint,
			owner: this.sdk.provider.walletKey,
		});
		const ixs: TransactionInstruction[] = [];
		if (proposerAccount.instruction) {
			ixs.push(proposerAccount.instruction);
		}

		const id = createProposalHash({ name });
		const [config] = this.sdk.pda.findOracleConfig({ tokenMint });
		const [proposal] = SureOracleSDK.pda().findProposalAddress({
			proposalName: name,
		});
		ixs.push(
			await this.program.methods
				.proposeVote(id, name, description, stake)
				.accounts({
					config,
					proposal,
					proposerAccount: proposerAccount.address,
					proposalVaultMint: tokenMint,
				})
				.instruction()
		);
		return this.sdk.provider.newTX(ixs);
	}

	/**
	 * finalize vote resul ts
	 *
	 * @param proposal - the proposal PK
	 * @returns TransactionEnvelope - a new transaction
	 */
	async finalizeVoteResults({
		proposal,
	}: FinalizeVoteResults): Promise<TransactionEnvelope> {
		const [voteArray] = this.sdk.pda.findRevealVoteArrayAddress({ proposal });

		const ixs: TransactionInstruction[] = [];
		ixs.push(
			await this.program.methods
				.finalizeVoteResults()
				.accounts({
					finalizer: this.sdk.provider.wallet.publicKey,
					proposal,
					revealedVotes: voteArray,
				})
				.instruction()
		);

		return this.sdk.provider.newTX(ixs);
	}

	/**
	 * collect proposer rewards
	 *
	 * @param proposal - the proposal PK
	 * @returns TransactionEnvelope - a new transaction
	 */
	async collectProposerRewards({
		proposal,
		tokenMint,
	}: CollectProposerReward): Promise<TransactionEnvelope> {
		const proposerTokenAccount = await getAssociatedTokenAddress(
			tokenMint,
			this.sdk.provider.wallet.publicKey
		);

		const [config] = this.sdk.pda.findOracleConfig({ tokenMint });
		const [proposalVault] = this.sdk.pda.findProposalVault({ proposal });
		const ixs: TransactionInstruction[] = [];
		ixs.push(
			await this.program.methods
				.collectProposerReward()
				.accounts({
					config,
					proposerTokenAccount,
					proposal,
					proposalVault,
					proposalVaultMint: tokenMint,
				})
				.instruction()
		);
		return this.sdk.provider.newTX(ixs);
	}
}
