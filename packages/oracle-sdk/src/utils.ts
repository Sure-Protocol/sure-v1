import { PublicKey } from '@solana/web3.js';
import { BN } from '@project-serum/anchor';
import { ProposalType } from './program';

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
	const currentTime = new BN(Math.floor(Date.now() / 1000));
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
	currentTime: BN
): Boolean => {
	return (
		currentTime >= proposal.voteStartAt && currentTime < proposal.voteEndAt
	);
};

const isBlindVoteFinished = (
	proposal: ProposalType,
	currentTime: BN
): Boolean => {
	return (
		currentTime >= proposal.voteEndAt && currentTime < proposal.voteEndRevealAt
	);
};

const isRevealVoteFinished = (
	proposal: ProposalType,
	currentTime: BN
): Boolean => {
	return currentTime >= proposal.voteEndRevealAt;
};
