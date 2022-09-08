import { writable } from 'svelte/store';
import type { SureOracleSDK } from '@surec/oracle';
import type { PublicKey } from '@solana/web3.js';
import type { Adapter } from '@solana/wallet-adapter-base';
import type { SureAugmentedProvider } from '@surec/oracle';

export type GlobalStoreT = {
	oracleSDK: SureOracleSDK | undefined;
	walletPk: PublicKey | undefined;
	wallet: Adapter | undefined;
	provider: SureAugmentedProvider | undefined;
};

export const globalStore = writable<GlobalStoreT>({
	oracleSDK: undefined,
	walletPk: undefined,
	wallet: undefined,
	provider: undefined
});

export type LoadingStateT = {
	isLoading: boolean;
	loadingFailed: boolean;
	refresh: boolean;
};

export const loadingState = writable<LoadingStateT>(
	{
		isLoading: false,
		loadingFailed: false,
		refresh: true
	},
	(set) => {
		set({ isLoading: false, loadingFailed: false, refresh: true });
		const interval = setInterval(() => {
			set({ isLoading: false, loadingFailed: false, refresh: true });
		}, 20000);
		() => clearInterval(interval);
	}
);

export const startLoading = () => {
	loadingState.set({ isLoading: true, loadingFailed: false, refresh: false });
};

export const loadingFailed = () => {
	loadingState.set({ isLoading: false, loadingFailed: true, refresh: false });
};

export const loadingSuccessful = () => {
	loadingState.set({ isLoading: false, loadingFailed: false, refresh: false });
};
