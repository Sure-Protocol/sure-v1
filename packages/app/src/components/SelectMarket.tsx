import down from './../assets/icons/expand_more.svg';
import { css } from '@emotion/css';
import { theme } from './Themes';
import SearchMarket from './SearchMarket';
import { useState } from 'react';
import { useTokens } from '../context/tokens';

export const SelectMarket = () => {
	const [showPools, toggleMarket] = useState(false);
	const tokens = useTokens();
	return (
		<div>
			<div
				className={css`
					border-radius: 5px;
					margin-right: 1rem;
					flex-grow: 2;
					background-color: ${theme.colors.sureBlue4};
					padding: 4px;
					display: flex;
					flex-direction: row;
				`}
			>
				<button
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
					onClick={() => toggleMarket(!showPools)}
				>
					<div className="sure-token">sol</div>
					<div className="sure-token--name">
						<p className="p--margin-0 p--white p--bold">SOL</p>
					</div>
					<div className="sure-icon">
						<img src={down} alt="logo" className="icon-small" />
					</div>
				</button>

				<input
					className={css`
						background-color: ${theme.colors.sureBlue4};
						color: ${theme.colors.sureWhite};
						cursor: pointer;
						border-radius: 5px;
						border-width: 1px;
						border-color: transparent;
						padding: 5px;

						flex-grow: 2;
						text-align: right;
						background-color: transparent;
						font-family: 'arvo-bold';

						&:focus {
							outline: none;
						}
					`}
					placeholder="0.00"
					typeof="decimals"
				/>
				<button
					className={css`
						background-color: ${theme.colors.sureBlue4};
						color: ${theme.colors.sureWhite};
						cursor: pointer;
						border-radius: 5px;
						border-width: 1px;
						border-color: transparent;
						padding: 5px;
					`}
				>
					<p className="p--margin-0">USDC</p>
				</button>
			</div>
			{showPools && <SearchMarket toggle={() => toggleMarket(false)} />}
		</div>
	);
};
