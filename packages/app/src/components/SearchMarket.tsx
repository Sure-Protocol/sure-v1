import { css, cx } from '@emotion/css';
import { PoolAccount } from '@sure/sdk';
import { useSurePools } from '../context/surePools';
import { explorerLink } from '../utils/links';
import { prettyPublicKey } from '../utils/publickey';
import { theme } from './Themes';

interface MarketListProps {
	surePools: PoolAccount[];
}

const MarketListItem: React.FunctionComponent<{ surePool: PoolAccount }> = ({
	surePool,
}) => {
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
		>
			<p className="p--medium p--white">{surePool.name}</p>
			<a
				className="p--small a--no-highlight"
				target="_blank"
				href={explorerLink(surePool.smartContract)}
			>
				{prettyPublicKey(surePool.smartContract)}
			</a>

			<div
				className={css`
					display: flex;
					flex-direction: column;
					justify-content: center;
					align-items: flex-start;
				`}
			>
				<p className="p--small p--margin-0">{`Liquidity: ${surePool.liquidity.toString()}`}</p>
				<p className="p--small p--margin-0">{`Premium: ${surePool.premiumRate}%`}</p>
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

const SearchMarket: React.FunctionComponent<{ toggle: () => void }> = ({
	toggle,
}) => {
	const surePools = useSurePools();
	return (
		<div
			className={css`
				transform: translate3d(0%, -10%, 0);
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
				onClick={() => toggle()}
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
						placeholder="> Search protocol"
					/>
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
						placeholder="USDC"
					/>
				</div>
				{surePools && <MarketList surePools={surePools} />}
			</div>
		</div>
	);
};

export default SearchMarket;
