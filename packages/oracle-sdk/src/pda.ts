import * as anchor from '@project-serum/anchor';
import { PublicKey } from '@solana/web3.js';
import {
	SURE_ADDRESSES,
	SURE_ORACLE_CONFIG_SEED,
	SURE_ORACLE_REVEAL_ARRAY_SEED,
	SURE_ORACLE_SEED,
	SURE_ORACLE_VOTE_SEED,
} from './constants';
import { createProposalHash } from './utils';

export class PDA {
	constructor() {}

	findProposalAddress({
		proposalName,
	}: {
		proposalName: string;
	}): [PublicKey, number] {
		const id = createProposalHash({ name: proposalName });
		return anchor.utils.publicKey.findProgramAddressSync(
			[SURE_ORACLE_SEED, id],
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
		return anchor.utils.publicKey.findProgramAddressSync(
			[SURE_ORACLE_VOTE_SEED, proposal.toBuffer(), voter.toBuffer()],
			SURE_ADDRESSES.Oracle
		);
	}

	findRevealVoteArrayAddress({
		proposal,
	}: {
		proposal: PublicKey;
	}): [PublicKey, number] {
		return anchor.utils.publicKey.findProgramAddressSync(
			[SURE_ORACLE_REVEAL_ARRAY_SEED, proposal.toBuffer()],
			SURE_ADDRESSES.Oracle
		);
	}

	findProposalVault({
		proposal,
	}: {
		proposal: PublicKey;
	}): [PublicKey, number] {
		return anchor.utils.publicKey.findProgramAddressSync(
			[SURE_ORACLE_SEED, proposal.toBuffer()],
			SURE_ADDRESSES.Oracle
		);
	}

	findOracleConfig({
		tokenMint,
	}: {
		tokenMint: PublicKey;
	}): [PublicKey, number] {
		return anchor.utils.publicKey.findProgramAddressSync(
			[SURE_ORACLE_CONFIG_SEED, tokenMint.toBuffer()],
			SURE_ADDRESSES.Oracle
		);
	}
}
