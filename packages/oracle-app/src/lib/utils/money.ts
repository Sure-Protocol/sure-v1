import * as anchor from '@project-serum/anchor';

import { SURE_MINT, type SureOracleSDK } from '@surec/oracle';
import * as spl from '@solana/spl-token';

const decimals10 = (decimals: number): anchor.BN => {
	return new anchor.BN(10).pow(new anchor.BN(decimals));
};

export const calculateAccountBalanceInDecimals = async (
	oracleSdk: SureOracleSDK
): Promise<anchor.BN> => {
	if (oracleSdk) {
		const userSureAta = await spl.getAssociatedTokenAddress(
			SURE_MINT,
			oracleSdk.provider.wallet.publicKey
		);
		const sureAtaAccount = await await spl.getAccount(oracleSdk.provider.connection, userSureAta);

		return calculateAmountInDecimals(oracleSdk, new anchor.BN(sureAtaAccount.amount));
	}
	return new anchor.BN(0);
};

export const calculateAmountInDecimals = async (
	oracleSdk: SureOracleSDK | undefined,
	amount: anchor.BN
): Promise<anchor.BN> => {
	if (oracleSdk) {
		const sureMint = await spl.getMint(oracleSdk.provider.connection, SURE_MINT);
		return amount.div(decimals10(sureMint.decimals));
	}
	return new anchor.BN(0);
};

export const calculateAmountInGivenDecimals = (amount: anchor.BN, decimals: number): anchor.BN => {
	return amount.div(decimals10(decimals));
};

export const calculateAccountBalanceFullAmount = async (
	oracleSdk: SureOracleSDK
): Promise<anchor.BN> => {
	if (oracleSdk) {
		const userSureAta = await spl.getAssociatedTokenAddress(
			SURE_MINT,
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
		const sureMint = await spl.getMint(oracleSdk.provider.connection, SURE_MINT);
		return amount.mul(decimals10(sureMint.decimals));
	}
	return new anchor.BN(0);
};
