import * as anchor from '@project-serum/anchor';
import { findProgramAddressSync } from '@project-serum/anchor/dist/cjs/utils/pubkey';
import { PublicKey } from '@solana/web3.js';
import { OracleIDL, OracleJSON } from '../../idls/oracle';
import { SURE_ADDRESSES, SURE_ORACLE_SEED } from './constants';
import { Provider, SureOracleSDK } from './sdk';

// ================== Types ==================
type ProposeVote = {
	name: string;
	description: string;
	stake: anchor.BN;
};

// ================= PDAs ====================
export const findProposalAddress = async (): Promise<[PublicKey, number]> => {
	return await findProgramAddressSync(
		[SURE_ORACLE_SEED],
		SURE_ADDRESSES.Oracle
	);
};

export class Proposal {
	readonly program: anchor.Program<OracleIDL>;
	constructor(readonly sdk: SureOracleSDK) {
		this.program = sdk.Oracle;
	}

	// Propose vote
	// checkpoint : finish propose vote methods
	async propose_vote({
		name,
		description,
		stake,
	}: ProposeVote): Promise<number> {
		// await this.program.methods.proposeVote(name, description, stake).accounts({
		// 	propose,
		// });
		return 0;
	}
}
