import { PoolAccount } from '@sure/sdk';
import { createContext, useContext, useState } from 'react';

type PoolSelectorType = [
	pool: PoolAccount | undefined,
	setPool: (data: PoolAccount) => void
];
const PoolContext = createContext<PoolSelectorType>([undefined, () => {}]);

export const PoolProvider: React.FunctionComponent<{
	children: JSX.Element;
}> = ({ children }) => {
	const [pool, setPool] = useState<PoolAccount | undefined>(undefined);
	return (
		<PoolContext.Provider value={[pool, setPool]}>
			{children}
		</PoolContext.Provider>
	);
};

export const usePool = (): PoolSelectorType => {
	return useContext(PoolContext);
};
