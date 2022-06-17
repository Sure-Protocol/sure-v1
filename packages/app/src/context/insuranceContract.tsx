import * as anchor from '@project-serum/anchor';
import { createContext, useContext, useEffect, useState } from 'react';
import {
	PoolAccount,
	PoolInformation,
	PoolInsuranceContract,
} from '@surec/sdk';
import { useSureSdk } from './sureSdk';
import { useSurePools } from './surePools';

type InsuranceContract = [
	PoolInsuranceContract | undefined,
	(data: PoolInformation) => void
];

const InsuranceContractContext = createContext<InsuranceContract>([
	undefined,
	() => {},
]);

export const InsuranceContractProvider: React.FunctionComponent<{
	children: JSX.Element;
}> = ({ children }) => {
	const sureSdk = useSureSdk();
	const [surePools] = useSurePools();
	const [insuranceContract, setInsuranceContract] = useState<
		PoolInsuranceContract | undefined
	>(undefined);

	const updateInsuranceContract = async (pool: PoolInformation) => {
		if (sureSdk) {
			const poolPDA = await sureSdk?.pool.getPoolPDA(pool.smartContract);
			const insuranceContract =
				await sureSdk?.insurance.getPoolInsuranceContractInfo(
					poolPDA,
					pool.tokenMint
				);
			setInsuranceContract(insuranceContract);
		}
	};

	useEffect(() => {
		if (surePools) {
			updateInsuranceContract(surePools[0]);
		}
	});

	return (
		<InsuranceContractContext.Provider
			value={[insuranceContract, updateInsuranceContract]}
		>
			{children}
		</InsuranceContractContext.Provider>
	);
};

export const useInsuranceContract = (): InsuranceContract => {
	return useContext(InsuranceContractContext);
};
