import { TokenInfo, TokenInfoMap } from '@solana/spl-token-registry';
import { TokensMap } from '../context/tokens';

export const selectRandomToken = (
	tokens: TokensMap | undefined
): TokenInfo | undefined => {
	if (tokens === undefined) {
		return undefined;
	}
	const tokenKeys = Array.from(tokens.keys());
	const randNum = Math.round(Math.random() * tokenKeys.length);
	return tokens.get(tokenKeys[randNum]);
};

export const cutString = (str: string, chars: number) => {
	if (str.length > chars) {
		return `${str.slice(0, chars)}...`;
	}
	return str;
};
