import type { BN } from '@project-serum/anchor';
import type { PublicKey } from '@solana/web3';

export const prettyPublicKey = (pk: PublicKey): string => {
	const pkString = pk.toString();
	return pkString.slice(0, 4) + '...' + pkString.slice(-4);
};

export const unixToReadable = (unixTimestamp: BN): string => {
	const dd = new Date(unixTimestamp.toNumber() * 1000);
	return `${dd.toLocaleDateString()}, ${dd.toLocaleTimeString()}`;
};

export const countdownFromUnix = (unixDeadline: number): string => {
	const d = Math.floor(Date.now() / 1000);
	const remainingTime = unixDeadline - d;
	const days = Math.floor(remainingTime / (60 * 60 * 24));
	const hours = Math.floor((remainingTime % (60 * 60 * 24)) / (60 * 60));
	const minutes = Math.floor((remainingTime % (60 * 60)) / 60);
	const seconds = Math.floor(remainingTime % 60);

	let remainingTimeString = '';
	if (days > 0) {
		remainingTimeString = `${remainingTimeString} ${days}d`;
	}
	if (hours > 0) {
		remainingTimeString = `${remainingTimeString} ${hours}h`;
	}
	if (minutes > 0) {
		remainingTimeString = `${remainingTimeString} ${minutes}m`;
	}
	if (seconds > 0) {
		remainingTimeString = `${remainingTimeString} ${seconds}s`;
	}
	return remainingTimeString;
};

export const prettyLargeNumber = (number: BN): string => {
	return number.toString().replace(/\B(?=(\d{3})+(?!\d))/g, ',');
};

export const prettySolanaExplorerLink = (tx: string, network: string): string => {
	return `https://explorer.solana.com/tx/${tx}?cluster=${network}`;
};
