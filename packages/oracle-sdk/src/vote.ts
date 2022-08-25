import * as anchor from '@project-serum/anchor';
import { SHA3 } from 'sha3';
import { findProgramAddressSync } from '@project-serum/anchor/dist/cjs/utils/pubkey';
import * as solana_contrib from '@saberhq/solana-contrib';
import * as token_utils from '@saberhq/token-utils';
import { PublicKey, TransactionInstruction } from '@solana/web3.js';
import { OracleIDL } from '../../idls/oracle';
import { randomBytes } from 'crypto';
import {
	SURE_ADDRESSES,
	SURE_ORACLE_REVEAL_ARRAY_SEED,
	SURE_ORACLE_VOTE_SEED,
	SURE_TOKEN,
} from './constants';
import { OracleProgram } from './program';
import { findProposalVault } from './proposal';
import { Provider, SureOracleSDK } from './sdk';
import { validateKeys } from './utils';

type SubmitVote = {
	vote: anchor.BN;
	mint?: PublicKey;
	locker: PublicKey;
	userEscrow: PublicKey;
	proposal: PublicKey;
};

export const findVoteAccount = async ({
	proposal,
	voter,
}: {
	proposal: PublicKey;
	voter: PublicKey;
}) => {
	return await findProgramAddressSync(
		[SURE_ORACLE_VOTE_SEED, proposal.toBuffer(), voter.toBuffer()],
		SURE_ADDRESSES.Oracle
	);
};

export const createVoteHash = ({ vote }: { vote: anchor.BN }): Buffer => {
	const hash = new SHA3(256);
	const salt = randomBytes(16);
	const voteCandidate = vote.toString() + salt.toString('utf8');
	hash.update(voteCandidate);
	return hash.digest();
};

export class Vote {
	readonly program: anchor.Program<OracleIDL>;
	constructor(readonly sdk: SureOracleSDK) {
		this.program = sdk.oracle;
	}

	/**
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
	}: SubmitVote): Promise<solana_contrib.TransactionEnvelope> {
		const tokenMint = mint ?? SURE_TOKEN;
		validateKeys([
			{ v: tokenMint, n: 'tokenMint' },
			{ v: proposal, n: 'proposal' },
			{ v: locker, n: 'lcoker' },
			{ v: userEscrow, n: 'escrow' },
		]);

		const voteHash = createVoteHash({ vote });
		console.log('byte length: ', voteHash.byteLength);
		console.log('', voteHash.toLocaleString().length);
		console.log('', voteHash.toString().length);
		let ixs: TransactionInstruction[] = [];
		const createATA = await token_utils.getOrCreateATA({
			provider: this.sdk.provider,
			mint,
		});
		const [proposalVault] = await findProposalVault(tokenMint);
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
		return this.sdk.provider.newTX(ixs);
	}
}
