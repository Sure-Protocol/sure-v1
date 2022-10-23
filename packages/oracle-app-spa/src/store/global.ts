import { writable } from 'svelte/store';
import type { SureOracleSDK } from '@surec/oracle';
import type { PublicKey } from '@solana/web3.js';
import type { Adapter } from '@solana/wallet-adapter-base';
import type { SureAugmentedProvider } from '@surec/oracle';
import * as web3 from '@solana/web3.js';
import * as oracle from '@surec/oracle';

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
	provider: undefined,
});

export type RPCConfig = {
	value?: string;
	label?: string;
};

export const rpcConfig = writable<RPCConfig>({
	value: 'https://api.devnet.solana.com',
	label: `Solana devnet`,
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
		refresh: true,
	},
	(set) => {
		set({ isLoading: false, loadingFailed: false, refresh: true });
		const interval = setInterval(() => {
			set({ isLoading: false, loadingFailed: false, refresh: true });
		}, 30000);
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

export const getUpdateOracleSdkConnection = (
	rpc: RPCConfig,
	store: GlobalStoreT
): GlobalStoreT | undefined => {
	console.log('getUpdateOracleSdkConnection ', rpc);
	let connection = new web3.Connection(
		rpc.value ?? web3.clusterApiUrl('devnet')
	);

	if (store?.wallet?.publicKey) {
		console.log('rpc: ', rpc);
		const oracleSdk = oracle.SureOracleSDK.init({
			connection,
			wallet: store.wallet,
			opts: { skipPreflight: true },
		});
		return {
			...store,
			oracleSDK: oracleSdk,
			walletPk: store.wallet.publicKey,
			wallet: store.wallet,
			provider: oracleSdk.provider,
		};
	}
	return undefined;
};
