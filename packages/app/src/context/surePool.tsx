import { Wallet } from '@project-serum/anchor/dist/cjs/provider';
import * as sureSdk from '@sure/sdk';
import * as anchor from '@project-serum/anchor';
import { PROGRAM_ID_STR } from './../utils/constants';
import React, { useEffect, useState } from 'react';
import { useConnection, useWallet } from '@solana/wallet-adapter-react';

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

	const provider = new anchor.AnchorProvider(connection, wallet as Wallet, {
		skipPreflight: false,
	});

	const surePool = new anchor.Program<sureSdk.SurePool>(
		sureSdk.IDL,
		PROGRAM_ID_STR,
		provider
	);

	sureSdk.Tick
		//etSurePoolProgram(surePool);

		// useEffect(() => {
		// 	if (wallet !== null) {
		// 		const provider = new anchor.AnchorProvider(connection, wallet as Wallet, {
		// 			skipPreflight: false,
		// 		});
		// 		const surePool = new anchor.Program<sureSdk.SurePool>(
		// 			sureSdk.IDL,
		// 			PROGRAM_ID_STR,
		// 			provider
		// 		);
		// 		setSurePoolProgram(surePool);
		// 	}
		// }, [surePoolProgram]);

		.return(
			<SurePoolProgramContext.Provider value={surePoolProgram}>
				{children}
			</SurePoolProgramContext.Provider>
		);
};
