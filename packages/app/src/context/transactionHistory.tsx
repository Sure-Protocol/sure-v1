import { useConnection, useWallet } from '@solana/wallet-adapter-react';
import { Context } from '@solana/web3.js';
import { createContext, useContext, useEffect, useState } from 'react';

const TransactionHistoryContext = createContext<undefined>(undefined);

export const TransactionHistoryProvider: React.FunctionComponent<{
	children: JSX.Element;
}> = ({ children }) => {
	const connection = useConnection();
	const wallet = useWallet();
	const [transactionHistory, setTransactionHistory] =
		useState<undefined>(undefined);
	console.log('TransactionHistoryProvider');
	useEffect(() => {
		(async () => {
			const sigs = await connection.connection.getSignaturesForAddress(
				wallet.publicKey
			);
			console.log('sigs ', sigs);

			const sureSigs = await sigs
				.map(async (sig) => {
					const tx = await connection.connection.getTransaction(sig.signature);
					return tx.transaction.message;
				})
				.filter(async (txMessage) => {
					return (await txMessage)
						.programIds()
						.some((id) => id.toBase58() === process.env.PROGRAM_ID);
				});
			console.log('sureSigs', sureSigs);
		})();
	}, [wallet]);
	return (
		<TransactionHistoryContext.Provider value={[transactionHistory]}>
			{children}
		</TransactionHistoryContext.Provider>
	);
};

export const useTransactionHistory = (): undefined => {
	return useContext(TransactionHistoryContext);
};
