import * as anchor from '@project-serum/anchor';
import { PublicKey } from '@solana/web3.js';
import { insurance, pool } from '@sure/sdk';
import { createContext, useContext, useEffect, useState } from 'react';
import { useSureSdk } from './sureSdk';

export const SurePoolsContext = createContext<undefined | SurePool[]>(
	undefined
);

interface Props {
	children: JSX.Element;
}

export interface SurePool {
	name: string;
	insuranceFee: number;
	liquidity: anchor.BN;
	usedLiquidity: anchor.BN;
	smartContract: PublicKey;
}
export const SurePoolsProvider: React.FunctionComponent<Props> = ({
	children,
}) => {
	const [surePools, setSurePools] = useState<undefined | SurePool[]>(undefined);
	const sureSdk = useSureSdk();
	console.log('SurePoolsProvider');

	useEffect(() => {
		(async () => {
			if (sureSdk !== undefined) {
				const surePoolsAccountPDA = await sureSdk.pool.getSurePoolsPDA();
				const surePoolsAccount = await sureSdk.program.account.surePools.fetch(
					surePoolsAccountPDA
				);
				const surePoolsT: SurePool[] = [];
				surePoolsAccount.pools.forEach(async (poolPDA) => {
					try {
						const poolAcccount =
							await sureSdk.program.account.poolAccount.fetch(poolPDA);
						surePoolsT.push(poolAcccount);
					} catch (err) {
						console.log(
							'could not fetch ' + poolPDA.toBase58() + ' cause:  ',
							err
						);
					}
				});
				setSurePools(surePoolsT);
			}
		})();
	}, [sureSdk]);

	return (
		<SurePoolsContext.Provider value={surePools}>
			{children}
		</SurePoolsContext.Provider>
	);
};

export const useSurePools = (): SurePool[] | undefined => {
	return useContext(SurePoolsContext);
};
