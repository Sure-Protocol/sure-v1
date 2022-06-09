import * as anchor from '@project-serum/anchor';
import NodeWallet from '@project-serum/anchor/dist/cjs/nodewallet';
import {
	createMint,
	getMint,
	getOrCreateAssociatedTokenAccount,
	mintTo,
} from '@solana/spl-token';
import { Connection, Keypair, LAMPORTS_PER_SOL } from '@solana/web3.js';
import { Money, SureSdk } from '@surec/sdk';
import fs from 'fs';
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
	const wallet = new NodeWallet(keypair);
	const network = process.env.NETWORK!;
	const connection = new Connection(network, {});

	const sureSDK = SureSdk.init(connection, wallet);
	// Mint token

	const tokenMint = await createMint(
		connection,
		(wallet as NodeWallet).payer,
		wallet.publicKey,
		wallet.publicKey,
		8
	);
	const tokenMintAccount = await getMint(connection, tokenMint);

	const walletAta = await getOrCreateAssociatedTokenAccount(
		connection,
		(wallet as NodeWallet).payer,
		tokenMint,
		wallet.publicKey
	);

	const mintAmount = Money.new(tokenMintAccount.decimals, 1000000);
	await mintTo(
		connection,
		wallet.payer,
		tokenMint,
		walletAta.address,
		wallet.payer,
		mintAmount.convertToDecimals()
	);
	console.log('New token: ', tokenMintAccount.address);
	fs.writeFile(
		'./log/token_mint.json',
		tokenMintAccount.address.toBase58(),
		{ flag: 'w+' },
		(err) => console.log('err: ', err)
	);
}

run().catch((err) => {
	console.log('sure-cli.mintToken.error. Cause: ' + err);
	console.error(err.stack);
	process.exit(1);
});
