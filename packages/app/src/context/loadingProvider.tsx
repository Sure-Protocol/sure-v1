import { createContext, useContext, useState } from 'react';

type LoadingType = [boolean, (loading: boolean) => void];
const LoadingContext = createContext<LoadingType>([false, () => {}]);

export const LoadingProvider: React.FunctionComponent<{
	children: JSX.Element;
}> = ({ children }) => {
	const [isLoading, setIsLoading] = useState(false);
	return (
		<LoadingContext.Provider value={[isLoading, setIsLoading]}>
			{children}
		</LoadingContext.Provider>
	);
};

export const useIsLoading = (): LoadingType => {
	return useContext(LoadingContext);
};
