import * as anchor from '@project-serum/anchor';
import { program } from '@project-serum/anchor/dist/cjs/spl/token';
import { findProgramAddressSync } from '@project-serum/anchor/dist/cjs/utils/pubkey';
import { getAssociatedTokenAddress } from '@solana/spl-token';
import { Connection, Keypair, PublicKey, SystemProgram } from '@solana/web3.js';
import { SurePool, SureSdk } from '@surec/sdk';
import { SmartWallet } from './smart_wallet';

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
	const gokiProgramId = new PublicKey(
		'GokivDYuQXPZCWRkwMhdH2h91KpDQXBEmpgBgs55bnpH'
	);
	const IDL = await anchor.Program.fetchIdl(gokiProgramId);
	if (IDL === null) {
		throw new Error('could not find IDL for goki');
	}
	const smart_wallet = new anchor.Program<SmartWallet>(
		IDL,
		gokiProgramId,
		provider
	);
	const [smartWalletPDA, smartWalletBump] = findProgramAddressSync(
		[
			anchor.utils.bytes.utf8.encode('GokiSmartWallet'),
			wallet.payer.publicKey.toBytes(),
		],
		smart_wallet.programId
	);
	const owners = [new PublicKey('')];
	smart_wallet.methods
		.createSmartWallet(0, 3, owners, new anchor.BN(2), new anchor.BN(10))
		.accounts({
			base: wallet.payer.publicKey,
			smartWallet: smartWalletPDA,
			payer: wallet.payer.publicKey,
			systemProgram: SystemProgram.programId,
		})
		.rpc();
}

run().catch((err) => {
	console.log('sure-cli.initializeProtocol.error. Cause: ' + err);
	console.error(err.stack);
	process.exit(1);
});
