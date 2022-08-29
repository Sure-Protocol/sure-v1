import { writable } from 'svelte/store';
import { SureOracleSDK, type ProposalType } from '@surec/oracle';
import type { PublicKey } from '@solana/web3.js';
import type { Adapter } from '@solana/wallet-adapter-base';
import type { Provider } from '@project-serum/anchor';
import type { SolanaProvider } from '@saberhq/solana-contrib';
import type { ProgramAccount } from '@saberhq/token-utils';

export type GlobalStoreT = {
	oracleSDK: SureOracleSDK | undefined;
	walletPk: PublicKey | undefined;
	wallet: Adapter | undefined;
	provider: SolanaProvider | undefined;
};

export const globalStore = writable<GlobalStoreT>({
	oracleSDK: undefined,
	walletPk: undefined,
	wallet: undefined,
	provider: undefined
});

// create writable store
export const createProposalState = writable(false, () => {
	console.log('subscribe');
	return () => console.log('unsubsribe');
});

export type Event = {
	name: string;
};
export const newEvent = writable<Event>({ name: '' }, () => {
	console.log('subscribe');
});

export const selectedProposal = writable<ProgramAccount<ProposalType> | undefined>(
	undefined,
	() => {
		console.log('Subsribe');
	}
);
