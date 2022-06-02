import { PublicKey } from '@solana/web3.js';

export function prettyPublicKey(publicKey: PublicKey): string {
	const pbBase = publicKey.toBase58();
	return pbBase.slice(0, 4) + '...' + pbBase.slice(-4);
}
