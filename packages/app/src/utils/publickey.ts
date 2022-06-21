import { PublicKey } from '@solana/web3.js';

export function prettyPublicKey(publicKey: PublicKey): string {
	const pbBase = publicKey.toBase58();
	return prettyPublicKeyString(pbBase);
}

export function prettyPublicKeyString(publicKeyStr: string): string {
	return publicKeyStr.slice(0, 4) + '...' + publicKeyStr.slice(-2);
}
