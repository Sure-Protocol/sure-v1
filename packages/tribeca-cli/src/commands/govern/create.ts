import { Command, Flags } from '@oclif/core';
import * as anchor from '@project-serum/anchor';
import { Connection, PublicKey } from '@solana/web3.js';
import * as saber_contrib from '@saberhq/solana-contrib';
import * as goki from '@gokiprotocol/client';
import { loadKeypairFromEnv } from '../../utils/loadkey';
import * as tribeca from '@tribecahq/tribeca-sdk';
import NodeWallet from '@project-serum/anchor/dist/cjs/nodewallet';

export default class Govern extends Command {
	static description = 'Create governor of governance ';
	static examples = [
		`$ tb govern create --network=dev
`,
	];

	static flags = {
		network: Flags.option({
			parse: async (input: string): Promise<string | undefined> => {
				console.log('input: ', input == 'dev');
				if (input == 'dev') {
					return 'https://api.devnet.solana.com';
				} else if (input == 'mainnet-beta') {
					return 'https://api.mainnet-beta.solana.com';
				} else if (input == 'testnet') {
					return 'https://api.testnet.solana.com';
				} else if (input == 'local') {
					return 'http://localhost:8899';
				}
				return undefined;
			},
			defaultHelp: 'hello there',
			helpValue: '<SOLANA NETWORK>',
			input: ['dev', 'mainnet-beta', 'testnet', 'local '],
			required: true,
			options: ['dev', 'mainnet-beta', 'testnet', 'local '],
			char: 'n',
		}),
	};

	async run(): Promise<void> {
		const { args, flags } = await this.parse(Govern);
		const keypair = loadKeypairFromEnv();
		const wallet = new anchor.Wallet(keypair);
		const network = process.env.NETWORK!;
		const connection = new Connection(network, {});

		const anchorProvider = new anchor.AnchorProvider(connection, wallet, {
			skipPreflight: false,
		});
		anchor.setProvider(anchorProvider);
		const provider = saber_contrib.SolanaProvider.load({
			connection: anchorProvider.connection,
			wallet: anchorProvider.wallet,
			opts: anchorProvider.opts,
		});

		const tribecaSDK = tribeca.TribecaSDK.load({
			provider,
		});

		this.log('> create governor');
		try {
			// locker as electrorate
			const smartWalletKey = goki.getSmartWalletAddress(wallet.publicKey);
			const electorate = tribeca.getLockerAddress(wallet.publicKey);
			const { wrapper, tx: tx2 } = await tribecaSDK.govern.createGovernor({
				electorate: electorate,
				smartWallet: smartWalletKey,
				baseKP: (wallet as NodeWallet).payer,
			});

			this.log('\n tb.govern.success!');
			this.log(`govern key: ${wrapper.governorKey.toString()}`);
			this.log(`expected - smart wallet key: ${smartWalletKey}`);
			this.log(`expected - electorate: ${electorate} \n`);
		} catch (err) {
			this.error(`tb.govern.failed. Cause: ${err}`);
		}
	}
}
