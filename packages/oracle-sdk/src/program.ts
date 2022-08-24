import * as anchor_contrib from '@saberhq/anchor-contrib';
import { OracleIDL } from '../../idls/oracle';

export type OracleTypes = anchor_contrib.AnchorTypes<
	OracleIDL,
	{
		proposal: ProposalType;
		revealedVoteArray: RevealedVoteArray;
		voteAccount: VoteAccount;
	}
>;

type Accounts = OracleTypes['Accounts'];
export type ProposalType = Accounts['Proposal'];
export type RevealedVoteArray = Accounts['RevealedVoteArray'];
export type VoteAccount = Accounts['VoteAccount'];
export type OracleProgram = OracleTypes['Program'];
