import React, { useContext, useEffect, useState } from 'react';
import { TokenInfo, TokenListProvider, ENV } from '@solana/spl-token-registry';
import { useConnection } from '@solana/wallet-adapter-react';

const TokensContext = React.createContext<Map<string, TokenInfo> | undefined>(
	undefined
);

interface Props {
	children: JSX.Element;
}

export type TokensMap = Map<string, TokenInfo>;

const convertRPCTOCluster: Record<string, string> = {
	'https://api.devnet.solana.com': 'devnet',
	'https://api.testnet.solana.com': 'testnet',
	'https://api.mainnet-beta.solana.com': 'mainnet',
};

export const TokensProvider: React.FunctionComponent<Props> = ({
	children,
}) => {
	const { connection } = useConnection();
	const [tokens, setTokens] = useState<TokensMap>(new Map());

	useEffect(() => {
		new TokenListProvider().resolve().then((tokens) => {
			const tokenList = tokens
				.filterByClusterSlug(convertRPCTOCluster[connection.rpcEndpoint])
				.getList();

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

export const useTokens = (): TokensMap | undefined => {
	return useContext(TokensContext);
};
