import * as anchor from '@project-serum/anchor';
import { program } from '@project-serum/anchor/dist/cjs/spl/token';
import { Connection, Keypair, PublicKey, SystemProgram } from '@solana/web3.js';
import { SmartWallet } from './smart_wallet';
import * as goki from '@gokiprotocol/client';
import * as saber_contrib from '@saberhq/solana-contrib';
import * as saber_anchor from '@saberhq/anchor-contrib';
import {
	GokiSDK,
	GOKI_ADDRESSES,
	SmartWalletWrapper,
} from '@gokiprotocol/client';
import * as tribeca from '@tribecahq/tribeca-sdk';
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
	const gokiSDK = GokiSDK.load({
		provider,
	});
	const [governor] = await tribeca.findGovernorAddress(wallet.publicKey);
	const { smartWalletWrapper, tx } = await gokiSDK.newSmartWallet({
		owners: [governor],
		numOwners: 3,
		threshold: new anchor.BN(1),
	});
	console.log('smart wallet res: ', tx);
	const { wrapper, tx: tx2 } = await tribecaSDK.govern.createGovernor({
		electorate: wallet.publicKey,
		smartWallet: smartWalletWrapper.key,
	});
	console.log('createGovenor res: ', tx2);
}

run().catch((err) => {
	console.log('sure-cli.initializeProtocol.error. Cause: ' + err);
	console.error(err.stack);
	process.exit(1);
});
