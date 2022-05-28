import React, { useEffect, useState } from 'react';
import { TokenInfo, TokenListProvider, ENV } from '@solana/spl-token-registry';

export const TokensContext = React.createContext<
	Map<string, TokenInfo> | undefined
>(undefined);

interface Props {
	children: JSX.Element;
}

export const TokensProvider: React.FunctionComponent<Props> = ({
	children,
}) => {
	const [tokens, setTokens] = useState<Map<string, TokenInfo> | undefined>(
		new Map()
	);

	useEffect(() => {
		new TokenListProvider().resolve().then((tokens) => {
			const tokenList = tokens.filterByChainId(ENV.MainnetBeta).getList();

			setTokens(
				tokenList.reduce((map, item) => {
					map.set(item.address, item);
					return map;
				}, new Map())
			);
		});
	}, [setTokens]);

	return (
		<TokensContext.Provider value={tokens}>{children}</TokensContext.Provider>
	);
};
