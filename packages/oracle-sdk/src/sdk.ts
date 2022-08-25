import * as anchor from '@project-serum/anchor';
import * as web3 from '@solana/web3.js';
import * as solana_contrib from '@saberhq/solana-contrib';
import * as anchor_contrib from '@saberhq/anchor-contrib';
import { Wallet } from '@project-serum/anchor/dist/cjs/provider';
import { OracleIDL, OracleJSON } from '../../idls/oracle';
import { SURE_ADDRESSES } from './constants';
import { Proposal } from './proposal';
import { SolanaAugmentedProvider } from '@saberhq/solana-contrib';
import { createNftOperationHandler } from '@metaplex-foundation/js-next';
import { Program } from '@project-serum/anchor';
import { OracleProgram } from './program';
import { Vote } from './vote';
import { PDA } from './pda';

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
		opts: anchor.web3.ConfirmOptions
	) {}

	static init({ connection, wallet, opts }: ProviderProps): Provider {
		return new Provider(connection, wallet, opts);
	}
}

export class SureOracleSDK {
	constructor(
		readonly provider: solana_contrib.AugmentedProvider,
		readonly oracle: anchor.Program<OracleIDL>,
		readonly pda: PDA
	) {}

	static init({
		provider,
	}: {
		provider: solana_contrib.Provider;
	}): SureOracleSDK {
		console.log('init sure oracle');

		// const program = anchor_contrib.newProgram<OracleProgram>(
		// 	OracleJSON,
		// 	SURE_ADDRESSES.Oracle,
		// 	provider
		// );
		const program = new anchor.Program<OracleIDL>(
			OracleJSON,
			SURE_ADDRESSES.Oracle
		);
		console.log('return finish');
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
