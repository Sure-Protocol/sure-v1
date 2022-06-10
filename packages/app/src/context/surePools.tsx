import * as anchor from '@project-serum/anchor';
import { PublicKey } from '@solana/web3.js';
import { insurance, pool, PoolAccount, PoolInformation } from '@surec/sdk';
import { createContext, useContext, useEffect, useState } from 'react';
import { useSureSdk } from './sureSdk';

export const SurePoolsContext = createContext<undefined | PoolInformation[]>(
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

	useEffect(() => {
		(async () => {
			console.log('sureSdk: ', sureSdk);
			if (sureSdk !== undefined) {
				const pools = await sureSdk.pool.getPoolsInformation();
				console.log('pools: ', pools);
				setSurePools(pools);
			}
		})();
	}, [sureSdk]);

	return (
		<SurePoolsContext.Provider value={surePools}>
			{children}
		</SurePoolsContext.Provider>
	);
};

export const useSurePools = (): PoolInformation[] | undefined => {
	return useContext(SurePoolsContext);
};
