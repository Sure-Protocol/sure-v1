import { Wallet } from '@project-serum/anchor/dist/cjs/provider';
import { SurePool, IDL, SureSdk } from '@surec/sdk';
import * as anchor from '@project-serum/anchor';
import React, { useContext, useEffect, useState } from 'react';
import {
	useAnchorWallet,
	useConnection,
	useWallet,
} from '@solana/wallet-adapter-react';
import { PublicKey } from '@solana/web3.js';

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
	const wallet = useAnchorWallet();
	const [surePoolProgram, setSurePoolProgram] = useState<undefined | SureSdk>(
		undefined
	);
	useEffect(() => {
		if (wallet?.publicKey) {
			const programId = new PublicKey(process.env.PROGRAM_ID);
			/// @ts-ignore
			setSurePoolProgram(
				SureSdk.init(connection, wallet as anchor.Wallet, programId)
			);
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
