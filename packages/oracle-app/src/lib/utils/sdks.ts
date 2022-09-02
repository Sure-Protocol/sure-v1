import { BASE_PK, type SureOracleSDK } from '@surec/oracle';
import type { LockerWrapper } from '@tribecahq/tribeca-sdk';
import * as tribeca from '@tribecahq/tribeca-sdk';

export const getLockerSdk = async (
	oracleSdk: SureOracleSDK | undefined
): Promise<LockerWrapper | undefined> => {
	if (oracleSdk) {
		const tribecaSdk = tribeca.TribecaSDK.load({ provider: oracleSdk.provider });
		const basePk = BASE_PK;

		const lockerKey = tribeca.getLockerAddress(basePk);
		const governorKey = tribeca.getGovernorAddress(basePk);
		return await tribeca.LockerWrapper.load(tribecaSdk, lockerKey, governorKey);
	}
	return undefined;
};
