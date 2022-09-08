import { AnchorConnectionProvider } from '@svelte-on-solana/wallet-adapter-anchor';

export * from './event';
export * from './global';
export * from './proposal';
export * from './config';
export * from './token';

export const oneDivXToFloat = (x: number): number => {
	return 1 / x;
};
