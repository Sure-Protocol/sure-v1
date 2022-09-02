import { writable } from 'svelte/store';
import type { SureOracleSDK, ProposalType } from '@surec/oracle';
import type { ProgramAccount } from '@saberhq/token-utils';
import { newEvent } from './event';

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
