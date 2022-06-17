import { css, cx } from '@emotion/css';
import { useInsuranceContract } from '../context/insuranceContract';
import { explorerLink } from '../utils/links';
import { prettyPublicKey } from '../utils/publickey';
import { theme } from './Themes';
import { TokensMap, useTokens } from '../context/tokens';
import TokenIcon from './TokenIcon';
import React, { useEffect, useMemo, useRef, useState } from 'react';
import { TokenInfo } from '@solana/spl-token-registry';
import { useSearchTokenToggle } from '../context/searchTokenToggle';
import { PublicKey } from '@solana/web3.js';

interface MarketListProps {
	tokens: TokensMap | undefined;
}

const MarketListItem: React.FunctionComponent<{
	token: TokenInfo;
}> = ({ token }) => {
	const [contract, setContract] = useInsuranceContract();
	const searchTokenToggle = useSearchTokenToggle();
	return (
		<button
			className={css`
				display: flex;
				flex-direction: row;
				align-items: center;
				justify-content: space-between;
				// Button
				background-color: transparent;
				cursor: pointer;
				border: none;

				&:hover {
					background-color: ${theme.colors.sureBlue3};
				}
			`}
			onClick={() => {
				searchTokenToggle.setSelectedToken(token);
				searchTokenToggle.toggle(false);
			}}
		>
			<TokenIcon tokenAddress={new PublicKey(token.address)} />
			<p className="p--medium p--white">{token.symbol}</p>
			<a
				className="p--small a--no-highlight"
				target="_blank"
				href={explorerLink(new PublicKey(token.address))}
			>
				{prettyPublicKey(new PublicKey(token.address))}
			</a>
		</button>
	);
};

const MarketList: React.FunctionComponent<MarketListProps> = ({ tokens }) => {
	return (
		<div
			className={css`
				display: flex;
				flex-direction: column;
				padding: 10px;
				overflow: scroll;
				height: 300px;
			`}
		>
			{tokens &&
				Array.from(tokens?.keys() ?? []).map((token) => {
					const tokenInfo = tokens.get(token);
					if (tokenInfo != undefined) {
						return <MarketListItem token={tokenInfo} />;
					}
				})}
		</div>
	);
};

const SearchTokens: React.FunctionComponent<{
	parentRef: React.RefObject<HTMLDivElement>;
}> = ({ parentRef }) => {
	const tokens = useTokens();
	const searchTokenToggle = useSearchTokenToggle();
	const ref = useRef<HTMLDivElement>(null);
	const [searchTerm, setSearchTerm] = useState('');
	const [filteredTokens, setFilteredTokens] = useState<TokensMap | undefined>(
		undefined
	);

	useEffect(() => {
		if (ref.current && parentRef?.current) {
			ref.current.style.top = `${parentRef?.current.offsetTop}px`;
		}
	}, [parentRef?.current]);

	useEffect(() => {
		setFilteredTokens(undefined);
		if (tokens && searchTerm.length > 2) {
			const _filteredTokens = new Map();
			tokens.forEach((token) => {
				if (token.name.toLowerCase().trim().includes(searchTerm)) {
					_filteredTokens.set(token.address, token);
				}
			});
			setFilteredTokens(_filteredTokens);
		}
	}, [searchTerm]);

	return (
		<div
			ref={ref}
			className={css`
				z-index: 1;
				transform: translateX(-50%);
				left: 50%;
				position: absolute;
				width: 300px;
				height: 400px;
				background-color: ${theme.colors.sureBlue4};
				border-radius: 5px;
			`}
		>
			<button
				className={cx(
					'button--round',
					css`
						position: absolute;
						right: 10px;
						top: 5px;
					`
				)}
				onClick={() => {
					searchTokenToggle.toggle(false);
				}}
			>
				X
			</button>
			<div
				className={css`
					padding: 1rem;
				`}
			>
				<div
					className={css`
						display: flex;
						flex-direction: row;
						color: ${theme.colors.sureTextGray};
						border-bottom: 2px solid;
						padding: 10px;
					`}
				>
					<input
						className={css`
							color: ${theme.colors.sureWhite};
							cursor: pointer;
							border-radius: 5px;
							border-width: 1px;
							border-color: transparent;
							padding: 5px;

							flex-grow: 2;
							text-align: left;
							background-color: transparent;
							font-family: 'arvo';

							&:focus {
								outline: none;
							}
						`}
						onChange={(e) => setSearchTerm(e.target.value.toLowerCase())}
						placeholder="Search token name"
					/>
				</div>
				{tokens && <MarketList tokens={filteredTokens} />}
			</div>
		</div>
	);
};

export default SearchTokens;
