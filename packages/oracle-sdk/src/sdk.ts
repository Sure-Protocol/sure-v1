import * as anchor from '@project-serum/anchor';
import { OracleIDL, OracleJSON } from '../../idls/oracle';
import { SURE_ADDRESSES } from './constants';
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
	constructor(
		readonly provider: Provider,
		readonly Oracle: anchor.Program<OracleIDL>
	) {}

	static init({ provider }: { provider: Provider }): SureOracleSDK {
		const oracleProgram = new anchor.Program<OracleIDL>(
			OracleJSON,
			SURE_ADDRESSES.Oracle,
			provider
		);
		return new SureOracleSDK(provider, oracleProgram);
	}

	proposal(): Proposal {
		return new Proposal(this);
	}
}
