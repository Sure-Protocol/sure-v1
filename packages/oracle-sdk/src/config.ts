import * as anchor from '@project-serum/anchor';
import * as token_utils from '@saberhq/token-utils';
import { PublicKey, TransactionInstruction } from '@solana/web3.js';
import * as oracleIDL from '../../idls/oracle';
import { SURE_TOKEN } from './constants';
import { SureOracleSDK } from './sdk';
import { TransactionEnvelope } from '@saberhq/solana-contrib';
import { createProposalHash, validateKeys } from './utils';
import { getATAAddressSync } from '@saberhq/token-utils';
import { ProposalType } from './program';
import { ProgramAccount } from '@project-serum/anchor';

export type InitializeOracleConfig = {
	protocolAuthority: PublicKey;
	tokenMint: PublicKey;
};

export type UpdateConfig = {};
export class Config {
	readonly program: anchor.Program<oracleIDL.Oracle>;
	constructor(readonly sdk: SureOracleSDK) {
		this.program = sdk.program;
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

	//async updateConfig({}: UpdateConfig): Promise<TransactionEnvelope> {}

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
