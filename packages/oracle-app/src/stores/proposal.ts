import { writable } from 'svelte/store';
import {
	type SureOracleSDK,
	type ProposalType,
	type VoteStatus,
	getVoteStatus
} from '@surec/oracle';
import type { ProgramAccount } from '@saberhq/token-utils';
import { newEvent } from './event';
import { countdownFromUnix, getNextDeadline, isInFuture } from '$lib/utils';

// load proposals
export type ProposalsState = {
	locked: boolean;
	isLoading: boolean;
	loadingFailed: boolean;
	proposals: ProgramAccount<ProposalType>[] | null;
};

export const proposalsState = writable<ProposalsState>({
	locked: false,
	isLoading: false,
	loadingFailed: false,
	proposals: null
});

export const hydrateProposalCallback = async (
	fn: () => void,
	oracleSdk: SureOracleSDK | undefined
) => {
	if (oracleSdk) {
		try {
			await fn();
			await hydrateProposals(oracleSdk);
		} catch (err) {
			throw new Error(err as string);
		}
	} else {
		throw new Error('Not connected to program');
	}
};

export const hydrateProposals = async (oracleSdk: SureOracleSDK) => {
	proposalsState.set({
		locked: true,
		isLoading: true,
		loadingFailed: false,
		proposals: null
	});
	try {
		const proposals = await oracleSdk.proposal().fetchAllProposals();
		proposalsState.set({
			locked: false,
			isLoading: false,
			loadingFailed: false,
			proposals
		});
	} catch (err) {
		proposalsState.set({
			locked: false,
			isLoading: false,
			loadingFailed: true,
			proposals: null
		});
		newEvent.set({
			name: 'failed to get proposals',
			message: err as string,
			status: 'error'
		});
	}
};

// whether create proposal is open
export const createProposalState = writable(false, () => {
	console.log('subscribe');
	return () => console.log('unsubsribe');
});

// selected proposal
export const selectedProposal = writable<ProgramAccount<ProposalType> | undefined>(
	undefined,
	() => {
		console.log('Subsribe');
	}
);

// proposal steps

export type ProposalSteps = {
	steps: { status: VoteStatus; text: string }[];
	currentStep: number;
};

const initialProposalSteps: ProposalSteps = {
	steps: [
		{ status: 'Voting', text: 'Voting' },
		{ status: 'Reveal vote', text: 'Reveal vote' },
		{ status: 'Calculate Reward', text: 'Calculate Reward' },
		{ status: 'Collect Reward', text: 'Collect Reward' },
		{ status: 'Failed', text: 'Failed' }
	],
	currentStep: 0
};

let timer: NodeJS.Timer | null = null;
export const proposalSteps = writable<ProposalSteps>(initialProposalSteps, () => {
	selectedProposal.subscribe(async (proposal) => {
		// update steps
		console.log('Subscribe to proposalSteps');
		if (proposal) {
			const currentStep = initialProposalSteps.steps.findIndex(
				(val) => val.text == getVoteStatus(proposal.account)
			);

			timer = setInterval(() => {
				const voteEndTime = proposal?.account.voteEndAt;
				const revealEndTime = proposal?.account.voteEndRevealAt;
				const endTime = getNextDeadline([voteEndTime, revealEndTime]);
				let updatedText = initialProposalSteps.steps[currentStep]?.status.toString() ?? 'PH';
				if (isInFuture(endTime)) {
					const countdown = countdownFromUnix(endTime);
					updatedText = `${updatedText} ${countdown.toString()}`;
				}

				const currentSteps = initialProposalSteps.steps;
				currentSteps[currentStep] = {
					...currentSteps[currentStep],
					text: updatedText
				};
				proposalSteps.set({
					steps: currentSteps,
					currentStep
				});
			}, 1000);
		}
	});
});
