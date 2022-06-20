import * as anchor from '@project-serum/anchor';
import { useConnection, useWallet } from '@solana/wallet-adapter-react';
import { Connection } from '@solana/web3.js';
import { useEffect, useState } from 'react';
import { useWatch } from 'react-hook-form';

const TransactionHistory: React.FunctionComponent = () => {
	const [transactions, setTransactions] = useState([]);
	const connection = useConnection();
	const wallet = useWallet();
	useEffect(() => {
		(async () => {
			const trxs = await connection.connection.getSignaturesForAddress(
				wallet.publicKey
			);
			console.log('trxs: ', trxs);
			trxs.forEach(async (tx) => {
				const txRes = await connection.connection.getTransaction(tx.signature);
				console.log('tx program Ids: ', txRes.transaction.message.programIds());
			});
		})();
	}, []);
	return <div></div>;
};
