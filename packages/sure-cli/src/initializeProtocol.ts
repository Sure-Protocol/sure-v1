import * as anchor from '@project-serum/anchor';
import { Connection, Keypair, PublicKey } from '@solana/web3.js';
import { SureSdk } from '@surec/sdk';

async function run() {
	const keypair = Keypair.fromSecretKey(
		Buffer.from(
			JSON.parse(
				require('fs').readFileSync(process.env.WALLET, {
					encoding: 'utf-8',
				})
			)
		)
	);
	const wallet = new anchor.Wallet(keypair);
	const network = process.env.NETWORK!;
	const connection = new Connection(network, {});

	const sureSDK = SureSdk.init(
		connection,
		wallet,
		new PublicKey('D47wvD2bTDXR9XqqHdP8bwYSXu2QPMW6fGHg2aEBKunM')
	);
	await sureSDK.protocol.initializeProtocol();
}

run().catch((err) => {
	console.log('sure-cli.initializeProtocol.error. Cause: ' + err);
	console.error(err.stack);
	process.exit(1);
});
