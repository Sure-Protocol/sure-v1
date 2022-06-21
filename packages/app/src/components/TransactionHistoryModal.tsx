import { css, cx } from '@emotion/css';
import * as anchor from '@project-serum/anchor';
import { useConnection, useWallet } from '@solana/wallet-adapter-react';
import { Connection, PublicKey } from '@solana/web3.js';
import { SureDate } from '@surec/sdk';
import { useEffect, useState } from 'react';
import { useWatch } from 'react-hook-form';
import { useTransactionHistory } from '../context/transactionHistory';
import { prettyPublicKey } from '../utils/publickey';
import { theme } from './Themes';
import SuccessCircle from '../assets/icons/successCircle.svg';
import ErrorCircle from '../assets/icons/errorCircle.svg';
import ShowMore from '../assets/icons/showMore.svg';
import CloseIcon from '../assets/icons/close.svg';
import MainButton from './MainButton';
import { round } from 'lodash';
import TokenIcon from './TokenIcon';

const transactionItemStyle = ({ isSuccess }: { isSuccess: boolean }) => css`
	display: flex;
	flex-direction: column;
	justify-content: space-around;
	padding: 10px;
	margin-bottom: 10px;
`;

const statusBorder = ({ isSuccess }: { isSuccess: boolean }) => css`
	border-left: 3px solid
		${isSuccess ? theme.colors.sureSuccess : theme.colors.sureError};
	border-bottom: 3px solid
		${isSuccess ? theme.colors.sureSuccess : theme.colors.sureError};
	border-radius: 2px;
	margin-bottom: 10px;
	:hover {
		cursor: pointer;
		border-left: 3px solid
			${isSuccess ? theme.colors.sureSuccess100 : theme.colors.sureError100};
		border-bottom: 3px solid
			${isSuccess ? theme.colors.sureSuccess100 : theme.colors.sureError100};
	}
`;

const TransactionHistoryModal: React.FunctionComponent = () => {
	const [transactions, setTransactions] = useState([]);
	const [transactionHistory, getMore] = useTransactionHistory();
	const [showTxDetails, setShowTxDetails] = useState<Map<string, boolean>>(
		new Map()
	);

	useEffect(() => {
		setShowTxDetails(
			new Map(
				transactionHistory.reduce((map, tx) => {
					map.set(tx.signatures[0], false);
					return map;
				}, new Map())
			)
		);

		console.log('transactionHistory2: ', transactionHistory);
	}, [transactionHistory]);

	const toggleTxDetails = ({ sig }: { sig: string }) => {
		const newIsOpen = showTxDetails.set(sig, !showTxDetails.get(sig));
		setShowTxDetails(new Map(newIsOpen));
	};

	return (
		<div
			className={css`
				position: relative;
				z-index: 3;
				left: 50%;
				top: 50%;
				transform: translate3d(-50%, -80%, 0);
				width: 500px;
				height: 600px;
				background-color: ${theme.colors.sureBlue4};
				border-radius: 10px;
				padding: 10px;

				@media (max-width: ${theme.screenSize.small}) {
					width: 90%;
				}
			`}
		>
			<div
				className={css`
					padding: 10px;
				`}
			>
				<div>
					<p className="p--bold p--large">Transactions </p>
				</div>
				<div
					className={css`
						display: flex;
						flex-direction: row;
						justify-content: space-evenly;
					`}
				>
					<p className="p--padding-s">Status</p>
					<p className="p--padding-s">Date</p>
					<p className="p--padding-s">Program</p>
					<p className="p--padding-s">Changed</p>
					<p className="p--padding-s">Token</p>
					<p className="p--padding-s"></p>
				</div>

				<div
					className={css`
						position: absolute;
						right: 10px;
						top: 10px;
					`}
				>
					<img
						className={css`
							right: 0;
							bottom: 0;
							width: 24px;
							height: 24px;
							margin-right: 10px;
						`}
						src={CloseIcon}
						alt={'Show transaction details'}
					/>
				</div>
			</div>
			<div
				className={css`
					height: 450px;
					overflow: scroll;
				`}
			>
				<ul
					className={css`
						display: flex;
						flex-direction: column;
						list-style: none;
						padding: 0;
					`}
				>
					{transactionHistory.map((tx) => {
						return (
							<li className={transactionItemStyle({ isSuccess: tx.success })}>
								<div
									className={cx(
										css`
											display: flex;
											flex-direction: row;
											justify-content: space-evenly;
											align-items: center;
											padding-top: 10px;
											padding-bottom: 10px;

											border-radius: 2px;
											margin-bottom: 10px;
											:hover {
												cursor: pointer;
											}
										`,
										statusBorder({ isSuccess: tx.success })
									)}
									onClick={() => toggleTxDetails({ sig: tx.signatures[0] })}
								>
									{tx.success ? (
										<img
											className={css`
												right: 0;
												bottom: 0;
												width: 24px;
												height: 24px;
												z-index: 0;
												margin-right: 10px;
												padding: 10px;
											`}
											src={SuccessCircle}
											alt={'Success circle'}
										/>
									) : (
										<img
											className={css`
												right: 0;
												bottom: 0;
												width: 24px;
												height: 24px;
												margin-right: 10px;
												padding: 10px;
											`}
											src={ErrorCircle}
											alt={'Error circle'}
										/>
									)}
									<div
										className={css`
											display: flex;
											flex-direction: column;
											width: 82px;
										`}
									>
										<p className="p--white p--small p--margin-0">
											{SureDate.new(tx.blockTime * 1000).toLocaleDateString()}
										</p>
										<p className="p--white p--small p--margin-0">
											{SureDate.new(tx.blockTime * 1000).toLocaleTimeString()}
										</p>
									</div>

									<p className="p--white">{prettyPublicKey(tx.programId)}</p>

									<div
										className={css`
											width: 96px;
										`}
									>
										{tx.tokenChange && (
											<div
												className={css`
													display: flex;
													justify-content: flex-start;
													align-items: center;
													flex-direction: row;
												`}
											>
												<p className="p--white text--width__xlarge">
													{round(
														tx.postBalanceToken?.uiAmount -
															tx.preBalanceToken?.uiAmount,
														5
													)}
												</p>
												<TokenIcon
													tokenAddress={tx.preBalanceToken?.mint ?? ''}
												/>
											</div>
										)}
									</div>
									<img
										className={css`
											right: 0;
											bottom: 0;
											width: 24px;
											height: 24px;
											margin-right: 10px;
										`}
										src={ShowMore}
										alt={'Show transaction details'}
									/>
								</div>
								{showTxDetails.get(tx.signatures[0]) && (
									<ul
										className={css`
											list-style: decimal;
											color: ${theme.colors.sureWhite};
											border-left: 5px solid ${theme.colors.sureError};
											margin-left: 20px;
										`}
									>
										{tx.prettyInstructions.map((ix) => (
											<li
												className={css`
													margin: 0;
												`}
												key={ix.title}
											>
												<p className="p--margin-0 p--white">{ix.title}</p>
											</li>
										))}
									</ul>
								)}
							</li>
						);
					})}
					<div
						className={css`
							display: flex;
							justify-content: center;
						`}
					>
						<MainButton isSubmit={false} onClick={() => getMore()}>
							<p className="p--white">Load more</p>
						</MainButton>
					</div>
				</ul>
			</div>
		</div>
	);
};

export default TransactionHistoryModal;
