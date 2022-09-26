import {
	calculateAccountBalanceInDecimals,
	calculateAmountInDecimals,
	getEscrowSdk,
} from './../lib/utils/index';
import { SURE_MINT, type SureOracleSDK } from '@surec/oracle';
import { writable } from 'svelte/store';
import * as spl from '@solana/spl-token';
import { newEvent } from './event';
import { BN } from 'bn.js';

export type TokenState = {
	mintDecimals: number;
	isLoading: boolean;
	veSureAmount: string;
	sureAmount: string;
};

const defaultTokenState: TokenState = {
	mintDecimals: 6,
	isLoading: false,
	veSureAmount: '__',
	sureAmount: '__',
};

export const tokenState = writable<TokenState>(defaultTokenState);

export const hydrateTokenState = async (oracleSdk: SureOracleSDK) => {
	try {
		tokenState.set({
			mintDecimals: 6,
			isLoading: true,
			veSureAmount: '__',
			sureAmount: '__',
		});
		const decimals = (
			await spl.getMint(oracleSdk.provider.connection, SURE_MINT)
		).decimals;

		const escrowSdk = await getEscrowSdk(oracleSdk);
		oracleSdk.provider.connection.rpcEndpoint;
		let votingPower;
		try {
			votingPower = await escrowSdk?.calculateVotingPower(new Date());
		} catch {
			votingPower = new BN(0);
		}

		const veSureAmount = await calculateAmountInDecimals(
			oracleSdk,
			votingPower
		);
		const sureAmount = await calculateAccountBalanceInDecimals(oracleSdk);
		tokenState.set({
			mintDecimals: decimals,
			isLoading: false,
			veSureAmount: veSureAmount.toString(),
			sureAmount: sureAmount.toString(),
		});
	} catch (err) {
		tokenState.set({
			mintDecimals: 6,
			isLoading: false,
			veSureAmount: '__',
			sureAmount: '__',
		});
		newEvent.set({
			name: 'failed to load tokens',
			message: err as string,
			status: 'error',
		});
	}
};
