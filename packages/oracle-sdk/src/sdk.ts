import * as anchor from '@project-serum/anchor';
import * as solana_contrib from '@saberhq/solana-contrib';
import { Wallet } from '@project-serum/anchor/dist/cjs/provider';
import { Oracle, IDL } from '../../idls/oracle';
import { Proposal } from './proposal';
import { SolanaAugmentedProvider } from '@saberhq/solana-contrib';
import { Vote } from './vote';
import { PDA } from './pda';
import { SURE_ADDRESSES } from './constants';

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
		readonly provider: solana_contrib.AugmentedProvider,
		readonly program: anchor.Program<Oracle>,
		readonly pda: PDA
	) {}

	static init({
		provider,
	}: {
		provider: solana_contrib.Provider;
	}): SureOracleSDK {
		const anchorProvider = new anchor.AnchorProvider(
			provider.connection,
			provider.wallet,
			{ skipPreflight: true }
		);
		// get anchorprogram properly
		const program = new anchor.Program(
			IDL,
			SURE_ADDRESSES.Oracle,
			anchorProvider
		);
		const pda = new PDA();
		return new SureOracleSDK(
			new SolanaAugmentedProvider(provider),
			program,
			pda
		);
	}

	static pda(): PDA {
		return new PDA();
	}

	proposal(): Proposal {
		return new Proposal(this);
	}

	vote(): Vote {
		return new Vote(this);
	}
}
