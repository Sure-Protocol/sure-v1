import * as anchor from '@project-serum/anchor';
import { program } from '@project-serum/anchor/dist/cjs/spl/token';
import { findProgramAddressSync } from '@project-serum/anchor/dist/cjs/utils/pubkey';
import { Connection, Keypair, PublicKey, SystemProgram } from '@solana/web3.js';
import { SmartWallet } from './smart_wallet';
import * as goki from '@gokiprotocol/client';
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

	const provider = new anchor.AnchorProvider(connection, wallet, {
		skipPreflight: false,
	});
	anchor.setProvider(provider);

	const program = new goki.GokiSDK(provider, goki.GOKI_ADDRESSES);
	const owners = [wallet.payer.publicKey];
	let res = await program.newSmartWallet({
		owners: owners,
		threshold: new anchor.BN(10),
		numOwners: 3,
	});
	console.log('res: ', res);
}

run().catch((err) => {
	console.log('sure-cli.initializeProtocol.error. Cause: ' + err);
	console.error(err.stack);
	process.exit(1);
});
