import { SURE_MINT_DEV } from '$lib/constants';
import * as anchor from '@project-serum/anchor';
import type { SureOracleSDK } from '@surec/oracle';
import * as spl from './../../node_modules/@solana/spl-token';

const decimals10 = (decimals: number): anchor.BN => {
	return new anchor.BN(10).pow(new anchor.BN(decimals));
};

export const calculateAccountBalanceInDecimals = async (
	oracleSdk: SureOracleSDK
): Promise<anchor.BN> => {
	if (oracleSdk) {
		const userSureAta = await spl.getAssociatedTokenAddress(
			SURE_MINT_DEV,
			oracleSdk.provider.wallet.publicKey
		);
		const sureAtaAccount = await await spl.getAccount(oracleSdk.provider.connection, userSureAta);

		return calculateAmountInDecimals(oracleSdk, new anchor.BN(sureAtaAccount.amount));
	}
	return new anchor.BN(0);
};

export const calculateAmountInDecimals = async (
	oracleSdk: SureOracleSDK,
	amount: anchor.BN
): Promise<anchor.BN> => {
	if (oracleSdk) {
		const sureMint = await spl.getMint(oracleSdk.provider.connection, SURE_MINT_DEV);
		return amount.div(decimals10(sureMint.decimals));
	}
	return new anchor.BN(0);
};

export const calculateAccountBalanceFullAmount = async (
	oracleSdk: SureOracleSDK
): Promise<anchor.BN> => {
	if (oracleSdk) {
		const userSureAta = await spl.getAssociatedTokenAddress(
			SURE_MINT_DEV,
			oracleSdk.provider.wallet.publicKey
		);
		const sureAtaAccount = await await spl.getAccount(oracleSdk.provider.connection, userSureAta);

		return calculateFullAmount(oracleSdk, new anchor.BN(sureAtaAccount.amount));
	}
	return new anchor.BN(0);
};

export const calculateFullAmount = async (
	oracleSdk: SureOracleSDK,
	amount: anchor.BN
): Promise<anchor.BN> => {
	if (oracleSdk) {
		const sureMint = await spl.getMint(oracleSdk.provider.connection, SURE_MINT_DEV);
		return amount.mul(decimals10(sureMint.decimals));
	}
	return new anchor.BN(0);
};
