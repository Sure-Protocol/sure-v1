import { css, cx } from '@emotion/css';
import { PoolInformation } from '@surec/sdk';
import { useInsuranceContract } from '../context/insuranceContract';
import { useSurePools } from '../context/surePools';
import { explorerLink } from '../utils/links';
import { prettyPublicKey } from '../utils/publickey';
import { theme } from './Themes';
import { useToggle } from '../context/searchToggle';
import { usePool } from '../context/surePool';
import { useTokens } from '../context/tokens';
import TokenIcon from './TokenIcon';
import React, { useEffect, useRef } from 'react';
import { prettyPrintPremium } from '../utils/premium';

interface MarketListProps {
	surePools: PoolInformation[];
}

const MarketListItem: React.FunctionComponent<{
	surePool: PoolInformation;
}> = ({ surePool }) => {
	const [contract, setContract] = useInsuranceContract();
	const [pool, setPool] = usePool();
	const [isOpen, toggle] = useToggle();
	const tokens = useTokens();
	return (
		<button
			className={css`
				display: flex;
				flex-direction: row;
				align-items: center;
				justify-content: space-around;
				// Button
				background-color: transparent;
				cursor: pointer;
				border: none;

				&:hover {
					background-color: ${theme.colors.sureBlue3};
				}
			`}
			onClick={() => {
				setContract(surePool);
				setPool(surePool);
				toggle(false);
			}}
		>
			<div
				className={css`
					margin-right: 20px;
				`}
			>
				<TokenIcon tokenAddress={surePool.tokenMint.toBase58()} />
			</div>

			<div
				className={css`
					display: flex;
					flex-direction: column;
					text-align: left;
					margin-right: 20px;
					min-width: 100px;
				`}
			>
				<p className="p--medium p--white text--margin-vertical__small">
					{surePool.name}
				</p>
				<a
					className="p--small a--no-highlight"
					target="_blank"
					href={explorerLink(surePool.smartContract)}
				>
					{prettyPublicKey(surePool.smartContract)}
				</a>
			</div>

			<div
				className={css`
					display: flex;
					flex-direction: column;
					text-align: left;
				`}
			>
				<p className="p--small p--margin-0 ">{`Utilization: ${
					(100 * parseInt(surePool.usedLiquidity)) /
					parseInt(surePool.liquidity)
				}%`}</p>
				<p className="p--small p--margin-0 ">
					{`Liquidity: ${parseInt(surePool.liquidity.toString())} ${
						tokens?.get(surePool.tokenMint?.toBase58())?.symbol ?? '?'
					}`}
				</p>
				<p className="p--small p--margin-0 ">
					{`Used Liquidity: ${parseInt(surePool.usedLiquidity)} ${
						tokens?.get(surePool.tokenMint?.toBase58())?.symbol ?? '?'
					}`}
				</p>

				<p className="p--small p--margin-0">{`Premium: ${prettyPrintPremium(
					surePool.lowestPremium
				)}bp`}</p>
			</div>
		</button>
	);
};

const MarketList: React.FunctionComponent<MarketListProps> = ({
	surePools,
}) => {
	return (
		<div
			className={css`
				display: flex;
				flex-direction: column;
				padding: 10px;
			`}
		>
			{surePools.map((pool) => (
				<MarketListItem surePool={pool} />
			))}
		</div>
	);
};

const SearchMarket: React.FunctionComponent<{
	parentRef: React.RefObject<HTMLDivElement>;
}> = ({ parentRef }) => {
	const [surePools] = useSurePools();
	const [isOpen, toggle] = useToggle();
	const ref = useRef<HTMLDivElement>(null);
	useEffect(() => {
		if (ref.current && parentRef?.current) {
			ref.current.style.top = `${parentRef?.current?.offsetTop}px`;
			ref.current.style.width = `${parentRef?.current?.offsetWidth}px`;
		}
	}, [parentRef?.current]);

	return (
		<div
			ref={ref}
			className={css`
				z-index: 3;
				position: absolute;
				width: 340px;
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
					toggle(false);
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
						placeholder="Search Pool"
					/>
				</div>
				{surePools && <MarketList surePools={surePools} />}
			</div>
		</div>
	);
};

export default SearchMarket;
