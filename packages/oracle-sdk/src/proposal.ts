import * as anchor from '@project-serum/anchor';
import * as token_utils from '@saberhq/token-utils';
import { PublicKey, TransactionInstruction } from '@solana/web3.js';
import * as oracleIDL from '../../idls/oracle';
import { SURE_TOKEN } from './constants';
import { SureOracleSDK } from './sdk';
import { TransactionEnvelope } from '@saberhq/solana-contrib';
import { validateKeys } from './utils';
import { getATAAddressSync } from '@saberhq/token-utils';

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

type CollectProposerReward = FinalizeVoteResults;

export class Proposal {
	readonly program: anchor.Program<oracleIDL.Oracle>;
	constructor(readonly sdk: SureOracleSDK) {
		this.program = sdk.program;
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
		const tokenMint = mint ?? SURE_TOKEN;
		validateKeys([{ v: tokenMint, n: 'mint' }]);
		if (name.length == 0) {
			throw new Error('proposal name cannot be empty');
		}

		if (description.length == 0) {
			throw new Error('proposal description cannot be empty');
		}

		const proposerAccount = await token_utils.getOrCreateATA({
			provider: this.sdk.provider,
			mint: tokenMint,
		});
		const ixs: TransactionInstruction[] = [];
		if (proposerAccount.instruction) {
			ixs.push(proposerAccount.instruction);
		}
		ixs.push(
			await this.program.methods
				.proposeVote(name, description, stake)
				.accounts({
					proposerAccount: proposerAccount.address,
					proposalVaultMint: tokenMint,
				})
				.instruction()
		);
		return this.sdk.provider.newTX(ixs);
	}

	/**
	 * finalize vote results
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
	}: CollectProposerReward): Promise<TransactionEnvelope> {
		const proposalAccount = await this.program.account.proposal.fetch(proposal);

		const proposerTokenAccount = getATAAddressSync({
			mint: proposalAccount.vaultMint,
			owner: this.sdk.provider.wallet.publicKey,
		});

		const [proposalVault] = this.sdk.pda.findProposalVault({ proposal });
		const ixs: TransactionInstruction[] = [];
		ixs.push(
			await this.program.methods
				.collectProposerReward()
				.accounts({
					proposerTokenAccount,
					proposal,
					proposalVault,
					proposalVaultMint: proposalAccount.vaultMint,
				})
				.instruction()
		);
		return this.sdk.provider.newTX(ixs);
	}
}
