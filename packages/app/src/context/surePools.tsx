import * as anchor from '@project-serum/anchor';
import { PublicKey } from '@solana/web3.js';
import { insurance, pool, PoolAccount } from '@sure/sdk';
import { createContext, useContext, useEffect, useState } from 'react';
import { useSureSdk } from './sureSdk';

export const SurePoolsContext = createContext<undefined | PoolAccount[]>(
	undefined
);

interface Props {
	children: JSX.Element;
}

export const SurePoolsProvider: React.FunctionComponent<Props> = ({
	children,
}) => {
	const [surePools, setSurePools] = useState<undefined | PoolAccount[]>(
		undefined
	);
	const sureSdk = useSureSdk();
	console.log('SurePoolsProvider');

	useEffect(() => {
		(async () => {
			if (sureSdk !== undefined) {
				const pools = await sureSdk.pool.getPools();
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

export const useSurePools = (): PoolAccount[] | undefined => {
	return useContext(SurePoolsContext);
};
