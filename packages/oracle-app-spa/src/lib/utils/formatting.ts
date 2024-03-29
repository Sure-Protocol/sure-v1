import type { PublicKey } from '@solana/web3.js';
import * as anchor from '@project-serum/anchor';

export const prettyPublicKeyString = (pkString: string) => {
	return pkString.slice(0, 4) + '...' + pkString.slice(-4);
};

export const prettyPublicKey = (pk: PublicKey): string => {
	const pkString = pk.toString();
	return prettyPublicKeyString(pkString);
};

export const unixToReadable = (unixTimestamp: anchor.BN): string => {
	const dd = new Date(unixTimestamp.toNumber() * 1000);
	return `${dd.toLocaleDateString()}, ${dd.toLocaleTimeString()}`;
};

export const unixSecondsToReadableString = (unix: number): string => {
	return countdownFromUnix(Math.floor(Date.now() / 1000) + unix);
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

export const prettyLargeNumberString = (number: string): string => {
	const bn = new anchor.BN(number);
	return prettyLargeNumber(bn);
};

export const prettyLargeNumber = (number: anchor.BN): string => {
	return number.toString().replace(/\B(?=(\d{3})+(?!\d))/g, ',');
};

export const prettySolanaExplorerLink = (
	tx: string,
	network: string
): string => {
	return `https://explorer.solana.com/tx/${tx}?cluster=${network}`;
};

export const maxXCharacters = (str: string, maxChar: number): string => {
	if (str) {
		return str.toString().slice(0, maxChar) + '...';
	}
	return str;
};
