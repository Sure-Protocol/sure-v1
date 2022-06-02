import { useConnection } from '@solana/wallet-adapter-react';
import { PublicKey } from '@solana/web3.js';

export function explorerLink(publicKey: PublicKey): string {
	const { connection } = useConnection();
	const endpoint = connection.rpcEndpoint;
	return `https://explorer.solana.com/address/${publicKey.toBase58()}?customUrl=${endpoint}&cluster=custom`;
}
