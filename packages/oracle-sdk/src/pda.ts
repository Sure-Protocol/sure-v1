import * as anchor from '@project-serum/anchor';
import { findProgramAddressSync } from '@project-serum/anchor/dist/cjs/utils/pubkey';
import { PublicKey } from '@solana/web3.js';
import {
	SURE_ADDRESSES,
	SURE_ORACLE_REVEAL_ARRAY_SEED,
	SURE_ORACLE_SEED,
	SURE_ORACLE_VOTE_SEED,
} from './constants';

export class PDA {
	constructor() {}

	findProposalAddress(proposal_name: string): [PublicKey, number] {
		return findProgramAddressSync(
			[SURE_ORACLE_SEED, anchor.utils.bytes.utf8.encode(proposal_name)],
			SURE_ADDRESSES.Oracle
		);
	}

	findVoteAccount({
		proposal,
		voter,
	}: {
		proposal: PublicKey;
		voter: PublicKey;
	}) {
		return findProgramAddressSync(
			[SURE_ORACLE_VOTE_SEED, proposal.toBuffer(), voter.toBuffer()],
			SURE_ADDRESSES.Oracle
		);
	}

	findRevealVoteArrayAddress(proposal_name: string): [PublicKey, number] {
		return findProgramAddressSync(
			[
				SURE_ORACLE_REVEAL_ARRAY_SEED,
				anchor.utils.bytes.utf8.encode(proposal_name),
			],
			SURE_ADDRESSES.Oracle
		);
	}

	findProposalVault(mint: PublicKey): [PublicKey, number] {
		return findProgramAddressSync(
			[SURE_ORACLE_SEED, mint.toBuffer()],
			SURE_ADDRESSES.Oracle
		);
	}
}
