import * as anchor from '@project-serum/anchor';
import { program } from '@project-serum/anchor/dist/cjs/spl/token';
import { Connection, Keypair, PublicKey, SystemProgram } from '@solana/web3.js';
import { SmartWallet } from './smart_wallet';
import * as goki from '@gokiprotocol/client';
import * as saber_contrib from '@saberhq/solana-contrib';
import * as saber_anchor from '@saberhq/anchor-contrib';
import * as chai_solana from '@saberhq/chai-solana';
import * as tribeca from '@tribecahq/tribeca-sdk';
import { expect } from 'chai';
import * as spl_token from '@solana/spl-token';
import { TOKEN_PROGRAM_ID } from '@solana/spl-token';
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
	const anchorProvider = new anchor.AnchorProvider(connection, wallet, {
		skipPreflight: false,
	});
	anchor.setProvider(anchorProvider);
	const provider = saber_contrib.SolanaProvider.init({
		connection: anchorProvider.connection,
		wallet: anchorProvider.wallet,
		opts: anchorProvider.opts,
	});

	const tribecaSDK = tribeca.TribecaSDK.load({
		provider,
	});

	const [governor] = await tribeca.findGovernorAddress(wallet.publicKey);

	const tokenMint = await spl_token.Token.createMint(
		connection,
		wallet.payer,
		wallet.payer.publicKey,
		null,
		6,
		TOKEN_PROGRAM_ID
	);

	const { locker, tx: lockerTx } = await tribecaSDK.createLocker({
		governor,
		proposalActivationMinVotes: new anchor.BN(1_000_000),
		govTokenMint: tokenMint.publicKey,
	});
	const [lockerFound] = await tribeca.findLockerAddress(governor);
	console.log('locker: ', locker.toString());
}

run().catch((err) => {
	console.log('sure-cli.initializeProtocol.error. Cause: ' + err);
	console.error(err.stack);
	process.exit(1);
});
