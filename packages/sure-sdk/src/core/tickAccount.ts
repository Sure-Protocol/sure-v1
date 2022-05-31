import * as anchor from '@project-serum/anchor';
import { Connection } from '@solana/web3.js';
import { SurePool } from '../anchor/types/sure_pool';
import { Common } from './commont';
import { SureSdk, Tick } from './index';

export class TickAccount extends Common {
	constructor(
		readonly program: anchor.Program<SurePool>,
		readonly connection: Connection,
		readonly wallet: anchor.Wallet
	) {
		super(program, connection, wallet);
	}
}
