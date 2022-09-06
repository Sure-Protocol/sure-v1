import { web3 } from '@project-serum/anchor';
import * as oracle from '../../packages/oracle-sdk/dist/esm/oracle-sdk/src';
import * as anchor from '@project-serum/anchor';
import { TransactionReceipt } from '@saberhq/solana-contrib';

export const createTestProposal = async (
	oracleSdk: oracle.SureOracleSDK,
	mint: web3.PublicKey,
	proposalName: string,
	proposedStake: anchor.BN
): Promise<TransactionReceipt> => {
	const createProposal = await oracleSdk.proposal().proposeVote({
		name: proposalName,
		description: 'how many eggs are in the basket',
		stake: proposedStake,
		mint,
	});

	return createProposal.confirm();
};
