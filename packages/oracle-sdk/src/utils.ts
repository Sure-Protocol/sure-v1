import {
	Commitment,
	Connection,
	PublicKey,
	Signer,
	TransactionInstruction,
} from '@solana/web3.js';
import { ProposalType } from './program.js';
import * as anchor from '@project-serum/anchor';
import { SHAKE } from 'sha3';
import * as spl from '@solana/spl-token';

export const validateKeys = (keys: { v: PublicKey; n: string }[]) => {
	const undefinedErrors = keys
		.filter((k) => k.v === undefined)
		.map((k) => `${k.n} is undefined.`)
		.join(', ');
	if (undefinedErrors.length > 0) {
		throw new Error(undefinedErrors);
	}
};

export type ProposalStatus =
	| 'Voting'
	| 'Reveal'
	| 'Creating reward distribution'
	| 'Calculate Reward'
	| 'Reward Payout'
	| 'Failed';

export const getProposalStatus = (proposal: ProposalType): ProposalStatus => {
	const currentTime = new anchor.BN(Math.floor(Date.now() / 1000));
	const hasReachedQuorum = proposal.votes >= proposal.requiredVotes;
	const isScaleParameterCalculated = proposal.scaleParameterCalculated;
	const isLocked = proposal.locked;
	if (isBlindVoteOngoing(proposal, currentTime) && !hasReachedQuorum) {
		return 'Voting';
	} else if (isBlindVoteFinished(proposal, currentTime) && hasReachedQuorum) {
		return 'Reveal';
	} else if (isRevealVoteFinished(proposal, currentTime) && hasReachedQuorum) {
		return 'Creating reward distribution';
	} else if (isScaleParameterCalculated) {
		return 'Calculate Reward';
	} else if (isLocked) {
		return 'Reward Payout';
	} else {
		return 'Failed';
	}
};

export type CauseOfFailedProposal = 'NotEnoughVotes' | 'Unknown';

export const proposalFailReason = (
	proposal: ProposalType
): CauseOfFailedProposal => {
	if (getProposalStatus(proposal) == 'Failed') {
		const hasReachedQuorum = proposal.votes >= proposal.requiredVotes;
		if (!hasReachedQuorum) {
			return 'NotEnoughVotes';
		}
	}
	return 'Unknown';
};

export type VoteStatus =
	| 'Voting'
	| 'Reveal vote'
	| 'Calculate Reward'
	| 'Collect Reward'
	| 'Failed';

export const getVoteStatus = (proposal: ProposalType): VoteStatus => {
	const propsalStatus = getProposalStatus(proposal);
	if (propsalStatus == 'Voting') {
		return 'Voting';
	} else if (propsalStatus == 'Reveal') {
		return 'Reveal vote';
	} else if (
		propsalStatus == 'Creating reward distribution' ||
		propsalStatus == 'Calculate Reward'
	) {
		return 'Calculate Reward';
	} else if (propsalStatus == 'Reward Payout') {
		return 'Collect Reward';
	} else {
		return 'Failed';
	}
};

const isBlindVoteOngoing = (
	proposal: ProposalType,
	currentTime: anchor.BN
): Boolean => {
	return (
		currentTime >= proposal.voteStartAt && currentTime < proposal.voteEndAt
	);
};

const isBlindVoteFinished = (
	proposal: ProposalType,
	currentTime: anchor.BN
): Boolean => {
	return (
		currentTime >= proposal.voteEndAt && currentTime < proposal.voteEndRevealAt
	);
};

const isRevealVoteFinished = (
	proposal: ProposalType,
	currentTime: anchor.BN
): Boolean => {
	return currentTime >= proposal.voteEndRevealAt;
};

export const createProposalHash = ({ name }: { name: string }): Buffer => {
	const hash = new SHAKE(128);
	hash.update(name);
	return hash.digest();
};

type ATAInput = {
	connection: Connection;
	payer: Signer;
	mint: PublicKey;
	owner: PublicKey;
	allowOwnerOffCurve?: boolean;
	commitment?: Commitment;
	programId?: PublicKey;
	associatedTokenProgramId?: PublicKey;
};

export const getOrCreateAssociatedTokenAccountIx = async ({
	connection,
	payer,
	mint,
	owner,
	allowOwnerOffCurve = false,
	commitment,
	programId = spl.TOKEN_PROGRAM_ID,
	associatedTokenProgramId = spl.ASSOCIATED_TOKEN_PROGRAM_ID,
}: ATAInput): Promise<{
	instruction: TransactionInstruction | null;
	address: PublicKey;
}> => {
	const associatedToken = await spl.getAssociatedTokenAddress(
		mint,
		owner,
		allowOwnerOffCurve,
		programId,
		associatedTokenProgramId
	);

	// This is the optimal logic, considering TX fee, client-side computation, RPC roundtrips and guaranteed idempotent.
	// Sadly we can't do this atomically.
	let account: spl.Account;
	try {
		account = await spl.getAccount(
			connection,
			associatedToken,
			commitment,
			programId
		);
		return {
			instruction: null,
			address: associatedToken,
		};
	} catch (error: unknown) {
		// TokenAccountNotFoundError can be possible if the associated address has already received some lamports,
		// becoming a system account. Assuming program derived addressing is safe, this is the only case for the
		// TokenInvalidAccountOwnerError in this code path.
		if (
			error instanceof spl.TokenAccountNotFoundError ||
			error instanceof spl.TokenInvalidOwnerError
		) {
			// As this isn't atomic, it's possible others can create associated accounts meanwhile.
			try {
				const transaction = new TransactionInstruction(
					spl.createAssociatedTokenAccountInstruction(
						payer.publicKey,
						associatedToken,
						owner,
						mint,
						programId,
						associatedTokenProgramId
					)
				);
				return {
					instruction: transaction,
					address: associatedToken,
				};
			} catch (error: unknown) {
				// Ignore all errors; for now there is no API-compatible way to selectively ignore the expected
				// instruction error if the associated account exists already.
			}

			// Now this should always succeed
			account = await spl.getAccount(
				connection,
				associatedToken,
				commitment,
				programId
			);
		} else {
			throw error;
		}
	}
	if (!account.mint.equals(mint)) throw new spl.TokenInvalidMintError();
	if (!account.owner.equals(owner)) throw new spl.TokenInvalidOwnerError();

	return {
		address: associatedToken,
		instruction: null,
	};
};
