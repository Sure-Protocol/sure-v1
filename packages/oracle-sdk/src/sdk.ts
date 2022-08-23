import * as anchor from '@project-serum/anchor';
import { oracle } from '../idls/oracle.json';
import { Proposal } from './proposal';

// checkpoint : generate oracle idl and use it in sdk
export class Provider {
	constructor(
		readonly connection: anchor.web3.Connection,
		readonly wallet: anchor.Wallet,
		opts: anchor.web3.ConfirmOptions
	) {}
}

export class SureOracleSDK {
	constructor(readonly provider: Provider, readonly Oracle: anchor.Idl) {}

	static init({ provider }: { provider: Provider }): SureOracleSDK {
		return new SureOracleSDK(provider, oracle);
	}

	proposal(): Proposal {
		return new Proposal(this);
	}
}
