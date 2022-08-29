import * as anchor from '@project-serum/anchor';
import * as tribeca from '@tribecahq/tribeca-sdk';

export const daysToSecond = (days: number): anchor.BN => {
	return new anchor.BN(days).mul(tribeca.ONE_DAY);
};
