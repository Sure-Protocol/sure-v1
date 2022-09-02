import { Command, Flags } from '@oclif/core';
import * as anchor from '@project-serum/anchor';
import { Connection, PublicKey } from '@solana/web3.js';
import * as saber_contrib from '@saberhq/solana-contrib';
import { GokiSDK } from '@gokiprotocol/client';
import { loadKeypairFromEnv } from '../../utils/loadkey';
import * as tribeca from '@tribecahq/tribeca-sdk';
import * as spl_token from '@solana/spl-token';
import NodeWallet from '@project-serum/anchor/dist/cjs/nodewallet';

export default class SmartWallet extends Command {
	static description = 'Create locker for storing ve tokens';
	static examples = [
		`$ tb locker create --network dev --mint <Publickey>
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
		mint: Flags.string({
			name: 'mint',
			char: 't',
			required: true,
			default: undefined,
			description:
				'Token mint to serve as the mint for the locker. If not provided a new mint will be generated',
		}),
	};

	async run(): Promise<void> {
		const { args, flags } = await this.parse(SmartWallet);
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

		// create locker
		try {
			const governor = tribeca.getGovernorAddress(wallet.publicKey);
			const { locker, tx: lockerTx } = await tribecaSDK.createLocker({
				baseKP: (wallet as NodeWallet).payer,
				governor,
				proposalActivationMinVotes: new anchor.BN(1_000_000),
				govTokenMint: new PublicKey(flags.mint),
			});
			this.log('tb.createLocker.success. ');
			this.log(`governor: ${governor.toString()}`);
			this.log(`locker: ${locker.toString()}`);
		} catch (err) {
			this.error(`tb.createLocker.error! Cause: ${err}`);
		}
	}
}
