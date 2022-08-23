import { Command, Flags } from '@oclif/core';
import * as anchor from '@project-serum/anchor';
import { Connection, PublicKey } from '@solana/web3.js';
import * as saber_contrib from '@saberhq/solana-contrib';
import { GokiSDK } from '@gokiprotocol/client';
import { loadKeypairFromEnv } from '../../utils/loadkey';
import * as tribeca from '@tribecahq/tribeca-sdk';
import { GovernorWrapper, GovernWrapper } from '@tribecahq/tribeca-sdk';

export default class CreateProposal extends Command {
	static description = `
	Create Proposal for the given governor. 
	
	NOTE: Instructions to be executed on completed poll is 
	not implemented.
	`;
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
		governor: Flags.string({
			name: 'wallet',
			char: 'g',
			required: true,
			default: undefined,
			description: 'Governor',
		}),
		title: Flags.string({
			name: 'title',
			char: 't',
			required: true,
			default: undefined,
			description: 'title of proposal',
		}),
		link: Flags.string({
			name: 'link',
			char: 'l',
			required: false,
			helpLabel: 'link needs to a valid url',
			default: undefined,
			description: 'Link to proposal description.',
		}),
	};

	async run(): Promise<void> {
		const { args, flags } = await this.parse(CreateProposal);
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

		// load governor
		const governor = new PublicKey(flags.governor);

		// load locker
		const governorWrapper = new GovernorWrapper(tribecaSDK, governor);

		this.log('> create governor');
		try {
			// locker as electrorate
			const pendingProposal = await governorWrapper.createProposal({
				proposer: wallet.publicKey,
				instructions: [],
			});
			await pendingProposal.tx.confirm();

			// add metadata to proposal
			const tx = await governorWrapper.createProposalMeta({
				proposal: pendingProposal.proposal,
				title: flags.title,
				descriptionLink: flags.link ?? '',
			});

			this.log('tb.govern.createProposal.success!');
			this.log(`proposal key: ${pendingProposal.proposal.toString()}`);
		} catch (err) {
			this.error(`tb.govern.createProposal.failed. Cause: ${err}`);
		}
	}
}
