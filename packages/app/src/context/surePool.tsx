import { Wallet } from '@project-serum/anchor/dist/cjs/provider';
import { IDL } from '@sure/sdk';
import * as anchor from '@project-serum/anchor';
import { PROGRAM_ID_STR } from './../utils/constants';
import React, { useEffect, useState } from 'react';
import { useConnection, useWallet } from '@solana/wallet-adapter-react';
import { PublicKey } from '@solana/web3.js';

export const SurePoolProgramContext = React.createContext<
	undefined | anchor.Program
>(undefined);

interface Props {
	children: JSX.Element;
}

export const SurePoolProgramProvider: React.FunctionComponent<Props> = ({
	children,
}) => {
	const { connection } = useConnection();
	const { publicKey, sendTransaction, wallet } = useWallet();
	const [surePoolProgram, setSurePoolProgram] = useState<
		undefined | anchor.Program
	>(undefined);
	console.log('SurePool: ', IDL);
	console.log('wallet: ', wallet);

	if (wallet) {
		const provider = new anchor.AnchorProvider(connection, wallet, {
			skipPreflight: false,
		});
		const sureProgram = new anchor.Program(IDL, PROGRAM_ID_STR, provider);
		const smartContractAddress = anchor.web3.PublicKey.default;
	}

	return (
		<SurePoolProgramContext.Provider value={surePoolProgram}>
			{children}
		</SurePoolProgramContext.Provider>
	);
};
