import * as anchor from '@project-serum/anchor';
import * as solana_contrib from '@saberhq/solana-contrib';
import * as token_utils from '@saberhq/token-utils';
import { findProgramAddressSync } from '@project-serum/anchor/dist/cjs/utils/pubkey';
import {
	Keypair,
	PublicKey,
	SystemProgram,
	Transaction,
	TransactionInstruction,
} from '@solana/web3.js';
import { OracleIDL, OracleJSON } from '../../idls/oracle';
import {
	SURE_ADDRESSES,
	SURE_ORACLE_REVEAL_ARRAY_SEED,
	SURE_ORACLE_SEED,
	SURE_TOKEN,
} from './constants';
import { Provider, SureOracleSDK } from './sdk';
import { OracleProgram } from './program';
import { getOrCreateAssociatedTokenAccount } from '@solana/spl-token/lib/types/actions/getOrCreateAssociatedTokenAccount';
import { TOKEN_PROGRAM_ID } from '@solana/spl-token';
import { TransactionEnvelope } from '@saberhq/solana-contrib';
import { validateKeys } from './utils';

// ================== Types ==================
type ProposeVote = {
	name: string;
	description: string;
	stake: anchor.BN;
	mint?: PublicKey;
};

type TransactionInformation = {
	address: PublicKey;
	envelope: TransactionEnvelope;
};

// ================= PDAs ====================
export const findProposalAddress = async (
	proposal_name: string
): Promise<[PublicKey, number]> => {
	return await findProgramAddressSync(
		[SURE_ORACLE_SEED, anchor.utils.bytes.utf8.encode(proposal_name)],
		SURE_ADDRESSES.Oracle
	);
};

export const findRevealVoteArrayAddress = async (
	proposal_name: string
): Promise<[PublicKey, number]> => {
	return await findProgramAddressSync(
		[
			SURE_ORACLE_REVEAL_ARRAY_SEED,
			anchor.utils.bytes.utf8.encode(proposal_name),
		],
		SURE_ADDRESSES.Oracle
	);
};

export const findProposalVault = async (
	mint: PublicKey
): Promise<[PublicKey, number]> => {
	return await findProgramAddressSync(
		[SURE_ORACLE_SEED, mint.toBuffer()],
		SURE_ADDRESSES.Oracle
	);
};

export class Proposal {
	readonly program: anchor.Program<OracleIDL>;
	constructor(readonly sdk: SureOracleSDK) {
		this.program = sdk.oracle;
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
		ixs.push(proposerAccount.instruction);
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
}
