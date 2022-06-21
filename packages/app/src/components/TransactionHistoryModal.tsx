import { css, cx } from '@emotion/css';
import * as anchor from '@project-serum/anchor';
import { useConnection, useWallet } from '@solana/wallet-adapter-react';
import { Connection } from '@solana/web3.js';
import { SureDate } from '@surec/sdk';
import { useEffect, useState } from 'react';
import { useWatch } from 'react-hook-form';
import { useTransactionHistory } from '../context/transactionHistory';
import { prettyPublicKey } from '../utils/publickey';
import { theme } from './Themes';

const transactionItemStyle = ({ isSuccess }: { isSuccess: boolean }) => css`
	display: flex;
	flex-direction: column;
	justify-content: space-around;
	padding: 10px;
	margin-bottom: 10px;
	background-color: ${isSuccess
		? theme.colors.sureSuccess
		: theme.colors.sureError};
`;

const TransactionHistoryModal: React.FunctionComponent = () => {
	const [transactions, setTransactions] = useState([]);
	const [transactionHistory, getMore] = useTransactionHistory();
	return (
		<div
			className={css`
				position: relative;
				z-index: 3;
				left: 50%;
				top: 30%;
				transform: translate3d(-50%, -100%, 0);
				width: 500px;
				height: 400px;
				background-color: ${theme.colors.sureBlue4};
				border-radius: 10px;
				padding: 10px;
				overflow: scroll;
				@media (max-width: ${theme.screenSize.small}) {
					width: 90%;
				}
			`}
		>
			<div>
				<p>Transactions for user </p>
				<div
					className={css`
						position: absolute;
						right: 10px;
						top: 10px;
					`}
				>
					x
				</div>
			</div>
			<div>
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
									className={css`
										display: flex;
										flex-direction: row;
										justify-content: flex-start;
									`}
								>
									<p className="p--white">
										{SureDate.new(tx.blockTime * 1000).toLocaleString()}
									</p>
									<p className="p--white">{prettyPublicKey(tx.programId)}</p>
								</div>
								<ul
									className={css`
										list-style: decimal;
										color: ${theme.colors.sureWhite};
									`}
								>
									{tx.prettyInstructions.map((ix) => (
										<li>
											<p className="p--margin-0 p--white">{ix.title}</p>
										</li>
									))}
								</ul>
							</li>
						);
					})}
				</ul>
			</div>
		</div>
	);
};

export default TransactionHistoryModal;
