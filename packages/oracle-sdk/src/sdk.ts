import * as anchor from '@project-serum/anchor';
import * as solanaContrib from '@saberhq/solana-contrib';
import { Wallet } from '@project-serum/anchor/dist/esm/provider';
import { Oracle, IDL } from './idls/oracle.js';
import { Proposal } from './proposal.js';
import { Vote } from './vote.js';
import { PDA } from './pda.js';
import { SURE_ADDRESSES } from './constants.js';
import { Config } from './config.js';
import {
	ConfirmOptions,
	Connection,
	TransactionInstruction,
} from '@solana/web3.js';

export type ProviderProps = {
	connection: anchor.web3.Connection;
	wallet: Wallet;
	opts: anchor.web3.ConfirmOptions;
};

// checkpoint : generate oracle idl and use it in sdk
export class Provider {
	constructor(
		readonly connection: anchor.web3.Connection,
		readonly wallet: Wallet,
		readonly opts: anchor.web3.ConfirmOptions
	) {}

	static init({ connection, wallet, opts }: ProviderProps): Provider {
		return new Provider(connection, wallet, opts);
	}
}

export class SureOracleSDK {
	constructor(
		readonly provider: solanaContrib.AugmentedProvider,
		readonly program: anchor.Program<Oracle>,
		readonly pda: PDA
	) {}

	static init({
		connection,
		wallet,
		opts,
	}: {
		connection: Connection;
		wallet: Wallet;
		opts?: ConfirmOptions;
	}): SureOracleSDK {
		const oracleProvider = solanaContrib.SolanaProvider.init({
			connection,
			wallet: wallet,
			opts,
		});

		const anchorProvider = new anchor.AnchorProvider(connection, wallet, {
			skipPreflight: true,
		});

		// get anchorprogram properly
		const program = new anchor.Program(
			IDL,
			SURE_ADDRESSES.Oracle,
			anchorProvider
		);
		const pda = new PDA();
		return new SureOracleSDK(
			new solanaContrib.SolanaAugmentedProvider(oracleProvider),
			program,
			pda
		);
	}

	static pda(): PDA {
		return new PDA();
	}

	config(): Config {
		return new Config(this);
	}
	proposal(): Proposal {
		return new Proposal(this);
	}

	vote(): Vote {
		return new Vote(this);
	}

	executeTransactionInstructions(
		tx: TransactionInstruction[]
	): solanaContrib.TransactionEnvelope {
		return this.provider.newTX(tx);
	}
}
