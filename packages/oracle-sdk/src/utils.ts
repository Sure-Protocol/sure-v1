import { PublicKey } from '@solana/web3.js';

export const validateKeys = (keys: { v: PublicKey; n: string }[]) => {
	const undefinedErrors = keys
		.filter((k) => k.v === undefined)
		.map((k) => `${k.n} is undefined.`)
		.join(', ');
	if (undefinedErrors.length > 0) {
		throw new Error(undefinedErrors);
	}
};
