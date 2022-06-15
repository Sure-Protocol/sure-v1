import { TokenInfo } from '@solana/spl-token-registry';
import { createContext, useContext, useState } from 'react';

type SearchTokenToggle = {
	isOpen: boolean;
	toggle: (val: boolean) => void;
	selectedToken: TokenInfo | undefined;
	setSelectedToken: (token: TokenInfo) => void;
};

const SearchTokenToggleContext = createContext<SearchTokenToggle>({
	isOpen: false,
	toggle: () => {},
	selectedToken: undefined,
	setSelectedToken: () => {},
});

export const SearchTokenToggleProvider: React.FunctionComponent<{
	children: JSX.Element;
}> = ({ children }) => {
	const [isOpen, toggle] = useState(false);
	const [token, setToken] = useState<TokenInfo | undefined>(undefined);

	return (
		<SearchTokenToggleContext.Provider
			value={{
				isOpen,
				toggle,
				selectedToken: token,
				setSelectedToken: setToken,
			}}
		>
			{children}
		</SearchTokenToggleContext.Provider>
	);
};

export const useSearchTokenToggle = (): SearchTokenToggle => {
	return useContext(SearchTokenToggleContext);
};
