import { writable } from 'svelte/store';
import type { SureOracleSDK } from '@surec/oracle';

export type GlobalStoreT = {
	oracleSDK: SureOracleSDK | undefined;
};

export const globalStore = writable<GlobalStoreT>({
	oracleSDK: undefined
});

// create writable store
export const createProposalState = writable(false, () => {
	console.log('subscribe');
	return () => console.log('unsubsribe');
});
