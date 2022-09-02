import { Command, Flags } from '@oclif/core';
import * as anchor from '@project-serum/anchor';
import { Connection, PublicKey } from '@solana/web3.js';
import * as saber_contrib from '@saberhq/solana-contrib';
import { GokiSDK } from '@gokiprotocol/client';
import { loadKeypairFromEnv } from '../../utils/loadkey';
import NodeWallet from '@project-serum/anchor/dist/cjs/nodewallet';

export default class SmartWallet extends Command {
	static description = 'Create Goki Smart Wallet';
	static examples = [
		`$ tb smart-wallet create --network dev 
`,
	];

	static flags = {
		network: Flags.option({
			parse: async (input: string): Promise<string | undefined> => {
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
		owners: Flags.string({
			helpValue: '<LIST OF OWNERS []>',
			required: true,
			description: 'owners in addition to you.',
			char: 'p',
		}),
	};

	async run(): Promise<void> {
		const { args, flags } = await this.parse(SmartWallet);

		const keypair = loadKeypairFromEnv();
		const wallet = new anchor.Wallet(keypair);
		const network = flags.network;
		if (network === undefined) {
			this.error(`network ${network} is undefined`);
		}
		const connection = new Connection(network, {});

		const anchorProvider = new anchor.AnchorProvider(connection, wallet, {
			skipPreflight: true,
		});
		anchor.setProvider(anchorProvider);
		const provider = saber_contrib.SolanaProvider.load({
			connection: anchorProvider.connection,
			wallet: anchorProvider.wallet,
			opts: anchorProvider.opts,
		});
		const gokiSDK = GokiSDK.load({ provider });

		const owners = flags.owners.split(',').map((owner) => new PublicKey(owner));

		try {
			const { smartWalletWrapper, tx } = await gokiSDK.newSmartWallet({
				base: (wallet as NodeWallet).payer,
				owners: owners,
				threshold: new anchor.BN(1),
				numOwners: owners.length,
			});
			await tx.confirm();
			this.log(`tb.createSmartWallet.success`);
			this.log(`smart wallet key: ${smartWalletWrapper.key}`);
			this.log(`owners: ${flags.owners}`);
		} catch (err) {
			this.log(`tb.createSmartWallet.fail. Cause: ${err}`);
		}
	}
}
