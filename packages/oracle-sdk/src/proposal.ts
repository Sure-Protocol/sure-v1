import * as anchor from '@project-serum/anchor';
import { Provider, SureOracleSDK } from './sdk';

export class Proposal {
	readonly program: anchor.Idl;
	constructor(readonly sdk: SureOracleSDK) {
		this.program = sdk.Oracle;
	}
}
