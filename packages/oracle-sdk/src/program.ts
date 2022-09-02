import * as anchor_contrib from '@saberhq/anchor-contrib';
import * as oracle from '../../idls/oracle';

export type OracleTypes = anchor_contrib.AnchorTypes<
	oracle.Oracle,
	{
		proposal: ProposalType;
		revealedVoteArray: RevealedVoteArray;
		voteAccount: VoteAccount;
		config: ConfigType;
	}
>;

type Accounts = OracleTypes['Accounts'];
export type ProposalType = Accounts['proposal'];
export type ConfigType = Accounts['config'];
export type RevealedVoteArray = Accounts['revealedVoteArray'];
export type VoteAccount = Accounts['voteAccount'];
export type OracleProgram = OracleTypes['Program'];
