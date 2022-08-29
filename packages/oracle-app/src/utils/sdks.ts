import type { SureOracleSDK } from '@surec/oracle';
import type { LockerWrapper } from '@tribecahq/tribeca-sdk';
import * as tribeca from '@tribecahq/tribeca-sdk';
import { getTestKeypairFromSeed } from '.';

export const getLockerSdk = async (
	oracleSdk: SureOracleSDK | undefined
): Promise<LockerWrapper | undefined> => {
	if (oracleSdk) {
		const tribecaSdk = tribeca.TribecaSDK.load({ provider: oracleSdk.provider });
		const base = getTestKeypairFromSeed(oracleSdk, 'sure_base_3');

		const lockerKey = tribeca.getLockerAddress(base.publicKey);
		const governorKey = tribeca.getGovernorAddress(base.publicKey);
		return await tribeca.LockerWrapper.load(tribecaSdk, lockerKey, governorKey);
	}
	return undefined;
};
