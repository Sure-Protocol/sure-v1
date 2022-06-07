import { createContext, useContext, useState } from 'react';

type ToggleType = [boolean, (val: boolean) => void];

const SearchContext = createContext<ToggleType>([false, () => {}]);

export const SearchProvider: React.FunctionComponent<{
	children: JSX.Element;
}> = ({ children }) => {
	const [isOpen, toggle] = useState(false);

	return (
		<SearchContext.Provider value={[isOpen, toggle]}>
			{children}
		</SearchContext.Provider>
	);
};

export const useToggle = (): ToggleType => {
	return useContext(SearchContext);
};
