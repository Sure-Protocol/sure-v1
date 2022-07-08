import { createContext, useContext, useState } from 'react';

interface Contracts {}
export type UserContractsType = [Contracts[] | undefined];
const UserContractsContext = createContext<UserContractsType>(undefined);

export const UserContractsProvider: React.FunctionComponent<{
	children: JSX.Element;
}> = ({ children }) => {
	const [userContracts, setUserContracts] =
		useState<UserContractsType[0]>(undefined);
	return (
		<UserContractsContext.Provider value={[userContracts]}>
			{children}
		</UserContractsContext.Provider>
	);
};

export const useUserContracts = (): UserContractsType => {
	return useContext(UserContractsContext);
};
