import { BASE_PK, type SureOracleSDK } from '@surec/oracle';
import {
	LockerWrapper,
	VoteEscrow,
	getLockerAddress,
	getGovernorAddress,
	getEscrowAddress,
	TribecaSDK
} from '@tribecahq/tribeca-sdk';

export const getLockerSdk = async (
	oracleSdk: SureOracleSDK | undefined
): Promise<LockerWrapper | undefined> => {
	if (oracleSdk) {
		const tribecaSdk = TribecaSDK.load({ provider: oracleSdk.provider });
		const basePk = BASE_PK;

		const lockerKey = getLockerAddress(basePk);
		const governorKey = getGovernorAddress(basePk);

		return await LockerWrapper.load(tribecaSdk, lockerKey, governorKey);
	}
	return undefined;
};

export const getEscrowSdk = async (
	oracleSdk: SureOracleSDK | undefined
): Promise<VoteEscrow | undefined> => {
	if (oracleSdk) {
		const tribecaSdk = TribecaSDK.load({ provider: oracleSdk.provider });
		const basePk = BASE_PK;

		const lockerKey = getLockerAddress(basePk);
		const governorKey = getGovernorAddress(basePk);
		const escrow = getEscrowAddress(lockerKey, oracleSdk.provider.wallet.publicKey);

		return new VoteEscrow(tribecaSdk, lockerKey, governorKey, escrow, oracleSdk.provider.walletKey);
	}
	return undefined;
};
