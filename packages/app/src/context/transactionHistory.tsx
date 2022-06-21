import { useConnection, useWallet } from '@solana/wallet-adapter-react';
import {
	Context,
	Message,
	ParsedTransactionMeta,
	ParsedTransactionWithMeta,
	PublicKey,
	SignaturesForAddressOptions,
	TokenBalance,
} from '@solana/web3.js';
import { last, values } from 'lodash';
import { createContext, useContext, useEffect, useState } from 'react';
import { useTokens } from './tokens';

export type TransactionHistoryValues = [SureTransaction[], () => void];

interface SureTokenBalance {
	mint: string;
	mintName: string;
	mintUrl: string;
	owner: string;
	uiAmount: number;
}

interface PrettyInstruction {
	title: string;
}
export interface SureTransaction {
	blockTime: number | null;
	programId: PublicKey;
	fee: number;
	signatures: string[];
	tokenChange: boolean;
	preBalanceToken: SureTokenBalance;
	postBalanceToken: SureTokenBalance;
	prettyInstructions: PrettyInstruction[];
	success: boolean;
}

const TransactionHistoryContext = createContext<TransactionHistoryValues>([
	[],
	() => {},
]);

export const TransactionHistoryProvider: React.FunctionComponent<{
	children: JSX.Element;
}> = ({ children }) => {
	const connection = useConnection();
	const tokens = useTokens();
	const wallet = useWallet();
	const [transactionHistory, setTransactionHistory] = useState<
		TransactionHistoryValues[0]
	>([]);

	const [lastSignature, setLastSignature] = useState<string | undefined>(
		undefined
	);

	const txTokenBalanceToUserBalanceToken = (
		tokenBalances: TokenBalance[]
	): SureTokenBalance => {
		return tokenBalances
			.filter((elem) => elem.owner === wallet.publicKey.toBase58())
			.map((elem) => {
				const tokenInfo = tokens.get(elem.mint);
				return {
					mint: elem.mint,
					mintName: tokenInfo?.name ?? 'unknown',
					mintUrl: tokenInfo?.logoURI ?? '',
					owner: elem.owner,
					uiAmount: elem.uiTokenAmount.uiAmount,
				};
			})[0];
	};

	const extractInstructions = (logMessages: string[]): PrettyInstruction[] => {
		return logMessages
			.filter((lm) => lm.includes('log: Instruction'))
			.map((lm) => {
				return { title: lm.match(/\w+$/g)[0] };
			});
	};

	const loadMoreTxs = async () => {
		const signatureForAddressOptions: SignaturesForAddressOptions = {
			limit: 5,
		};
		if (lastSignature) {
			signatureForAddressOptions.before = lastSignature;
		}
		const sigs = await connection.connection.getSignaturesForAddress(
			wallet.publicKey,
			signatureForAddressOptions
		);
		const txs = await connection.connection.getParsedTransactions(
			sigs.map((sig) => sig.signature)
		);
		const suretxs = txs.filter((tx) => {
			return (
				tx.transaction.message.instructions[0]?.programId.toBase58() ===
				process.env.PROGRAM_ID
			);
		});
		console.log(suretxs[0]);
		const prettySureTxs = await Promise.all(
			suretxs.map(async (tx): Promise<SureTransaction> => {
				const userPreTokenBalance = txTokenBalanceToUserBalanceToken(
					tx.meta.preTokenBalances
				);

				const userPostTokenBalance = txTokenBalanceToUserBalanceToken(
					tx.meta.postTokenBalances
				);

				return {
					blockTime: tx.blockTime,
					programId: tx.transaction.message.instructions[0]?.programId,
					fee: tx.meta.fee,
					signatures: tx.transaction.signatures,
					tokenChange:
						userPreTokenBalance?.uiAmount !== undefined ? true : false,
					preBalanceToken: userPreTokenBalance,
					postBalanceToken: userPostTokenBalance,
					prettyInstructions: extractInstructions(tx.meta.logMessages),
					success: tx.meta.err === null,
				};
			})
		);
		console.log('prettySureTxs: ', prettySureTxs);
		setLastSignature(sigs[sigs.length - 1].signature);
		setTransactionHistory(prettySureTxs);
	};

	useEffect(() => {
		(async () => {
			await loadMoreTxs();
		})();
	}, [wallet]);

	return (
		<TransactionHistoryContext.Provider
			value={[transactionHistory, loadMoreTxs]}
		>
			{children}
		</TransactionHistoryContext.Provider>
	);
};

export const useTransactionHistory = (): TransactionHistoryValues => {
	return useContext(TransactionHistoryContext);
};
