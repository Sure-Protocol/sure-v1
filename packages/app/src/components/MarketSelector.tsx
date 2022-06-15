import { css } from '@emotion/css';
import { theme } from './Themes';
import down from './../assets/icons/down.svg';
import { useToggle } from '../context/searchToggle';
import React, { useEffect, useMemo, useState } from 'react';
import { PoolInformation } from '@surec/sdk';
import { FieldValues, UseFormRegister } from 'react-hook-form';
import { TokensMap, useTokens } from '../context/tokens';
import { TokenInfo } from '@solana/spl-token-registry';
import TokenIcon from './TokenIcon';
import { PublicKey } from '@solana/web3.js';
import TokenIconInfo from './TokenIconInfo';
import { selectRandomToken } from '../utils/string';

interface MarketSelectorProps {
	marketRef?: React.RefObject<HTMLDivElement>;
	pool?: PoolInformation;
	register: UseFormRegister<FieldValues>;
	includeTokenMint?: boolean;
}

const MarketSelector: React.FunctionComponent<MarketSelectorProps> = ({
	marketRef,
	pool,
	register,
	includeTokenMint = true,
}) => {
	const [isOpen, toggle] = useToggle();
	const tokens = useTokens();
	const token = useMemo(() => {
		if (pool === undefined) {
			return selectRandomToken(tokens);
		} else {
			return tokens?.get(pool?.tokenMint.toBase58());
		}
	}, [tokens, pool]);
	return (
		<div
			ref={marketRef}
			className={css`
				border-radius: 5px;
				background-color: ${theme.colors.sureBlue4};
				padding: 4px;
				display: flex;
				flex-direction: row;
			`}
		>
			<div
				className={css`
					background-color: ${theme.colors.sureBlue4};
					color: ${theme.colors.sureWhite};
					cursor: pointer;
					border-radius: 5px;
					border-width: 1px;
					border-color: transparent;
					padding: 5px;

					display: flex;
					align-items: center;
					justify-content: center;

					&:hover {
						background-color: ${theme.colors.sureBlue2};
					}
				`}
				onClick={() => toggle(true)}
			>
				<div className={css``}>
					<p
						className={`p--margin-0 p--small ${
							pool?.name != undefined ? 'p--white' : ''
						}`}
					>
						{pool?.name ?? 'Select protocol'}
					</p>
				</div>
				<div className="sure-icon">
					<img src={down} alt="logo" className="icon-small" />
				</div>
			</div>

			<input
				{...register('amount', { min: 0, valueAsNumber: true })}
				className={'input-number-field'}
				placeholder="0.00"
				typeof="decimals"
			/>
			{includeTokenMint && token && (
				<TokenIconInfo token={token} isVisible={false} />
			)}
		</div>
	);
};

export default MarketSelector;
