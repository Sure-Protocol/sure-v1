import { BN } from 'bn.js';
import { SURE_MINT, type SureOracleSDK } from '@surec/oracle';
import * as spl from '@solana/spl-token';

const decimals10 = (decimals: number): BN => {
	return new BN(10).pow(new BN(decimals));
};

export const calculateAccountBalanceInDecimals = async (oracleSdk: SureOracleSDK): Promise<BN> => {
	if (oracleSdk) {
		const userSureAta = await spl.getAssociatedTokenAddress(
			SURE_MINT,
			oracleSdk.provider.wallet.publicKey
		);
		const sureAtaAccount = await await spl.getAccount(oracleSdk.provider.connection, userSureAta);

		return calculateAmountInDecimals(oracleSdk, new BN(sureAtaAccount.amount));
	}
	return new BN(0);
};

export const calculateAmountInDecimals = async (
	oracleSdk: SureOracleSDK | undefined,
	amount: BN
): Promise<BN> => {
	if (oracleSdk) {
		const sureMint = await spl.getMint(oracleSdk.provider.connection, SURE_MINT);
		return amount.div(decimals10(sureMint.decimals));
	}
	return new BN(0);
};

export const calculateAmountInGivenDecimals = (amount: BN, decimals: number): BN => {
	return amount.div(decimals10(decimals));
};

export const calculateAccountBalanceFullAmount = async (oracleSdk: SureOracleSDK): Promise<BN> => {
	if (oracleSdk) {
		const userSureAta = await spl.getAssociatedTokenAddress(
			SURE_MINT,
			oracleSdk.provider.wallet.publicKey
		);
		const sureAtaAccount = await await spl.getAccount(oracleSdk.provider.connection, userSureAta);

		return calculateFullAmount(oracleSdk, new BN(sureAtaAccount.amount));
	}
	return new BN(0);
};

export const calculateFullAmount = async (oracleSdk: SureOracleSDK, amount: BN): Promise<BN> => {
	if (oracleSdk) {
		const sureMint = await spl.getMint(oracleSdk.provider.connection, SURE_MINT);
		return amount.mul(decimals10(sureMint.decimals));
	}
	return new BN(0);
};
