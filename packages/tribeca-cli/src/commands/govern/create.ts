import { Command, Flags } from '@oclif/core';
import * as anchor from '@project-serum/anchor';
import { Connection, PublicKey } from '@solana/web3.js';
import * as saber_contrib from '@saberhq/solana-contrib';
import { GokiSDK } from '@gokiprotocol/client';
import { loadKeypairFromEnv } from '../../utils/loadkey';
import * as tribeca from '@tribecahq/tribeca-sdk';

export default class Govern extends Command {
	static description = 'Create governor of governance ';
	static examples = [
		`$ tb govern create --network dev --wallet <Publickey>
`,
	];

	static flags = {
		network: Flags.option({
			parse: async (input: string): Promise<string | undefined> => {
				if (input == 'dev') {
					return 'https://api.devnet.solana.com';
				} else if (input == 'mainnet') {
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
			input: ['dev', 'mainnet', 'testnet', 'local '],
			required: true,
			options: ['dev', 'mainnet', 'testnet', 'local '],
			char: 'n',
		}),
		wallet: Flags.string({
			name: 'wallet',
			char: 's',
			required: true,
			default: undefined,
			description:
				'Goki smart wallet public key. If not provided a new one will be generated',
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
		const gokiSDK = GokiSDK.load({
			provider,
		});
		const [governor] = await tribeca.findGovernorAddress(wallet.publicKey);
		let smartWalletKey: PublicKey;
		if (flags.wallet === undefined) {
			this.log('> Create new goki smart wallet');
			const { smartWalletWrapper, tx } = await gokiSDK.newSmartWallet({
				owners: [governor],
				numOwners: 3,
				threshold: new anchor.BN(1),
			});
			smartWalletKey = smartWalletWrapper.key;
		} else {
			smartWalletKey = new PublicKey(flags.wallet);
		}

		this.log('> create governor');
		try {
			// locker as electrorate
			const electorate = tribeca.getLockerAddress(wallet.publicKey);
			const { wrapper, tx: tx2 } = await tribecaSDK.govern.createGovernor({
				electorate: electorate,
				smartWallet: smartWalletKey,
			});

			this.log('tb.govern.success!');
			this.log(`govern key: ${wrapper.governorKey.toString()}`);
		} catch (err) {
			this.error(`tb.govern.failed. Cause: ${err}`);
		}
	}
}
