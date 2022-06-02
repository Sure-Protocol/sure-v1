import * as anchor from '@project-serum/anchor';
import { token } from '@project-serum/anchor/dist/cjs/utils';
import { PublicKey } from '@solana/web3.js';
import { createContext, useContext, useState } from 'react';
import { useSureSdk } from './sureSdk';

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
const InsuranceContractContext = createContext<InsuranceContract | undefined>(
	undefined
);

export const InsuranceContractProvider: React.FunctionComponent<{
	children: JSX.Element;
}> = ({ children }) => {
	const [insuranceContract, setInsuranceContract] = useState(undefined);
	const sureSdk = useSureSdk();
	const updateInsuranceContract = (pool: PublicKey, tokenMint: PublicKey) => {
		
        const insuredAmount = await sureSdk?.insurance.getInsuredAmount(
			pool,
			tokenMint
		);
        const contractExpiry = await sureSdk?.insurance.
        const contractExpiry = sureSdk?.insurance.

	};
	return (
		<InsuranceContractContext.Provider value={insuranceContract}>
			{children}
		</InsuranceContractContext.Provider>
	);
};

export const useInsuranceContract = (): undefined | InsuranceContract => {
	return useContext(InsuranceContractContext);
};
