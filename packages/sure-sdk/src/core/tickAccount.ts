import * as anchor from '@project-serum/anchor';
import { Connection } from '@solana/web3.js';
import { SurePool } from '../anchor/types/sure_pool';
import { SureSdk, Tick } from './index';

export class TickAccount {
	constructor(
		readonly program: anchor.Program<SurePool>,
		readonly connection: Connection,
		readonly wallet: anchor.Wallet
	) {}
}
