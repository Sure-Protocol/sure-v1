import * as anchor from '@project-serum/anchor';
import { token } from '@project-serum/anchor/dist/cjs/utils';
import { PublicKey } from '@solana/web3.js';
import { createContext, useContext, useEffect, useState } from 'react';
import { useSureSdk } from './sureSdk';
import * as sureSDK from '@sure/sdk';
import { PoolAccount } from '@sure/sdk';

interface InsuranceContractData {
	iconUrl: string;
	name: string;
	ticker: string;
	amountInsured: anchor.BN;
	denomination: string;
	expiry: string;
	pool: PublicKey;
	tokenMint: PublicKey;
}

interface InsuranceContract {
	contract: InsuranceContractData;
	setInsuranceContract: () => void;
}
const InsuranceContractContext = createContext<PoolAccount | undefined>(
	undefined
);

export const InsuranceContractProvider: React.FunctionComponent<{
	children: JSX.Element;
}> = ({ children }) => {
	const [insuranceContract, setInsuranceContract] = useState(undefined);
	const sureSdk = useSureSdk();

	useEffect(() => {}, []);

	return (
		<InsuranceContractContext.Provider value={insuranceContract}>
			{children}
		</InsuranceContractContext.Provider>
	);
};

export const useInsuranceContract = (): undefined | PoolAccount => {
	return useContext(InsuranceContractContext);
};
