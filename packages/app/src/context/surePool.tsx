import { PoolAccount, PoolInformation } from '@surec/sdk';
import { createContext, useContext, useState } from 'react';

export type PoolSelectorType = [
	PoolInformation | undefined,
	(data: PoolInformation) => void
];
const PoolContext = createContext<PoolSelectorType>([undefined, () => {}]);

export const PoolProvider: React.FunctionComponent<{
	children: JSX.Element;
}> = ({ children }) => {
	const [pool, setPool] = useState<PoolInformation | undefined>(undefined);
	return (
		<PoolContext.Provider value={[pool, setPool]}>
			{children}
		</PoolContext.Provider>
	);
};

export const usePool = (): PoolSelectorType => {
	return useContext(PoolContext);
};
