import type { BN } from '@project-serum/anchor';
import type { PublicKey } from '@solana/web3';

export const prettyPublicKey = (pk: PublicKey): string => {
	const pkString = pk.toString();
	return pkString.slice(0, 4) + '...' + pkString.slice(-4);
};

export const unixToReadable = (unixTimestamp: BN): string => {
	const dd = new Date(unixTimestamp.toNumber() * 1000);
	return dd.toDateString();
};

export const prettyLargeNumber = (number: BN): string => {
	return number.toString().replace(/\B(?=(\d{3})+(?!\d))/g, ',');
};
