import * as anchor from '@project-serum/anchor';
import { PublicKey, TransactionInstruction } from '@solana/web3.js';
import { Oracle } from './idls/oracle.js';
import { SureOracleSDK } from './sdk.js';
import { TransactionEnvelope } from '@saberhq/solana-contrib';
import { ConfigType } from './program.js';
import { getOrCreateAssociatedTokenAccountIx } from './utils.js';

export type InitializeOracleConfig = {
	protocolAuthority: PublicKey;
	tokenMint: PublicKey;
};

export type UpdateConfig = {
	proposalPk: PublicKey;
	votingPeriod?: anchor.BN;
	revealPeriod?: anchor.BN;
	requiredVotes?: anchor.BN;
	minimumProposalStake?: anchor.BN;
	voteStakeRate?: number;
	protocolFeeRate?: number;
};
export class Config {
	readonly program: anchor.Program<Oracle>;
	constructor(readonly sdk: SureOracleSDK) {
		this.program = sdk.program;
	}

	async fetchConfig({
		tokenMint,
	}: {
		tokenMint: PublicKey;
	}): Promise<ConfigType> {
		const [config] = this.sdk.pda.findOracleConfig({
			tokenMint,
		});
		return this.program.account.config.fetch(config);
	}

	/**
	 * initialize oracle config
	 *
	 * initialize
	 *
	 * @param protocolAuthority - owner of the config
	 * @param tokenMint - mint of vault
	 */
	async initializeOracleConfig({
		protocolAuthority,
		tokenMint,
	}: InitializeOracleConfig): Promise<TransactionEnvelope> {
		const ixs: TransactionInstruction[] = [];
		ixs.push(
			await this.program.methods
				.initializeConfig(protocolAuthority)
				.accounts({
					tokenMint,
				})
				.instruction()
		);
		return this.sdk.provider.newTX(ixs);
	}

	/**
	 * update oracle config
	 *
	 * @param proposalPk - public key of proposal
	 * @param votingPeriod - the voting period in seconds
	 * @param revealPeriod - the period for which the users can reveal their vote in seconds
	 * @param requiredVotes - the number of required voted to reach quorum
	 * @param minimumProposalStake - the minimum amount of stake a proposer needs to put into the vault (escrow)
	 * @param voteStakeRate - the 1/x of total voting power the user have to deposit to vote
	 * @param protocolFeeRate - the 1/x the protocol will extract from the voting pool
	 *
	 * @returns TransactionEnvelope
	 */
	async updateConfigInstructions({
		proposalPk,
		votingPeriod,
		revealPeriod,
		requiredVotes,
		minimumProposalStake,
		voteStakeRate,
		protocolFeeRate,
	}: UpdateConfig): Promise<TransactionInstruction[]> {
		const ixs: TransactionInstruction[] = [];
		const proposal = await this.sdk.program.account.proposal.fetch(proposalPk);

		if (votingPeriod && this.sdk.program) {
			ixs.push(
				await this.sdk.program.methods
					.updateVotingPeriod(votingPeriod)
					.accounts({
						config: proposal.config,
					})
					.instruction()
			);
		}

		if (revealPeriod) {
			ixs.push(
				await this.sdk.program.methods
					.updateRevealPeriod(revealPeriod)
					.accounts({
						config: proposal.config,
					})
					.instruction()
			);
		}

		if (requiredVotes) {
			ixs.push(
				await this.sdk.program.methods
					.updateRequiredVotes(requiredVotes)
					.accounts({
						config: proposal.config,
					})
					.instruction()
			);
		}

		if (minimumProposalStake) {
			ixs.push(
				await this.sdk.program.methods
					.updateProposalMinimumStake(minimumProposalStake)
					.accounts({
						config: proposal.config,
					})
					.instruction()
			);
		}

		if (voteStakeRate) {
			ixs.push(
				await this.sdk.program.methods
					.updateVoteStakeRate(voteStakeRate)
					.accounts({
						config: proposal.config,
					})
					.instruction()
			);
		}

		if (protocolFeeRate) {
			ixs.push(
				await this.sdk.program.methods
					.updateProtocolFeeRate(protocolFeeRate)
					.accounts({
						config: proposal.config,
					})
					.instruction()
			);
		}

		return ixs;
	}

	/**
	 * collect protocol fees
	 *
	 *
	 * @params tokenMint - the mint of the vault
	 * @params proposalName - name of the proposal
	 * @params feeDestination - the token account for where to send the fees
	 */
	async collectProtocolFees({
		tokenMint,
		proposalName,
		feeDestination,
	}: {
		tokenMint: PublicKey;
		proposalName: string;
		feeDestination: PublicKey;
	}): Promise<TransactionEnvelope> {
		const [config] = this.sdk.pda.findOracleConfig({ tokenMint });
		const [proposal] = this.sdk.pda.findProposalAddress({ proposalName });
		const [proposalVault] = this.sdk.pda.findProposalVault({ proposal });

		const proposerAccount = await getOrCreateAssociatedTokenAccountIx({
			connection: this.sdk.provider.connection,
			payer: (this.sdk.provider.wallet as anchor.Wallet).payer,
			mint: tokenMint,
			owner: this.sdk.provider.walletKey,
		});
		const ixs: TransactionInstruction[] = [];
		if (proposerAccount.instruction) {
			ixs.push(proposerAccount.instruction);
		}

		ixs.push(
			await this.program.methods
				.collectProtocolFees()
				.accounts({
					config,
					proposal,
					proposalVault,
					feeDestination,
				})
				.instruction()
		);
		return this.sdk.provider.newTX(ixs);
	}
}
