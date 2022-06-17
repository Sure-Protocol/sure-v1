import { Wallet } from '@project-serum/anchor/dist/cjs/provider';
import { SurePool, IDL, SureSdk } from '@surec/sdk';
import * as anchor from '@project-serum/anchor';
import React, { useContext, useEffect, useState } from 'react';
import { useConnection, useWallet } from '@solana/wallet-adapter-react';

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
			/// @ts-ignore
			setSurePoolProgram(
				SureSdk.init(connection, wallet, process.env.PROGRAM_ID)
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
