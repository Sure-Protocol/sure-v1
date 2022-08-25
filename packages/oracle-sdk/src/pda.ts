import * as anchor from '@project-serum/anchor';
import { findProgramAddressSync } from '@project-serum/anchor/dist/cjs/utils/pubkey';
import { PublicKey } from '@solana/web3.js';
import { SURE_ADDRESSES, SURE_ORACLE_SEED } from './constants';

export class PDA {
	constructor() {}

	findProposalAddress(proposal_name: string): [PublicKey, number] {
		return findProgramAddressSync(
			[SURE_ORACLE_SEED, anchor.utils.bytes.utf8.encode(proposal_name)],
			SURE_ADDRESSES.Oracle
		);
	}
}
