import { Keypair } from '@solana/web3.js';

export const loadKeypairFromEnv = (): Keypair => {
	return Keypair.fromSecretKey(
		Buffer.from(
			JSON.parse(
				require('fs').readFileSync(process.env.WALLET, {
					encoding: 'utf-8',
				})
			)
		)
	);
};
