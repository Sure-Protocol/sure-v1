import { Wallet } from '@project-serum/anchor/dist/cjs/provider';
import { SurePool, IDL, SureSdk } from '@sure/sdk';
import * as anchor from '@project-serum/anchor';
import { PROGRAM_ID, PROGRAM_ID_STR } from './../utils/constants';
import React, { useEffect, useState } from 'react';
import { useConnection, useWallet } from '@solana/wallet-adapter-react';
import { PublicKey } from '@solana/web3.js';
import { NodeWallet } from '@metaplex/js';

export const SurePoolProgramContext = React.createContext<
	undefined | anchor.Program<SurePool>
>(undefined);

interface Props {
	children: JSX.Element;
}

export const SurePoolProgramProvider: React.FunctionComponent<Props> = ({
	children,
}) => {
	const { connection } = useConnection();
	const wallet = useWallet();
	const [surePoolProgram, setSurePoolProgram] = useState<
		undefined | anchor.Program<SurePool>
	>(undefined);
	console.log('SurePool: ', IDL);
	console.log('wallet: ', wallet);

	useEffect(() => {
		if (wallet !== null) {
			const sureSdk = SureSdk.init(connection, wallet as NodeWallet);
		}
	}, [wallet]);

	return (
		<SurePoolProgramContext.Provider value={surePoolProgram}>
			{children}
		</SurePoolProgramContext.Provider>
	);
};
