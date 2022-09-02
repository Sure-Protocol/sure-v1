import { Command, Flags } from '@oclif/core';
import * as anchor from '@project-serum/anchor';
import { Connection, PublicKey } from '@solana/web3.js';
import * as saber_contrib from '@saberhq/solana-contrib';
import * as token_utils from '@saberhq/token-utils';
import { GokiSDK } from '@gokiprotocol/client';
import { loadKeypairFromEnv } from '../../utils/loadkey';
import * as tribeca from '@tribecahq/tribeca-sdk';
import * as spl_token from '@solana/spl-token';
import { LockerWrapper } from '@tribecahq/tribeca-sdk';
import { program } from '@project-serum/anchor/dist/cjs/spl/associated-token';
import { TransactionBuilder } from '@metaplex-foundation/js-next';
import { getMintInfo } from '@saberhq/token-utils';
import * as spl from './../../../node_modules/@solana/spl-token';

const NUMBER_SECONDS_IN_DAY = 86400;
const NUMBER_MS_IN_DAY = NUMBER_SECONDS_IN_DAY * 1000;

export default class SmartWallet extends Command {
	static description = 'Lock tokens in Tribeca locker';
	static examples = [
		`$ tb locker lock --network dev --locker <Publickey> --amount 1.2 --period 365
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
		locker: Flags.string({
			name: 'locker public key',
			char: 'l',
			required: true,
			description: 'PublicKey of locker to lock tokens in.',
		}),
		amount: Flags.string({
			name: 'amount',
			char: 'a',
			required: true,
			description: 'Amount of tokens to lock in decimals',
		}),
		period: Flags.integer({
			name: 'lock period',
			char: 'p',
			required: true,
			description: 'Lock period in days',
		}),
	};

	async run(): Promise<void> {
		const { args, flags } = await this.parse(SmartWallet);

		// load locker key
		let lockerKey: PublicKey;
		try {
			lockerKey = new PublicKey(flags.locker);
		} catch (err) {
			this.error('tb.locker.lock.fail! Could not get locker key.');
		}

		const keypair = loadKeypairFromEnv();
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

		// load locker
		const lockerAccount =
			await tribecaSDK.programs.LockedVoter.account.locker.fetch(lockerKey);

		// load amount
		const amount = flags.amount;
		const decimals = (
			await spl.getMint(provider.connection, lockerAccount.tokenMint)
		).decimals;

		const lockAmountBN = new anchor.BN(parseFloat(amount)).mul(
			new anchor.BN(10).pow(new anchor.BN(decimals))
		);

		// load lock period
		const lockPeriod = new anchor.BN(flags.period);
		const lockDuration = lockPeriod.mul(new anchor.BN(NUMBER_SECONDS_IN_DAY));
		this.log(`> lock duration ${lockDuration}s`);
		// load governor from locker
		const governor = lockerAccount.governor;
		this.log(`> governor address: ${governor.toString()}`);

		const locker = await LockerWrapper.load(tribecaSDK, lockerKey, governor);
		// create locker
		try {
			const tx = await locker.lockTokens({
				amount: lockAmountBN,
				duration: lockDuration,
			});

			this.log('tb.locker.lockTokens.success. ');
		} catch (err) {
			this.error(`tb.locker.lockTokens.error! Cause: ${err}`);
		}
	}
}
