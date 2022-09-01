import { web3 } from '@project-serum/anchor';
import { SureOracleSDK } from '../../packages/oracle-sdk/src';
import { TransactionReceipt } from '@saberhq/solana-contrib';

export const createTestConfig = async (
	protocolAuthority: web3.PublicKey,
	oracleSdk: SureOracleSDK,
	mint: web3.PublicKey
): Promise<TransactionReceipt> => {
	const createProposal = await oracleSdk.config().initializeOracleConfig({
		protocolAuthority,
		tokenMint: mint,
	});

	return createProposal.confirm();
};
