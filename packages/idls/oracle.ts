export type OracleIDL = {
	version: '0.1.0';
	name: 'oracle';
	instructions: [
		{
			name: 'proposeVote';
			docs: [
				'Propose vote',
				'',
				'proposes a vote or observation that the holder of veSure can',
				'vote on.',
				'',
				'### paramters',
				'* `ctx`: Context',
				'* `name`: Name of the observation',
				'* `description`: Clear description about the event',
				'* `stake`: The amount staked on event. In BN:  x*10^{decimals}'
			];
			accounts: [
				{
					name: 'proposer';
					isMut: true;
					isSigner: true;
				},
				{
					name: 'proposal';
					isMut: true;
					isSigner: false;
					pda: {
						seeds: [
							{
								kind: 'const';
								type: 'string';
								value: 'sure-oracle';
							},
							{
								kind: 'arg';
								type: 'string';
								path: 'name';
							}
						];
					};
				},
				{
					name: 'revealVoteArray';
					isMut: true;
					isSigner: false;
					pda: {
						seeds: [
							{
								kind: 'const';
								type: 'string';
								value: 'sure-oracle-reveal-array';
							},
							{
								kind: 'arg';
								type: 'string';
								path: 'name';
							}
						];
					};
				},
				{
					name: 'proposerAccount';
					isMut: true;
					isSigner: false;
				},
				{
					name: 'proposalVaultMint';
					isMut: false;
					isSigner: false;
				},
				{
					name: 'proposalVault';
					isMut: true;
					isSigner: false;
					pda: {
						seeds: [
							{
								kind: 'const';
								type: 'string';
								value: 'sure-oracle';
							},
							{
								kind: 'account';
								type: 'publicKey';
								account: 'Mint';
								path: 'proposal_vault_mint';
							}
						];
					};
				},
				{
					name: 'tokenProgram';
					isMut: false;
					isSigner: false;
				},
				{
					name: 'associatedTokenProgram';
					isMut: false;
					isSigner: false;
				},
				{
					name: 'rent';
					isMut: false;
					isSigner: false;
				},
				{
					name: 'systemProgram';
					isMut: false;
					isSigner: false;
				}
			];
			args: [
				{
					name: 'name';
					type: 'string';
				},
				{
					name: 'description';
					type: 'string';
				},
				{
					name: 'stake';
					type: 'u64';
				}
			];
		},
		{
			name: 'submitVote';
			docs: [
				'Submit vote',
				'',
				'lets user vote blindly on a proposal using a vote hash',
				'',
				'### Parameters',
				'* `ctx` - context',
				'* `vote_hash` - hash of vote with secret salt'
			];
			accounts: [
				{
					name: 'voter';
					isMut: true;
					isSigner: true;
				},
				{
					name: 'voterAccount';
					isMut: true;
					isSigner: false;
				},
				{
					name: 'locker';
					isMut: false;
					isSigner: false;
				},
				{
					name: 'userEscrow';
					isMut: false;
					isSigner: false;
				},
				{
					name: 'proposal';
					isMut: true;
					isSigner: false;
				},
				{
					name: 'proposalVault';
					isMut: true;
					isSigner: false;
				},
				{
					name: 'proposalVaultMint';
					isMut: false;
					isSigner: false;
				},
				{
					name: 'voteAccount';
					isMut: true;
					isSigner: false;
					pda: {
						seeds: [
							{
								kind: 'const';
								type: 'string';
								value: 'sure-oracle-vote';
							},
							{
								kind: 'account';
								type: 'publicKey';
								account: 'Proposal';
								path: 'proposal';
							},
							{
								kind: 'account';
								type: 'publicKey';
								path: 'voter';
							}
						];
					};
				},
				{
					name: 'tokenProgram';
					isMut: false;
					isSigner: false;
				},
				{
					name: 'rent';
					isMut: false;
					isSigner: false;
				},
				{
					name: 'systemProgram';
					isMut: false;
					isSigner: false;
				}
			];
			args: [
				{
					name: 'voteHash';
					type: 'bytes';
				}
			];
		},
		{
			name: 'updateVote';
			docs: [
				'Updates vote',
				'',
				'updates the vote hash of the previous submitted vote',
				'',
				'### parameters',
				'* `ctx` - context',
				'* `vote_hash` - hash of vote with secret salt'
			];
			accounts: [
				{
					name: 'voter';
					isMut: true;
					isSigner: true;
				},
				{
					name: 'proposal';
					isMut: false;
					isSigner: false;
				},
				{
					name: 'voteAccount';
					isMut: true;
					isSigner: false;
					pda: {
						seeds: [
							{
								kind: 'const';
								type: 'string';
								value: 'sure-oracle-vote';
							},
							{
								kind: 'account';
								type: 'publicKey';
								account: 'Proposal';
								path: 'proposal';
							},
							{
								kind: 'account';
								type: 'publicKey';
								path: 'voter';
							}
						];
					};
				},
				{
					name: 'systemProgram';
					isMut: false;
					isSigner: false;
				}
			];
			args: [
				{
					name: 'voteHash';
					type: 'string';
				}
			];
		},
		{
			name: 'cancelVote';
			docs: [
				'cancel vote',
				'',
				'a user can cancel the vote in the voting period',
				'',
				'### parameters',
				'* `ctx` - CancelVote context'
			];
			accounts: [
				{
					name: 'voter';
					isMut: true;
					isSigner: true;
				},
				{
					name: 'voterAccount';
					isMut: true;
					isSigner: false;
				},
				{
					name: 'proposalVault';
					isMut: true;
					isSigner: false;
				},
				{
					name: 'proposalVaultMint';
					isMut: false;
					isSigner: false;
				},
				{
					name: 'proposal';
					isMut: true;
					isSigner: false;
				},
				{
					name: 'voteAccount';
					isMut: true;
					isSigner: false;
				},
				{
					name: 'systemProgram';
					isMut: false;
					isSigner: false;
				},
				{
					name: 'tokenProgram';
					isMut: false;
					isSigner: false;
				}
			];
			args: [];
		},
		{
			name: 'revealVote';
			docs: [
				'reveal vote',
				'',
				'let the user reveal the vote when the voting period is over',
				'the user can only receive rewards if revealed',
				'',
				'### parameters',
				'* `ctx` - RevealVote context',
				'* `salt` - the salt used to hash the vote',
				'* `vote`- the actual vote value'
			];
			accounts: [
				{
					name: 'voter';
					isMut: true;
					isSigner: true;
				},
				{
					name: 'proposal';
					isMut: false;
					isSigner: false;
					pda: {
						seeds: [
							{
								kind: 'const';
								type: 'string';
								value: 'sure-oracle';
							},
							{
								kind: 'account';
								type: 'string';
								account: 'Proposal';
								path: 'proposal.name';
							}
						];
					};
				},
				{
					name: 'revealVoteArray';
					isMut: true;
					isSigner: false;
					pda: {
						seeds: [
							{
								kind: 'const';
								type: 'string';
								value: 'sure-oracle';
							},
							{
								kind: 'account';
								type: 'publicKey';
								account: 'Proposal';
								path: 'proposal';
							}
						];
					};
				},
				{
					name: 'voteAccount';
					isMut: true;
					isSigner: false;
					pda: {
						seeds: [
							{
								kind: 'const';
								type: 'string';
								value: 'sure-oracle-vote';
							},
							{
								kind: 'account';
								type: 'publicKey';
								account: 'Proposal';
								path: 'proposal';
							},
							{
								kind: 'account';
								type: 'publicKey';
								path: 'voter';
							}
						];
					};
				},
				{
					name: 'systemProgram';
					isMut: false;
					isSigner: false;
				}
			];
			args: [
				{
					name: 'salt';
					type: 'string';
				},
				{
					name: 'vote';
					type: 'i64';
				}
			];
		},
		{
			name: 'finalizeVoteResults';
			docs: [
				'finalize vote results',
				'',
				'after the reveal period the proposal can be finalized',
				'from this point on it is not possible to reveal the vote',
				'',
				'the proposer reward and scale parameter is calculated',
				'',
				'### parameters',
				'*  `ctx` - the Finalize Vote context'
			];
			accounts: [
				{
					name: 'finalizer';
					isMut: true;
					isSigner: true;
				},
				{
					name: 'proposal';
					isMut: true;
					isSigner: false;
				},
				{
					name: 'revealedVotes';
					isMut: false;
					isSigner: false;
				},
				{
					name: 'systemProgram';
					isMut: false;
					isSigner: false;
				}
			];
			args: [];
		},
		{
			name: 'finalizeVote';
			docs: [
				'finalize vote',
				'',
				'after the vote results are finalized the voters can calculate',
				'their vote share and close their vote account',
				'',
				'### parameters',
				'* `ctx` - Finalize Vote context'
			];
			accounts: [
				{
					name: 'signer';
					isMut: true;
					isSigner: true;
				},
				{
					name: 'voteAccount';
					isMut: true;
					isSigner: false;
				},
				{
					name: 'proposal';
					isMut: false;
					isSigner: false;
				},
				{
					name: 'systemProgram';
					isMut: false;
					isSigner: false;
				}
			];
			args: [];
		},
		{
			name: 'collectProposerReward';
			docs: [
				'collect proposer reward',
				'',
				'after the vote results are finalized the proposer is free to',
				'collect the reward at any time'
			];
			accounts: [
				{
					name: 'proposer';
					isMut: true;
					isSigner: true;
				},
				{
					name: 'proposerAccount';
					isMut: true;
					isSigner: false;
				},
				{
					name: 'proposal';
					isMut: true;
					isSigner: false;
				},
				{
					name: 'proposalVault';
					isMut: true;
					isSigner: false;
				},
				{
					name: 'proposalVaultMint';
					isMut: true;
					isSigner: false;
				},
				{
					name: 'systemProgram';
					isMut: false;
					isSigner: false;
				},
				{
					name: 'tokenProgram';
					isMut: false;
					isSigner: false;
				}
			];
			args: [];
		},
		{
			name: 'collectVoteReward';
			docs: [
				'collect vote reward',
				'',
				'after the vote results are finalized the voter can collect rewards'
			];
			accounts: [
				{
					name: 'voter';
					isMut: true;
					isSigner: true;
				},
				{
					name: 'voterAccount';
					isMut: true;
					isSigner: false;
				},
				{
					name: 'voteAccount';
					isMut: true;
					isSigner: false;
				},
				{
					name: 'proposal';
					isMut: true;
					isSigner: false;
				},
				{
					name: 'proposalVaultMint';
					isMut: false;
					isSigner: false;
				},
				{
					name: 'proposalVault';
					isMut: false;
					isSigner: false;
				},
				{
					name: 'systemProgram';
					isMut: false;
					isSigner: false;
				},
				{
					name: 'tokenProgram';
					isMut: false;
					isSigner: false;
				}
			];
			args: [];
		}
	];
	accounts: [
		{
			name: 'Proposal';
			type: {
				kind: 'struct';
				fields: [
					{
						name: 'bump';
						docs: ['bump for verification'];
						type: 'u8';
					},
					{
						name: 'bumpArray';
						type: {
							array: ['u8', 1];
						};
					},
					{
						name: 'name';
						docs: ['name of vote'];
						type: 'string';
					},
					{
						name: 'description';
						docs: ['description of vote'];
						type: 'string';
					},
					{
						name: 'proposedResult';
						docs: ['Proposed result'];
						type: 'i64';
					},
					{
						name: 'proposer';
						docs: ['user who proposed the vote'];
						type: 'publicKey';
					},
					{
						name: 'proposedStaked';
						docs: ['amount staked by propose Q32.32'];
						type: 'u64';
					},
					{
						name: 'vault';
						docs: ['vault for storing stake and votes'];
						type: 'publicKey';
					},
					{
						name: 'requiredVotes';
						docs: [
							'% of ve tokens needed to conclude',
							'represented as basis points 1% = 100bp'
						];
						type: 'u64';
					},
					{
						name: 'votes';
						docs: [
							'Current votes given in basis points',
							'1 vote = 1 veToken@',
							'Q64.0'
						];
						type: 'u64';
					},
					{
						name: 'revealedVotes';
						type: 'u64';
					},
					{
						name: 'runningSumWeightedVote';
						type: 'i64';
					},
					{
						name: 'runningWeight';
						type: 'u64';
					},
					{
						name: 'voteStartAt';
						docs: ['Start of vote'];
						type: 'i64';
					},
					{
						name: 'voteEndAt';
						docs: ['Blind vote deadline'];
						type: 'i64';
					},
					{
						name: 'voteEndRevealAt';
						docs: ['start reveal'];
						type: 'i64';
					},
					{
						name: 'earnedRewards';
						docs: ['reward earned by propsing vote', 'Q64.64'];
						type: 'u128';
					},
					{
						name: 'scaleParameter';
						docs: ['Scale parameter in exp(L)', 'Q16.16'];
						type: 'u32';
					},
					{
						name: 'scaleParameterCalculated';
						type: 'bool';
					},
					{
						name: 'locked';
						docs: ['when the vote is finished and', 'users can reap rewards'];
						type: 'bool';
					},
					{
						name: 'voteFactorSum';
						type: 'u64';
					},
					{
						name: 'distributionSum';
						type: 'u128';
					},
					{
						name: 'consensus';
						type: 'i64';
					}
				];
			};
		},
		{
			name: 'RevealedVoteArray';
			type: {
				kind: 'struct';
				fields: [
					{
						name: 'bump';
						type: 'u8';
					},
					{
						name: 'proposal';
						type: 'publicKey';
					},
					{
						name: 'weightedVotes';
						docs: ['Q32.32'];
						type: {
							array: ['i64', 1024];
						};
					},
					{
						name: 'lastIndex';
						type: 'i16';
					}
				];
			};
		},
		{
			name: 'VoteAccount';
			type: {
				kind: 'struct';
				fields: [
					{
						name: 'bump';
						type: 'u8';
					},
					{
						name: 'bumpArray';
						type: {
							array: ['u8', 1];
						};
					},
					{
						name: 'proposal';
						type: 'publicKey';
					},
					{
						name: 'owner';
						type: 'publicKey';
					},
					{
						name: 'voteHash';
						type: {
							array: ['u8', 32];
						};
					},
					{
						name: 'vote';
						docs: ['real vote:', 'I32.32'];
						type: 'i64';
					},
					{
						name: 'voteFactor';
						docs: ['F = V * l * exp(-l*(x-X))'];
						type: 'u64';
					},
					{
						name: 'earnedRewards';
						docs: ['rewards earned from voting', 'C * F / S_v'];
						type: 'u64';
					},
					{
						name: 'votePower';
						type: 'u32';
					},
					{
						name: 'revealedVote';
						type: 'bool';
					},
					{
						name: 'locked';
						type: 'bool';
					}
				];
			};
		}
	];
	types: [
		{
			name: 'ProposalStatus';
			type: {
				kind: 'enum';
				variants: [
					{
						name: 'Failed';
					},
					{
						name: 'Proposed';
					},
					{
						name: 'Voting';
					},
					{
						name: 'ReachedQuorum';
					},
					{
						name: 'RevealVote';
					},
					{
						name: 'VoteRevealFinished';
					},
					{
						name: 'RewardCalculation';
					},
					{
						name: 'RewardPayout';
					}
				];
			};
		}
	];
	events: [
		{
			name: 'CancelledVote';
			fields: [
				{
					name: 'vote';
					type: 'publicKey';
					index: false;
				},
				{
					name: 'proposal';
					type: 'publicKey';
					index: false;
				},
				{
					name: 'time';
					type: 'i64';
					index: false;
				},
				{
					name: 'refund';
					type: 'u64';
					index: false;
				}
			];
		},
		{
			name: 'ProposeVoteEvent';
			fields: [
				{
					name: 'name';
					type: 'string';
					index: false;
				},
				{
					name: 'proposer';
					type: 'publicKey';
					index: false;
				}
			];
		}
	];
	errors: [
		{
			code: 6000;
			name: 'StakeTooLittle';
			msg: 'Not enough staked on vote';
		},
		{
			code: 6001;
			name: 'InvalidLockPeriod';
			msg: 'Invalid lock period';
		},
		{
			code: 6002;
			name: 'InvalidVoteEndTime';
			msg: 'Invalid vote end time';
		},
		{
			code: 6003;
			name: 'VotingPeriodEnded';
			msg: 'Voting period for proposal has ended';
		},
		{
			code: 6004;
			name: 'RevealPeriodNotActive';
			msg: 'Currently not in vote reveal period';
		},
		{
			code: 6005;
			name: 'RevealPeriodIsNotFinished';
			msg: 'Reveal period is not over';
		},
		{
			code: 6006;
			name: 'InvalidSalt';
			msg: 'Invalid salt resulted in invalid vote_hash';
		},
		{
			code: 6007;
			name: 'FullRevealList';
			msg: 'Revealed vote list full';
		},
		{
			code: 6008;
			name: 'VoteNotRevealed';
			msg: "Vote hasn't been revealed";
		},
		{
			code: 6009;
			name: 'OverflowU64';
			msg: 'U64 overflow';
		},
		{
			code: 6010;
			name: 'OverflowU32';
			msg: 'U32 overflow';
		},
		{
			code: 6011;
			name: 'NotPossibleToCalculateVoteReward';
			msg: 'Could not calculate the vote reward at this time';
		},
		{
			code: 6012;
			name: 'NotPossibleToCollectProposerReward';
			msg: 'Cannot payout the proposer reward at this time';
		},
		{
			code: 6013;
			name: 'NotPossibleToCollectVoterReward';
			msg: 'Cannot payout the voter reward at this time';
		},
		{
			code: 6014;
			name: 'FailedToFinalizeVote';
			msg: 'Cannot finalize user vote at this time';
		},
		{
			code: 6015;
			name: 'FailedToFinalizeVoteResult';
			msg: 'Cannot finalize vote result at this time';
		},
		{
			code: 6016;
			name: 'FailedToCancelVote';
			msg: 'Too late to cancel vote';
		}
	];
};
export const OracleJSON: OracleIDL = {
	version: '0.1.0',
	name: 'oracle',
	instructions: [
		{
			name: 'proposeVote',
			docs: [
				'Propose vote',
				'',
				'proposes a vote or observation that the holder of veSure can',
				'vote on.',
				'',
				'### paramters',
				'* `ctx`: Context',
				'* `name`: Name of the observation',
				'* `description`: Clear description about the event',
				'* `stake`: The amount staked on event. In BN:  x*10^{decimals}',
			],
			accounts: [
				{
					name: 'proposer',
					isMut: true,
					isSigner: true,
				},
				{
					name: 'proposal',
					isMut: true,
					isSigner: false,
					pda: {
						seeds: [
							{
								kind: 'const',
								type: 'string',
								value: 'sure-oracle',
							},
							{
								kind: 'arg',
								type: 'string',
								path: 'name',
							},
						],
					},
				},
				{
					name: 'revealVoteArray',
					isMut: true,
					isSigner: false,
					pda: {
						seeds: [
							{
								kind: 'const',
								type: 'string',
								value: 'sure-oracle-reveal-array',
							},
							{
								kind: 'arg',
								type: 'string',
								path: 'name',
							},
						],
					},
				},
				{
					name: 'proposerAccount',
					isMut: true,
					isSigner: false,
				},
				{
					name: 'proposalVaultMint',
					isMut: false,
					isSigner: false,
				},
				{
					name: 'proposalVault',
					isMut: true,
					isSigner: false,
					pda: {
						seeds: [
							{
								kind: 'const',
								type: 'string',
								value: 'sure-oracle',
							},
							{
								kind: 'account',
								type: 'publicKey',
								account: 'Mint',
								path: 'proposal_vault_mint',
							},
						],
					},
				},
				{
					name: 'tokenProgram',
					isMut: false,
					isSigner: false,
				},
				{
					name: 'associatedTokenProgram',
					isMut: false,
					isSigner: false,
				},
				{
					name: 'rent',
					isMut: false,
					isSigner: false,
				},
				{
					name: 'systemProgram',
					isMut: false,
					isSigner: false,
				},
			],
			args: [
				{
					name: 'name',
					type: 'string',
				},
				{
					name: 'description',
					type: 'string',
				},
				{
					name: 'stake',
					type: 'u64',
				},
			],
		},
		{
			name: 'submitVote',
			docs: [
				'Submit vote',
				'',
				'lets user vote blindly on a proposal using a vote hash',
				'',
				'### Parameters',
				'* `ctx` - context',
				'* `vote_hash` - hash of vote with secret salt',
			],
			accounts: [
				{
					name: 'voter',
					isMut: true,
					isSigner: true,
				},
				{
					name: 'voterAccount',
					isMut: true,
					isSigner: false,
				},
				{
					name: 'locker',
					isMut: false,
					isSigner: false,
				},
				{
					name: 'userEscrow',
					isMut: false,
					isSigner: false,
				},
				{
					name: 'proposal',
					isMut: true,
					isSigner: false,
				},
				{
					name: 'proposalVault',
					isMut: true,
					isSigner: false,
				},
				{
					name: 'proposalVaultMint',
					isMut: false,
					isSigner: false,
				},
				{
					name: 'voteAccount',
					isMut: true,
					isSigner: false,
					pda: {
						seeds: [
							{
								kind: 'const',
								type: 'string',
								value: 'sure-oracle-vote',
							},
							{
								kind: 'account',
								type: 'publicKey',
								account: 'Proposal',
								path: 'proposal',
							},
							{
								kind: 'account',
								type: 'publicKey',
								path: 'voter',
							},
						],
					},
				},
				{
					name: 'tokenProgram',
					isMut: false,
					isSigner: false,
				},
				{
					name: 'rent',
					isMut: false,
					isSigner: false,
				},
				{
					name: 'systemProgram',
					isMut: false,
					isSigner: false,
				},
			],
			args: [
				{
					name: 'voteHash',
					type: 'bytes',
				},
			],
		},
		{
			name: 'updateVote',
			docs: [
				'Updates vote',
				'',
				'updates the vote hash of the previous submitted vote',
				'',
				'### parameters',
				'* `ctx` - context',
				'* `vote_hash` - hash of vote with secret salt',
			],
			accounts: [
				{
					name: 'voter',
					isMut: true,
					isSigner: true,
				},
				{
					name: 'proposal',
					isMut: false,
					isSigner: false,
				},
				{
					name: 'voteAccount',
					isMut: true,
					isSigner: false,
					pda: {
						seeds: [
							{
								kind: 'const',
								type: 'string',
								value: 'sure-oracle-vote',
							},
							{
								kind: 'account',
								type: 'publicKey',
								account: 'Proposal',
								path: 'proposal',
							},
							{
								kind: 'account',
								type: 'publicKey',
								path: 'voter',
							},
						],
					},
				},
				{
					name: 'systemProgram',
					isMut: false,
					isSigner: false,
				},
			],
			args: [
				{
					name: 'voteHash',
					type: 'string',
				},
			],
		},
		{
			name: 'cancelVote',
			docs: [
				'cancel vote',
				'',
				'a user can cancel the vote in the voting period',
				'',
				'### parameters',
				'* `ctx` - CancelVote context',
			],
			accounts: [
				{
					name: 'voter',
					isMut: true,
					isSigner: true,
				},
				{
					name: 'voterAccount',
					isMut: true,
					isSigner: false,
				},
				{
					name: 'proposalVault',
					isMut: true,
					isSigner: false,
				},
				{
					name: 'proposalVaultMint',
					isMut: false,
					isSigner: false,
				},
				{
					name: 'proposal',
					isMut: true,
					isSigner: false,
				},
				{
					name: 'voteAccount',
					isMut: true,
					isSigner: false,
				},
				{
					name: 'systemProgram',
					isMut: false,
					isSigner: false,
				},
				{
					name: 'tokenProgram',
					isMut: false,
					isSigner: false,
				},
			],
			args: [],
		},
		{
			name: 'revealVote',
			docs: [
				'reveal vote',
				'',
				'let the user reveal the vote when the voting period is over',
				'the user can only receive rewards if revealed',
				'',
				'### parameters',
				'* `ctx` - RevealVote context',
				'* `salt` - the salt used to hash the vote',
				'* `vote`- the actual vote value',
			],
			accounts: [
				{
					name: 'voter',
					isMut: true,
					isSigner: true,
				},
				{
					name: 'proposal',
					isMut: false,
					isSigner: false,
					pda: {
						seeds: [
							{
								kind: 'const',
								type: 'string',
								value: 'sure-oracle',
							},
							{
								kind: 'account',
								type: 'string',
								account: 'Proposal',
								path: 'proposal.name',
							},
						],
					},
				},
				{
					name: 'revealVoteArray',
					isMut: true,
					isSigner: false,
					pda: {
						seeds: [
							{
								kind: 'const',
								type: 'string',
								value: 'sure-oracle',
							},
							{
								kind: 'account',
								type: 'publicKey',
								account: 'Proposal',
								path: 'proposal',
							},
						],
					},
				},
				{
					name: 'voteAccount',
					isMut: true,
					isSigner: false,
					pda: {
						seeds: [
							{
								kind: 'const',
								type: 'string',
								value: 'sure-oracle-vote',
							},
							{
								kind: 'account',
								type: 'publicKey',
								account: 'Proposal',
								path: 'proposal',
							},
							{
								kind: 'account',
								type: 'publicKey',
								path: 'voter',
							},
						],
					},
				},
				{
					name: 'systemProgram',
					isMut: false,
					isSigner: false,
				},
			],
			args: [
				{
					name: 'salt',
					type: 'string',
				},
				{
					name: 'vote',
					type: 'i64',
				},
			],
		},
		{
			name: 'finalizeVoteResults',
			docs: [
				'finalize vote results',
				'',
				'after the reveal period the proposal can be finalized',
				'from this point on it is not possible to reveal the vote',
				'',
				'the proposer reward and scale parameter is calculated',
				'',
				'### parameters',
				'*  `ctx` - the Finalize Vote context',
			],
			accounts: [
				{
					name: 'finalizer',
					isMut: true,
					isSigner: true,
				},
				{
					name: 'proposal',
					isMut: true,
					isSigner: false,
				},
				{
					name: 'revealedVotes',
					isMut: false,
					isSigner: false,
				},
				{
					name: 'systemProgram',
					isMut: false,
					isSigner: false,
				},
			],
			args: [],
		},
		{
			name: 'finalizeVote',
			docs: [
				'finalize vote',
				'',
				'after the vote results are finalized the voters can calculate',
				'their vote share and close their vote account',
				'',
				'### parameters',
				'* `ctx` - Finalize Vote context',
			],
			accounts: [
				{
					name: 'signer',
					isMut: true,
					isSigner: true,
				},
				{
					name: 'voteAccount',
					isMut: true,
					isSigner: false,
				},
				{
					name: 'proposal',
					isMut: false,
					isSigner: false,
				},
				{
					name: 'systemProgram',
					isMut: false,
					isSigner: false,
				},
			],
			args: [],
		},
		{
			name: 'collectProposerReward',
			docs: [
				'collect proposer reward',
				'',
				'after the vote results are finalized the proposer is free to',
				'collect the reward at any time',
			],
			accounts: [
				{
					name: 'proposer',
					isMut: true,
					isSigner: true,
				},
				{
					name: 'proposerAccount',
					isMut: true,
					isSigner: false,
				},
				{
					name: 'proposal',
					isMut: true,
					isSigner: false,
				},
				{
					name: 'proposalVault',
					isMut: true,
					isSigner: false,
				},
				{
					name: 'proposalVaultMint',
					isMut: true,
					isSigner: false,
				},
				{
					name: 'systemProgram',
					isMut: false,
					isSigner: false,
				},
				{
					name: 'tokenProgram',
					isMut: false,
					isSigner: false,
				},
			],
			args: [],
		},
		{
			name: 'collectVoteReward',
			docs: [
				'collect vote reward',
				'',
				'after the vote results are finalized the voter can collect rewards',
			],
			accounts: [
				{
					name: 'voter',
					isMut: true,
					isSigner: true,
				},
				{
					name: 'voterAccount',
					isMut: true,
					isSigner: false,
				},
				{
					name: 'voteAccount',
					isMut: true,
					isSigner: false,
				},
				{
					name: 'proposal',
					isMut: true,
					isSigner: false,
				},
				{
					name: 'proposalVaultMint',
					isMut: false,
					isSigner: false,
				},
				{
					name: 'proposalVault',
					isMut: false,
					isSigner: false,
				},
				{
					name: 'systemProgram',
					isMut: false,
					isSigner: false,
				},
				{
					name: 'tokenProgram',
					isMut: false,
					isSigner: false,
				},
			],
			args: [],
		},
	],
	accounts: [
		{
			name: 'Proposal',
			type: {
				kind: 'struct',
				fields: [
					{
						name: 'bump',
						docs: ['bump for verification'],
						type: 'u8',
					},
					{
						name: 'bumpArray',
						type: {
							array: ['u8', 1],
						},
					},
					{
						name: 'name',
						docs: ['name of vote'],
						type: 'string',
					},
					{
						name: 'description',
						docs: ['description of vote'],
						type: 'string',
					},
					{
						name: 'proposedResult',
						docs: ['Proposed result'],
						type: 'i64',
					},
					{
						name: 'proposer',
						docs: ['user who proposed the vote'],
						type: 'publicKey',
					},
					{
						name: 'proposedStaked',
						docs: ['amount staked by propose Q32.32'],
						type: 'u64',
					},
					{
						name: 'vault',
						docs: ['vault for storing stake and votes'],
						type: 'publicKey',
					},
					{
						name: 'requiredVotes',
						docs: [
							'% of ve tokens needed to conclude',
							'represented as basis points 1% = 100bp',
						],
						type: 'u64',
					},
					{
						name: 'votes',
						docs: [
							'Current votes given in basis points',
							'1 vote = 1 veToken@',
							'Q64.0',
						],
						type: 'u64',
					},
					{
						name: 'revealedVotes',
						type: 'u64',
					},
					{
						name: 'runningSumWeightedVote',
						type: 'i64',
					},
					{
						name: 'runningWeight',
						type: 'u64',
					},
					{
						name: 'voteStartAt',
						docs: ['Start of vote'],
						type: 'i64',
					},
					{
						name: 'voteEndAt',
						docs: ['Blind vote deadline'],
						type: 'i64',
					},
					{
						name: 'voteEndRevealAt',
						docs: ['start reveal'],
						type: 'i64',
					},
					{
						name: 'earnedRewards',
						docs: ['reward earned by propsing vote', 'Q64.64'],
						type: 'u128',
					},
					{
						name: 'scaleParameter',
						docs: ['Scale parameter in exp(L)', 'Q16.16'],
						type: 'u32',
					},
					{
						name: 'scaleParameterCalculated',
						type: 'bool',
					},
					{
						name: 'locked',
						docs: ['when the vote is finished and', 'users can reap rewards'],
						type: 'bool',
					},
					{
						name: 'voteFactorSum',
						type: 'u64',
					},
					{
						name: 'distributionSum',
						type: 'u128',
					},
					{
						name: 'consensus',
						type: 'i64',
					},
				],
			},
		},
		{
			name: 'RevealedVoteArray',
			type: {
				kind: 'struct',
				fields: [
					{
						name: 'bump',
						type: 'u8',
					},
					{
						name: 'proposal',
						type: 'publicKey',
					},
					{
						name: 'weightedVotes',
						docs: ['Q32.32'],
						type: {
							array: ['i64', 1024],
						},
					},
					{
						name: 'lastIndex',
						type: 'i16',
					},
				],
			},
		},
		{
			name: 'VoteAccount',
			type: {
				kind: 'struct',
				fields: [
					{
						name: 'bump',
						type: 'u8',
					},
					{
						name: 'bumpArray',
						type: {
							array: ['u8', 1],
						},
					},
					{
						name: 'proposal',
						type: 'publicKey',
					},
					{
						name: 'owner',
						type: 'publicKey',
					},
					{
						name: 'voteHash',
						type: {
							array: ['u8', 32],
						},
					},
					{
						name: 'vote',
						docs: ['real vote:', 'I32.32'],
						type: 'i64',
					},
					{
						name: 'voteFactor',
						docs: ['F = V * l * exp(-l*(x-X))'],
						type: 'u64',
					},
					{
						name: 'earnedRewards',
						docs: ['rewards earned from voting', 'C * F / S_v'],
						type: 'u64',
					},
					{
						name: 'votePower',
						type: 'u32',
					},
					{
						name: 'revealedVote',
						type: 'bool',
					},
					{
						name: 'locked',
						type: 'bool',
					},
				],
			},
		},
	],
	types: [
		{
			name: 'ProposalStatus',
			type: {
				kind: 'enum',
				variants: [
					{
						name: 'Failed',
					},
					{
						name: 'Proposed',
					},
					{
						name: 'Voting',
					},
					{
						name: 'ReachedQuorum',
					},
					{
						name: 'RevealVote',
					},
					{
						name: 'VoteRevealFinished',
					},
					{
						name: 'RewardCalculation',
					},
					{
						name: 'RewardPayout',
					},
				],
			},
		},
	],
	events: [
		{
			name: 'CancelledVote',
			fields: [
				{
					name: 'vote',
					type: 'publicKey',
					index: false,
				},
				{
					name: 'proposal',
					type: 'publicKey',
					index: false,
				},
				{
					name: 'time',
					type: 'i64',
					index: false,
				},
				{
					name: 'refund',
					type: 'u64',
					index: false,
				},
			],
		},
		{
			name: 'ProposeVoteEvent',
			fields: [
				{
					name: 'name',
					type: 'string',
					index: false,
				},
				{
					name: 'proposer',
					type: 'publicKey',
					index: false,
				},
			],
		},
	],
	errors: [
		{
			code: 6000,
			name: 'StakeTooLittle',
			msg: 'Not enough staked on vote',
		},
		{
			code: 6001,
			name: 'InvalidLockPeriod',
			msg: 'Invalid lock period',
		},
		{
			code: 6002,
			name: 'InvalidVoteEndTime',
			msg: 'Invalid vote end time',
		},
		{
			code: 6003,
			name: 'VotingPeriodEnded',
			msg: 'Voting period for proposal has ended',
		},
		{
			code: 6004,
			name: 'RevealPeriodNotActive',
			msg: 'Currently not in vote reveal period',
		},
		{
			code: 6005,
			name: 'RevealPeriodIsNotFinished',
			msg: 'Reveal period is not over',
		},
		{
			code: 6006,
			name: 'InvalidSalt',
			msg: 'Invalid salt resulted in invalid vote_hash',
		},
		{
			code: 6007,
			name: 'FullRevealList',
			msg: 'Revealed vote list full',
		},
		{
			code: 6008,
			name: 'VoteNotRevealed',
			msg: "Vote hasn't been revealed",
		},
		{
			code: 6009,
			name: 'OverflowU64',
			msg: 'U64 overflow',
		},
		{
			code: 6010,
			name: 'OverflowU32',
			msg: 'U32 overflow',
		},
		{
			code: 6011,
			name: 'NotPossibleToCalculateVoteReward',
			msg: 'Could not calculate the vote reward at this time',
		},
		{
			code: 6012,
			name: 'NotPossibleToCollectProposerReward',
			msg: 'Cannot payout the proposer reward at this time',
		},
		{
			code: 6013,
			name: 'NotPossibleToCollectVoterReward',
			msg: 'Cannot payout the voter reward at this time',
		},
		{
			code: 6014,
			name: 'FailedToFinalizeVote',
			msg: 'Cannot finalize user vote at this time',
		},
		{
			code: 6015,
			name: 'FailedToFinalizeVoteResult',
			msg: 'Cannot finalize vote result at this time',
		},
		{
			code: 6016,
			name: 'FailedToCancelVote',
			msg: 'Too late to cancel vote',
		},
	],
};
