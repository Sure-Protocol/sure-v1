import * as anchor from '@project-serum/anchor';
import * as tribeca from '@tribecahq/tribeca-sdk';

export const daysToSecond = (days: number): anchor.BN => {
	return new anchor.BN(days).mul(tribeca.ONE_DAY);
};

export const isInFuture = (unix: number): boolean => {
	const d = Math.floor(Date.now() / 1000);
	return unix > d;
};

export const getNextDeadline = (unixDeadlines: number[]): number => {
	const d = Math.floor(Date.now() / 1000);
	return unixDeadlines.reduce((t0, t1) => {
		if (d < t0 && d < t1) {
			return t0;
		} else if (d > t0 && d < t1) {
			return t1;
		} else {
			return d;
		}
	}, unixDeadlines[0]);
};
