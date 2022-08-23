import * as anchor from '@project-serum/anchor';
import { OracleIDL } from '../../idls/oracle';
import { Provider, SureOracleSDK } from './sdk';

export class Vote {
	readonly program: anchor.Program<OracleIDL>;
	constructor(readonly sdk: SureOracleSDK) {
		this.program = sdk.Oracle;
	}
}
