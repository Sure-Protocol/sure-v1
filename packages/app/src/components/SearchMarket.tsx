import { css } from '@emotion/css';
import { theme } from './Themes';

const SearchMarket = () => {
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

				<div
					className={css`
						display: flex;
						flex-direction: column;
					`}
				>
					<div
						className={css`
							display: flex;
							flex-direction: row;
						`}
					>
						<p>Serum</p>
						<p> 23wjhewjoe2o3j2m</p>
						<div>
							<p>1,000,000 TVL</p>
						</div>
					</div>
				</div>
			</div>
		</div>
	);
};

export default SearchMarket;
