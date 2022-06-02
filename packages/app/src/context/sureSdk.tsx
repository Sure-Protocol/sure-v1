import { Wallet } from '@project-serum/anchor/dist/cjs/provider';
import { SurePool, IDL, SureSdk } from '@sure/sdk';
import * as anchor from '@project-serum/anchor';
import { PROGRAM_ID, PROGRAM_ID_STR } from '../utils/constants';
import React, { useContext, useEffect, useState } from 'react';
import { useConnection, useWallet } from '@solana/wallet-adapter-react';
import { PublicKey } from '@solana/web3.js';
import { NodeWallet } from '@metaplex/js';

export const SureSdkContext = React.createContext<undefined | SureSdk>(
	undefined
);

interface Props {
	children: JSX.Element;
}

export const SureSdkProvider: React.FunctionComponent<Props> = ({
	children,
}) => {
	const { connection } = useConnection();
	const wallet = useWallet();
	const [surePoolProgram, setSurePoolProgram] = useState<undefined | SureSdk>(
		undefined
	);

	useEffect(() => {
		if (wallet.publicKey) {
			setSurePoolProgram(SureSdk.init(connection, wallet));
		}
	}, [wallet]);

	return (
		<SureSdkContext.Provider value={surePoolProgram}>
			{children}
		</SureSdkContext.Provider>
	);
};

export const useSureSdk = (): SureSdk | undefined => {
	return useContext(SureSdkContext);
};
