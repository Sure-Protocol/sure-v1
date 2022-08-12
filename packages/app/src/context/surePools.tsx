import * as anchor from '@project-serum/anchor';
import { useWallet } from '@solana/wallet-adapter-react';
import { PublicKey } from '@solana/web3.js';
import { insurance, pool, PoolAccount, PoolInformation } from '@surec/sdk';
import { createContext, useContext, useEffect, useState } from 'react';
import { useWatch } from 'react-hook-form';
import { useIsLoading } from './loadingProvider';
import { useSureSdk } from './sureSdk';

type SurePoolsContextType = [
	PoolInformation[],
	(data: PoolInformation[]) => void
];

export const SurePoolsContext = createContext<SurePoolsContextType | undefined>(
	undefined
);

interface Props {
	children: JSX.Element;
}

export const SurePoolsProvider: React.FunctionComponent<Props> = ({
	children,
}) => {
	const [surePools, setSurePools] = useState<undefined | PoolInformation[]>(
		undefined
	);
	const sureSdk = useSureSdk();
	const wallet = useWallet();
	const [isLoading, setIsLoading] = useIsLoading();

	useEffect(() => {
		(async () => {
			if (sureSdk !== undefined) {
				setIsLoading(true);
				const pools = await sureSdk.pool.getTokenPoolsInformationV2();
				setSurePools(pools);
				setIsLoading(false);
			}
		})();
	}, [sureSdk, wallet]);

	return (
		<SurePoolsContext.Provider value={[surePools, setSurePools]}>
			{children}
		</SurePoolsContext.Provider>
	);
};

export const loadSurePools = async () => {
	const sureSdk = useSureSdk();
	const [pools, setPools] = useContext(SurePoolsContext);
	if (sureSdk !== undefined) {
		const pools = await sureSdk.pool.getTokenPoolsInformationV2();
		setPools(pools);
	}
};

export const useSurePools = (): SurePoolsContextType | undefined => {
	return useContext(SurePoolsContext);
};
