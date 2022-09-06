export const IDL = {
    "version": "0.1.0",
    "name": "oracle",
    "instructions": [
        {
            "name": "initializeConfig",
            "docs": [
                "initialize config",
                "",
                "config will be used in proposals to set voting parameters",
                "and limit protocol fee collectors to the protocol_authority",
                "",
                "### args",
                "* protocol_authority<Pubkey>: unique for vault mint. the authority can",
                "- change config parameters",
                "- collect protocol fees"
            ],
            "accounts": [
                {
                    "name": "signer",
                    "isMut": true,
                    "isSigner": true
                },
                {
                    "name": "config",
                    "isMut": true,
                    "isSigner": false,
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "type": "string",
                                "value": "sure-oracle-config"
                            },
                            {
                                "kind": "account",
                                "type": "publicKey",
                                "account": "Mint",
                                "path": "token_mint"
                            }
                        ]
                    }
                },
                {
                    "name": "tokenMint",
                    "isMut": false,
                    "isSigner": false
                },
                {
                    "name": "systemProgram",
                    "isMut": false,
                    "isSigner": false
                }
            ],
            "args": [
                {
                    "name": "protocolAuthority",
                    "type": "publicKey"
                }
            ]
        },
        {
            "name": "updateVotingPeriod",
            "docs": [
                "update config: voting period",
                "",
                "change the voting period and reveal period",
                "",
                "### args",
                "* voting_period<i64>: period for which the voter can submit a vote hash. In seconds",
                "* reveal_period<i64>: period for which the voter can reveal the vote. In seconds"
            ],
            "accounts": [
                {
                    "name": "protocolAuthority",
                    "isMut": false,
                    "isSigner": true
                },
                {
                    "name": "config",
                    "isMut": false,
                    "isSigner": false
                },
                {
                    "name": "systemProgram",
                    "isMut": false,
                    "isSigner": false
                }
            ],
            "args": [
                {
                    "name": "votingPeriod",
                    "type": "i64"
                }
            ]
        },
        {
            "name": "updateRevealPeriod",
            "docs": [
                "update config: reveal period",
                "",
                "change the reveal period and reveal period",
                "",
                "### args",
                "* voting_period<i64>: period for which the voter can submit a vote hash. In seconds",
                "* reveal_period<i64>: period for which the voter can reveal the vote. In seconds"
            ],
            "accounts": [
                {
                    "name": "protocolAuthority",
                    "isMut": false,
                    "isSigner": true
                },
                {
                    "name": "config",
                    "isMut": false,
                    "isSigner": false
                },
                {
                    "name": "systemProgram",
                    "isMut": false,
                    "isSigner": false
                }
            ],
            "args": [
                {
                    "name": "votingPeriod",
                    "type": "i64"
                }
            ]
        },
        {
            "name": "updateRequiredVotes",
            "docs": [
                "update required votes",
                "",
                "required votes to reach quorum",
                "",
                "### args",
                "* require_votes<u64>: number of votes needed to conclude vote"
            ],
            "accounts": [
                {
                    "name": "protocolAuthority",
                    "isMut": false,
                    "isSigner": true
                },
                {
                    "name": "config",
                    "isMut": false,
                    "isSigner": false
                },
                {
                    "name": "systemProgram",
                    "isMut": false,
                    "isSigner": false
                }
            ],
            "args": [
                {
                    "name": "requiredVotes",
                    "type": "u64"
                }
            ]
        },
        {
            "name": "updateProposalMinimumStake",
            "docs": [
                "update proposal minimum stake",
                "",
                "the minimum amount that needs to be staked in order to create a",
                "proposal",
                "",
                "### args",
                "* minimum_stake<u64>: stake needed to propose vote"
            ],
            "accounts": [
                {
                    "name": "protocolAuthority",
                    "isMut": false,
                    "isSigner": true
                },
                {
                    "name": "config",
                    "isMut": false,
                    "isSigner": false
                },
                {
                    "name": "systemProgram",
                    "isMut": false,
                    "isSigner": false
                }
            ],
            "args": [
                {
                    "name": "minimumStake",
                    "type": "u64"
                }
            ]
        },
        {
            "name": "updateVoteStakeRate",
            "docs": [
                "update vote stake rate",
                "",
                "the stake rate sets requirements to how much a",
                "voter needs to stake in order to vote. typically 1% of voting",
                "power",
                "",
                "### args",
                "* vote_stake_rate<u32>: 1/x of voting power"
            ],
            "accounts": [
                {
                    "name": "protocolAuthority",
                    "isMut": false,
                    "isSigner": true
                },
                {
                    "name": "config",
                    "isMut": false,
                    "isSigner": false
                },
                {
                    "name": "systemProgram",
                    "isMut": false,
                    "isSigner": false
                }
            ],
            "args": [
                {
                    "name": "voteStakeRate",
                    "type": "u32"
                }
            ]
        },
        {
            "name": "updateProtocolFeeRate",
            "docs": [
                "update protocol fee rate",
                "",
                "the amount the protocol can take in fees",
                "",
                "### args",
                "* protocol_fee_rate<u32>: 1/x of the voting pool"
            ],
            "accounts": [
                {
                    "name": "protocolAuthority",
                    "isMut": false,
                    "isSigner": true
                },
                {
                    "name": "config",
                    "isMut": false,
                    "isSigner": false
                },
                {
                    "name": "systemProgram",
                    "isMut": false,
                    "isSigner": false
                }
            ],
            "args": [
                {
                    "name": "protocolFeeRate",
                    "type": "u32"
                }
            ]
        },
        {
            "name": "proposeVote",
            "docs": [
                "Propose vote",
                "",
                "proposes a vote or observation that the holder of veSure can",
                "vote on.",
                "",
                "### paramters",
                "* `ctx`: Context",
                "* `name`: Name of the observation",
                "* `description`: Clear description about the event",
                "* `stake`: The amount staked on event. In BN:  x*10^{decimals}"
            ],
            "accounts": [
                {
                    "name": "proposer",
                    "isMut": true,
                    "isSigner": true
                },
                {
                    "name": "config",
                    "isMut": false,
                    "isSigner": false,
                    "docs": [
                        "configuration for the proposal"
                    ]
                },
                {
                    "name": "proposal",
                    "isMut": true,
                    "isSigner": false,
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "type": "string",
                                "value": "sure-oracle"
                            },
                            {
                                "kind": "arg",
                                "type": "bytes",
                                "path": "id"
                            }
                        ]
                    }
                },
                {
                    "name": "revealVoteArray",
                    "isMut": true,
                    "isSigner": false,
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "type": "string",
                                "value": "sure-oracle-reveal-array"
                            },
                            {
                                "kind": "account",
                                "type": "publicKey",
                                "account": "Proposal",
                                "path": "proposal"
                            }
                        ]
                    }
                },
                {
                    "name": "proposerAccount",
                    "isMut": true,
                    "isSigner": false
                },
                {
                    "name": "proposalVaultMint",
                    "isMut": false,
                    "isSigner": false
                },
                {
                    "name": "proposalVault",
                    "isMut": true,
                    "isSigner": false,
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "type": "string",
                                "value": "sure-oracle"
                            },
                            {
                                "kind": "account",
                                "type": "publicKey",
                                "account": "Proposal",
                                "path": "proposal"
                            }
                        ]
                    }
                },
                {
                    "name": "tokenProgram",
                    "isMut": false,
                    "isSigner": false
                },
                {
                    "name": "associatedTokenProgram",
                    "isMut": false,
                    "isSigner": false
                },
                {
                    "name": "rent",
                    "isMut": false,
                    "isSigner": false
                },
                {
                    "name": "systemProgram",
                    "isMut": false,
                    "isSigner": false
                }
            ],
            "args": [
                {
                    "name": "id",
                    "type": "bytes"
                },
                {
                    "name": "name",
                    "type": "string"
                },
                {
                    "name": "description",
                    "type": "string"
                },
                {
                    "name": "stake",
                    "type": "u64"
                }
            ]
        },
        {
            "name": "submitVote",
            "docs": [
                "Submit vote",
                "",
                "lets user vote blindly on a proposal using a vote hash",
                "",
                "### Parameters",
                "* `ctx` - context",
                "* `vote_hash` - hash of vote with secret salt"
            ],
            "accounts": [
                {
                    "name": "voter",
                    "isMut": true,
                    "isSigner": true
                },
                {
                    "name": "voterAccount",
                    "isMut": true,
                    "isSigner": false
                },
                {
                    "name": "locker",
                    "isMut": false,
                    "isSigner": false
                },
                {
                    "name": "userEscrow",
                    "isMut": false,
                    "isSigner": false
                },
                {
                    "name": "proposal",
                    "isMut": true,
                    "isSigner": false
                },
                {
                    "name": "proposalVault",
                    "isMut": true,
                    "isSigner": false
                },
                {
                    "name": "proposalVaultMint",
                    "isMut": false,
                    "isSigner": false
                },
                {
                    "name": "voteAccount",
                    "isMut": true,
                    "isSigner": false,
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "type": "string",
                                "value": "sure-oracle-vote"
                            },
                            {
                                "kind": "account",
                                "type": "publicKey",
                                "account": "Proposal",
                                "path": "proposal"
                            },
                            {
                                "kind": "account",
                                "type": "publicKey",
                                "path": "voter"
                            }
                        ]
                    }
                },
                {
                    "name": "tokenProgram",
                    "isMut": false,
                    "isSigner": false
                },
                {
                    "name": "rent",
                    "isMut": false,
                    "isSigner": false
                },
                {
                    "name": "systemProgram",
                    "isMut": false,
                    "isSigner": false
                }
            ],
            "args": [
                {
                    "name": "voteHash",
                    "type": "bytes"
                }
            ]
        },
        {
            "name": "updateVote",
            "docs": [
                "Updates vote",
                "",
                "updates the vote hash of the previous submitted vote",
                "",
                "### parameters",
                "* `ctx` - context",
                "* `vote_hash` - hash of vote with secret salt"
            ],
            "accounts": [
                {
                    "name": "voter",
                    "isMut": false,
                    "isSigner": true
                },
                {
                    "name": "proposal",
                    "isMut": true,
                    "isSigner": false
                },
                {
                    "name": "voteAccount",
                    "isMut": true,
                    "isSigner": false,
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "type": "string",
                                "value": "sure-oracle-vote"
                            },
                            {
                                "kind": "account",
                                "type": "publicKey",
                                "account": "Proposal",
                                "path": "proposal"
                            },
                            {
                                "kind": "account",
                                "type": "publicKey",
                                "path": "voter"
                            }
                        ]
                    }
                },
                {
                    "name": "systemProgram",
                    "isMut": false,
                    "isSigner": false
                }
            ],
            "args": [
                {
                    "name": "voteHash",
                    "type": "bytes"
                }
            ]
        },
        {
            "name": "cancelVote",
            "docs": [
                "cancel vote",
                "",
                "a user can cancel the vote in the voting period",
                "",
                "### parameters",
                "* `ctx` - CancelVote context"
            ],
            "accounts": [
                {
                    "name": "voter",
                    "isMut": true,
                    "isSigner": true
                },
                {
                    "name": "voterAccount",
                    "isMut": true,
                    "isSigner": false
                },
                {
                    "name": "proposalVault",
                    "isMut": true,
                    "isSigner": false
                },
                {
                    "name": "proposalVaultMint",
                    "isMut": false,
                    "isSigner": false
                },
                {
                    "name": "proposal",
                    "isMut": true,
                    "isSigner": false
                },
                {
                    "name": "voteAccount",
                    "isMut": true,
                    "isSigner": false
                },
                {
                    "name": "systemProgram",
                    "isMut": false,
                    "isSigner": false
                },
                {
                    "name": "tokenProgram",
                    "isMut": false,
                    "isSigner": false
                }
            ],
            "args": []
        },
        {
            "name": "revealVote",
            "docs": [
                "reveal vote",
                "",
                "let the user reveal the vote when the voting period is over",
                "the user can only receive rewards if revealed",
                "",
                "### parameters",
                "* `ctx` - RevealVote context",
                "* `salt` - the salt used to hash the vote",
                "* `vote`- the actual vote value"
            ],
            "accounts": [
                {
                    "name": "voter",
                    "isMut": true,
                    "isSigner": true
                },
                {
                    "name": "proposal",
                    "isMut": true,
                    "isSigner": false,
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "type": "string",
                                "value": "sure-oracle"
                            },
                            {
                                "kind": "account",
                                "type": {
                                    "array": [
                                        "u8",
                                        16
                                    ]
                                },
                                "account": "Proposal",
                                "path": "proposal.id"
                            }
                        ]
                    }
                },
                {
                    "name": "revealVoteArray",
                    "isMut": true,
                    "isSigner": false,
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "type": "string",
                                "value": "sure-oracle-reveal-array"
                            },
                            {
                                "kind": "account",
                                "type": "publicKey",
                                "account": "Proposal",
                                "path": "proposal"
                            }
                        ]
                    }
                },
                {
                    "name": "voteAccount",
                    "isMut": true,
                    "isSigner": false,
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "type": "string",
                                "value": "sure-oracle-vote"
                            },
                            {
                                "kind": "account",
                                "type": "publicKey",
                                "account": "Proposal",
                                "path": "proposal"
                            },
                            {
                                "kind": "account",
                                "type": "publicKey",
                                "path": "voter"
                            }
                        ]
                    }
                },
                {
                    "name": "systemProgram",
                    "isMut": false,
                    "isSigner": false
                }
            ],
            "args": [
                {
                    "name": "salt",
                    "type": "string"
                },
                {
                    "name": "vote",
                    "type": "i64"
                }
            ]
        },
        {
            "name": "finalizeVoteResults",
            "docs": [
                "finalize vote results",
                "",
                "after the reveal period the proposal can be finalized",
                "from this point on it is not possible to reveal the vote",
                "",
                "the proposer reward and scale parameter is calculated",
                "",
                "### parameters",
                "*  `ctx` - the Finalize Vote context"
            ],
            "accounts": [
                {
                    "name": "finalizer",
                    "isMut": true,
                    "isSigner": true
                },
                {
                    "name": "proposal",
                    "isMut": true,
                    "isSigner": false
                },
                {
                    "name": "revealedVotes",
                    "isMut": false,
                    "isSigner": false
                },
                {
                    "name": "systemProgram",
                    "isMut": false,
                    "isSigner": false
                }
            ],
            "args": []
        },
        {
            "name": "finalizeVote",
            "docs": [
                "finalize vote",
                "",
                "after the vote results are finalized the voters can calculate",
                "their vote share and close their vote account",
                "",
                "### parameters",
                "* `ctx` - Finalize Vote context"
            ],
            "accounts": [
                {
                    "name": "signer",
                    "isMut": true,
                    "isSigner": true
                },
                {
                    "name": "voteAccount",
                    "isMut": true,
                    "isSigner": false
                },
                {
                    "name": "proposal",
                    "isMut": false,
                    "isSigner": false,
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "type": "string",
                                "value": "sure-oracle"
                            },
                            {
                                "kind": "account",
                                "type": {
                                    "array": [
                                        "u8",
                                        16
                                    ]
                                },
                                "account": "Proposal",
                                "path": "proposal.id"
                            }
                        ]
                    }
                },
                {
                    "name": "systemProgram",
                    "isMut": false,
                    "isSigner": false
                }
            ],
            "args": []
        },
        {
            "name": "collectProposerReward",
            "docs": [
                "collect proposer reward",
                "",
                "after the vote results are finalized the proposer is free to",
                "collect the reward at any time"
            ],
            "accounts": [
                {
                    "name": "proposer",
                    "isMut": true,
                    "isSigner": true
                },
                {
                    "name": "config",
                    "isMut": false,
                    "isSigner": false
                },
                {
                    "name": "proposerTokenAccount",
                    "isMut": true,
                    "isSigner": false
                },
                {
                    "name": "proposal",
                    "isMut": true,
                    "isSigner": false
                },
                {
                    "name": "proposalVault",
                    "isMut": true,
                    "isSigner": false
                },
                {
                    "name": "proposalVaultMint",
                    "isMut": true,
                    "isSigner": false
                },
                {
                    "name": "systemProgram",
                    "isMut": false,
                    "isSigner": false
                },
                {
                    "name": "tokenProgram",
                    "isMut": false,
                    "isSigner": false
                }
            ],
            "args": []
        },
        {
            "name": "collectVoteReward",
            "docs": [
                "collect vote reward",
                "",
                "after the vote results are finalized the voter can collect rewards"
            ],
            "accounts": [
                {
                    "name": "voter",
                    "isMut": true,
                    "isSigner": true
                },
                {
                    "name": "config",
                    "isMut": false,
                    "isSigner": false
                },
                {
                    "name": "voterAccount",
                    "isMut": true,
                    "isSigner": false
                },
                {
                    "name": "voteAccount",
                    "isMut": true,
                    "isSigner": false
                },
                {
                    "name": "proposal",
                    "isMut": true,
                    "isSigner": false
                },
                {
                    "name": "proposalVaultMint",
                    "isMut": false,
                    "isSigner": false
                },
                {
                    "name": "proposalVault",
                    "isMut": true,
                    "isSigner": false
                },
                {
                    "name": "systemProgram",
                    "isMut": false,
                    "isSigner": false
                },
                {
                    "name": "tokenProgram",
                    "isMut": false,
                    "isSigner": false
                }
            ],
            "args": []
        },
        {
            "name": "collectProtocolFees",
            "docs": [
                "collect protocol fees",
                "",
                "the config authority can at any time collect the protocol fees"
            ],
            "accounts": [
                {
                    "name": "protocolAuthority",
                    "isMut": false,
                    "isSigner": true
                },
                {
                    "name": "config",
                    "isMut": false,
                    "isSigner": false,
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "type": "string",
                                "value": "sure-oracle-config"
                            },
                            {
                                "kind": "account",
                                "type": "publicKey",
                                "account": "Config",
                                "path": "config.token_mint"
                            }
                        ]
                    }
                },
                {
                    "name": "proposal",
                    "isMut": true,
                    "isSigner": false
                },
                {
                    "name": "proposalVault",
                    "isMut": true,
                    "isSigner": false
                },
                {
                    "name": "feeDestination",
                    "isMut": true,
                    "isSigner": false
                },
                {
                    "name": "tokenProgram",
                    "isMut": false,
                    "isSigner": false
                }
            ],
            "args": []
        }
    ],
    "accounts": [
        {
            "name": "config",
            "type": {
                "kind": "struct",
                "fields": [
                    {
                        "name": "bump",
                        "type": "u8"
                    },
                    {
                        "name": "votingLengthSeconds",
                        "docs": [
                            "voting period in seconds"
                        ],
                        "type": "i64"
                    },
                    {
                        "name": "revealLengthSeconds",
                        "docs": [
                            "the lenght of the reveal period",
                            "in seconds"
                        ],
                        "type": "i64"
                    },
                    {
                        "name": "defaultRequiredVotes",
                        "docs": [
                            "the default required votes to reach",
                            "quorum."
                        ],
                        "type": "u64"
                    },
                    {
                        "name": "minimumProposalStake",
                        "docs": [
                            "the minimum amount of tokens that must",
                            "be staked on a proposal"
                        ],
                        "type": "u64"
                    },
                    {
                        "name": "voteStakeRate",
                        "docs": [
                            "the 1/x of the voting power that needs",
                            "to be staked in order to vote"
                        ],
                        "type": "u32"
                    },
                    {
                        "name": "protocolFeeRate",
                        "docs": [
                            "the 1/x of the total voting escrow",
                            "that's going to the protocol"
                        ],
                        "type": "u32"
                    },
                    {
                        "name": "tokenMint",
                        "docs": [
                            "official mint of pool"
                        ],
                        "type": "publicKey"
                    },
                    {
                        "name": "protocolAuthority",
                        "docs": [
                            "who can collect the rewards"
                        ],
                        "type": "publicKey"
                    },
                    {
                        "name": "initialized",
                        "type": "bool"
                    }
                ]
            }
        },
        {
            "name": "proposal",
            "type": {
                "kind": "struct",
                "fields": [
                    {
                        "name": "config",
                        "type": "publicKey"
                    },
                    {
                        "name": "bump",
                        "docs": [
                            "bump for verification"
                        ],
                        "type": "u8"
                    },
                    {
                        "name": "bumpArray",
                        "type": {
                            "array": [
                                "u8",
                                1
                            ]
                        }
                    },
                    {
                        "name": "locked",
                        "docs": [
                            "when the vote is finished and",
                            "users can reap rewards"
                        ],
                        "type": "bool"
                    },
                    {
                        "name": "optimistic",
                        "docs": [
                            "Optimistic"
                        ],
                        "type": "bool"
                    },
                    {
                        "name": "status",
                        "type": "u8"
                    },
                    {
                        "name": "name",
                        "docs": [
                            "name of vote"
                        ],
                        "type": "string"
                    },
                    {
                        "name": "id",
                        "docs": [
                            "id - hashed name"
                        ],
                        "type": {
                            "array": [
                                "u8",
                                16
                            ]
                        }
                    },
                    {
                        "name": "description",
                        "docs": [
                            "description of vote"
                        ],
                        "type": "string"
                    },
                    {
                        "name": "proposedResult",
                        "docs": [
                            "Proposed result"
                        ],
                        "type": "i64"
                    },
                    {
                        "name": "proposer",
                        "docs": [
                            "user who proposed the vote"
                        ],
                        "type": "publicKey"
                    },
                    {
                        "name": "stakeRate",
                        "docs": [
                            "1/x of vote power that must be staked"
                        ],
                        "type": "u32"
                    },
                    {
                        "name": "staked",
                        "docs": [
                            "amount staked by propose Q32.32"
                        ],
                        "type": "u64"
                    },
                    {
                        "name": "vault",
                        "docs": [
                            "vault for storing stake and votes"
                        ],
                        "type": "publicKey"
                    },
                    {
                        "name": "requiredVotes",
                        "docs": [
                            "% of ve tokens needed to conclude",
                            "represented as basis points 1% = 100bp"
                        ],
                        "type": "u64"
                    },
                    {
                        "name": "protocolFeeRate",
                        "docs": [
                            "as 1/x of revealed vote staked"
                        ],
                        "type": "u32"
                    },
                    {
                        "name": "votes",
                        "docs": [
                            "Current votes given in basis points",
                            "1 vote = 1 veToken@",
                            "Q64.0"
                        ],
                        "type": "u64"
                    },
                    {
                        "name": "revealedVotes",
                        "type": "u64"
                    },
                    {
                        "name": "runningSumWeightedVote",
                        "type": "i64"
                    },
                    {
                        "name": "runningWeight",
                        "type": "u64"
                    },
                    {
                        "name": "voteStartAt",
                        "docs": [
                            "Start of vote"
                        ],
                        "type": "i64"
                    },
                    {
                        "name": "voteEndAt",
                        "docs": [
                            "Blind vote deadline"
                        ],
                        "type": "i64"
                    },
                    {
                        "name": "voteEndRevealAt",
                        "docs": [
                            "start reveal"
                        ],
                        "type": "i64"
                    },
                    {
                        "name": "earnedRewards",
                        "docs": [
                            "reward earned by propsing vote",
                            "Q64.64"
                        ],
                        "type": "u128"
                    },
                    {
                        "name": "protocolFees",
                        "docs": [
                            "protocol fees"
                        ],
                        "type": "u128"
                    },
                    {
                        "name": "scaleParameter",
                        "docs": [
                            "Scale parameter in exp(L)",
                            "Q16.16"
                        ],
                        "type": "u32"
                    },
                    {
                        "name": "scaleParameterCalculated",
                        "type": "bool"
                    },
                    {
                        "name": "voteFactorSum",
                        "type": "u64"
                    },
                    {
                        "name": "distributionSum",
                        "type": "u128"
                    },
                    {
                        "name": "consensus",
                        "type": "i64"
                    }
                ]
            }
        },
        {
            "name": "revealedVoteArray",
            "type": {
                "kind": "struct",
                "fields": [
                    {
                        "name": "bump",
                        "type": "u8"
                    },
                    {
                        "name": "proposal",
                        "type": "publicKey"
                    },
                    {
                        "name": "weightedVotes",
                        "docs": [
                            "Q32.32"
                        ],
                        "type": {
                            "array": [
                                "i64",
                                1024
                            ]
                        }
                    },
                    {
                        "name": "lastIndex",
                        "type": "i16"
                    }
                ]
            }
        },
        {
            "name": "voteAccount",
            "type": {
                "kind": "struct",
                "fields": [
                    {
                        "name": "bump",
                        "type": "u8"
                    },
                    {
                        "name": "bumpArray",
                        "type": {
                            "array": [
                                "u8",
                                1
                            ]
                        }
                    },
                    {
                        "name": "proposal",
                        "type": "publicKey"
                    },
                    {
                        "name": "owner",
                        "type": "publicKey"
                    },
                    {
                        "name": "stakeMint",
                        "type": "publicKey"
                    },
                    {
                        "name": "voteHash",
                        "type": {
                            "array": [
                                "u8",
                                32
                            ]
                        }
                    },
                    {
                        "name": "vote",
                        "docs": [
                            "real vote:",
                            "I32.32"
                        ],
                        "type": "i64"
                    },
                    {
                        "name": "staked",
                        "type": "u64"
                    },
                    {
                        "name": "voteFactor",
                        "docs": [
                            "F = V * l * exp(-l*(x-X))"
                        ],
                        "type": "u64"
                    },
                    {
                        "name": "earnedRewards",
                        "docs": [
                            "rewards earned from voting",
                            "C * F / S_v"
                        ],
                        "type": "u64"
                    },
                    {
                        "name": "votePower",
                        "type": "u32"
                    },
                    {
                        "name": "revealedVote",
                        "type": "bool"
                    },
                    {
                        "name": "locked",
                        "type": "bool"
                    }
                ]
            }
        }
    ],
    "types": [
        {
            "name": "ProposalStatus",
            "type": {
                "kind": "enum",
                "variants": [
                    {
                        "name": "Failed"
                    },
                    {
                        "name": "Proposed"
                    },
                    {
                        "name": "Voting"
                    },
                    {
                        "name": "ReachedQuorum"
                    },
                    {
                        "name": "RevealVote"
                    },
                    {
                        "name": "VoteRevealFinished"
                    },
                    {
                        "name": "RewardCalculation"
                    },
                    {
                        "name": "RewardPayout"
                    }
                ]
            }
        }
    ],
    "events": [
        {
            "name": "CancelledVote",
            "fields": [
                {
                    "name": "vote",
                    "type": "publicKey",
                    "index": false
                },
                {
                    "name": "proposal",
                    "type": "publicKey",
                    "index": false
                },
                {
                    "name": "time",
                    "type": "i64",
                    "index": false
                },
                {
                    "name": "refund",
                    "type": "u64",
                    "index": false
                }
            ]
        },
        {
            "name": "CollectProposerRewardEvent",
            "fields": [
                {
                    "name": "proposal",
                    "type": "publicKey",
                    "index": false
                },
                {
                    "name": "time",
                    "type": "i64",
                    "index": false
                },
                {
                    "name": "reward",
                    "type": "u64",
                    "index": false
                }
            ]
        },
        {
            "name": "CollectedProtocolFees",
            "fields": [
                {
                    "name": "proposal",
                    "type": "publicKey",
                    "index": false
                },
                {
                    "name": "fees",
                    "type": "u64",
                    "index": false
                },
                {
                    "name": "destination",
                    "type": "publicKey",
                    "index": false
                }
            ]
        },
        {
            "name": "CollectVoteRewardEvent",
            "fields": [
                {
                    "name": "vote",
                    "type": "publicKey",
                    "index": false
                },
                {
                    "name": "proposal",
                    "type": "publicKey",
                    "index": false
                },
                {
                    "name": "time",
                    "type": "i64",
                    "index": false
                },
                {
                    "name": "reward",
                    "type": "u64",
                    "index": false
                }
            ]
        },
        {
            "name": "InitializedConfigEvent",
            "fields": []
        },
        {
            "name": "UpdatedVotingPeriod",
            "fields": [
                {
                    "name": "oldVotingPeriod",
                    "type": "i64",
                    "index": false
                },
                {
                    "name": "votingPeriod",
                    "type": "i64",
                    "index": false
                }
            ]
        },
        {
            "name": "UpdatedRevealPeriod",
            "fields": [
                {
                    "name": "oldRevealPeriod",
                    "type": "i64",
                    "index": false
                },
                {
                    "name": "revealPeriod",
                    "type": "i64",
                    "index": false
                }
            ]
        },
        {
            "name": "UpdatedRequiredVotes",
            "fields": [
                {
                    "name": "oldRequiredVotes",
                    "type": "u64",
                    "index": false
                },
                {
                    "name": "requiredVotes",
                    "type": "u64",
                    "index": false
                }
            ]
        },
        {
            "name": "UpdatedProposalMinumumStake",
            "fields": [
                {
                    "name": "oldMinimumStake",
                    "type": "u64",
                    "index": false
                },
                {
                    "name": "minimumStake",
                    "type": "u64",
                    "index": false
                }
            ]
        },
        {
            "name": "UpdatedVoteStakeRate",
            "fields": [
                {
                    "name": "oldVoteStakeRate",
                    "type": "u32",
                    "index": false
                },
                {
                    "name": "voteStakeRate",
                    "type": "u32",
                    "index": false
                }
            ]
        },
        {
            "name": "UpdatedProtocolFeerate",
            "fields": [
                {
                    "name": "oldProtocolFeeRate",
                    "type": "u32",
                    "index": false
                },
                {
                    "name": "protocolFeeRate",
                    "type": "u32",
                    "index": false
                }
            ]
        },
        {
            "name": "FinalizedVoteResultsEvent",
            "fields": [
                {
                    "name": "proposal",
                    "type": "publicKey",
                    "index": false
                },
                {
                    "name": "time",
                    "type": "i64",
                    "index": false
                },
                {
                    "name": "revealedVotes",
                    "type": "u64",
                    "index": false
                },
                {
                    "name": "consensus",
                    "type": "i64",
                    "index": false
                },
                {
                    "name": "status",
                    "type": "u8",
                    "index": false
                }
            ]
        },
        {
            "name": "FinalizedVoteEvent",
            "fields": [
                {
                    "name": "proposal",
                    "type": "publicKey",
                    "index": false
                },
                {
                    "name": "time",
                    "type": "i64",
                    "index": false
                }
            ]
        },
        {
            "name": "ProposeVoteEvent",
            "fields": [
                {
                    "name": "name",
                    "type": "string",
                    "index": false
                },
                {
                    "name": "description",
                    "type": "string",
                    "index": false
                },
                {
                    "name": "id",
                    "type": "bytes",
                    "index": false
                },
                {
                    "name": "proposer",
                    "type": "publicKey",
                    "index": false
                },
                {
                    "name": "stake",
                    "type": "u64",
                    "index": false
                }
            ]
        },
        {
            "name": "RevealedVoteEvent",
            "fields": [
                {
                    "name": "proposal",
                    "type": "publicKey",
                    "index": false
                },
                {
                    "name": "time",
                    "type": "i64",
                    "index": false
                },
                {
                    "name": "revealedVote",
                    "type": "i64",
                    "index": false
                },
                {
                    "name": "votePower",
                    "type": "u32",
                    "index": false
                }
            ]
        },
        {
            "name": "SubmittedVoteEvent",
            "fields": [
                {
                    "name": "proposal",
                    "type": "publicKey",
                    "index": false
                },
                {
                    "name": "time",
                    "type": "i64",
                    "index": false
                },
                {
                    "name": "voteHash",
                    "type": "bytes",
                    "index": false
                },
                {
                    "name": "votePower",
                    "type": "u64",
                    "index": false
                }
            ]
        },
        {
            "name": "UpdatedVoteEvent",
            "fields": [
                {
                    "name": "proposal",
                    "type": "publicKey",
                    "index": false
                },
                {
                    "name": "time",
                    "type": "i64",
                    "index": false
                },
                {
                    "name": "voteHash",
                    "type": "bytes",
                    "index": false
                }
            ]
        }
    ],
    "errors": [
        {
            "code": 6000,
            "name": "StakeTooLittle",
            "msg": "Not enough staked on vote"
        },
        {
            "code": 6001,
            "name": "InvalidLockPeriod",
            "msg": "Invalid lock period"
        },
        {
            "code": 6002,
            "name": "InvalidVoteEndTime",
            "msg": "Invalid vote end time"
        },
        {
            "code": 6003,
            "name": "VotingPeriodEnded",
            "msg": "Voting period for proposal has ended"
        },
        {
            "code": 6004,
            "name": "RevealPeriodNotActive",
            "msg": "Currently not in vote reveal period"
        },
        {
            "code": 6005,
            "name": "RevealPeriodIsNotFinished",
            "msg": "Reveal period is not over"
        },
        {
            "code": 6006,
            "name": "InvalidSalt",
            "msg": "Invalid salt resulted in invalid vote_hash"
        },
        {
            "code": 6007,
            "name": "FullRevealList",
            "msg": "Revealed vote list full"
        },
        {
            "code": 6008,
            "name": "VoteNotRevealed",
            "msg": "Vote hasn't been revealed"
        },
        {
            "code": 6009,
            "name": "OverflowU64",
            "msg": "U64 overflow"
        },
        {
            "code": 6010,
            "name": "OverflowU32",
            "msg": "U32 overflow"
        },
        {
            "code": 6011,
            "name": "NotPossibleToCalculateVoteReward",
            "msg": "Could not calculate the vote reward at this time"
        },
        {
            "code": 6012,
            "name": "NotPossibleToCollectProposerReward",
            "msg": "Cannot payout the proposer reward at this time"
        },
        {
            "code": 6013,
            "name": "NotPossibleToCollectVoterReward",
            "msg": "Cannot payout the voter reward at this time"
        },
        {
            "code": 6014,
            "name": "FailedToFinalizeVote",
            "msg": "Cannot finalize user vote at this time"
        },
        {
            "code": 6015,
            "name": "FailedToFinalizeVoteResult",
            "msg": "Cannot finalize vote result at this time"
        },
        {
            "code": 6016,
            "name": "FailedToCancelVote",
            "msg": "Too late to cancel vote"
        },
        {
            "code": 6017,
            "name": "InvalidOwnerOfVoteAccount",
            "msg": "The owner of the vote account is not the signer"
        },
        {
            "code": 6018,
            "name": "ProposalVaultMintKeyDoesNotMatchProposalStateVaultMint",
            "msg": "Proposal.vault_mint does not match the input proposal_vault_mint key"
        },
        {
            "code": 6019,
            "name": "ProposalVaultMintKeyDoesNotMatchVaultMint",
            "msg": "Proposal.vault_mint does not match the vault mint key"
        },
        {
            "code": 6020,
            "name": "NotEnoughProposalStake",
            "msg": "Not enough stake to propose a vote "
        },
        {
            "code": 6021,
            "name": "InvalidRequiredVotesParam",
            "msg": "Quorum requirements are too low"
        },
        {
            "code": 6022,
            "name": "InvalidMinimumStakedParam",
            "msg": "Invalid minimum staked on proposal"
        },
        {
            "code": 6023,
            "name": "InvalidVoteStakeRateParam",
            "msg": "Invalid vote stake rate param. Probably less than 1"
        },
        {
            "code": 6024,
            "name": "InvalidProtocolFeeRateParam",
            "msg": "Invalid protocol fee rate param. Probably less than 1"
        },
        {
            "code": 6025,
            "name": "UnauthorizedSigner",
            "msg": "Unauthorized signer"
        }
    ]
};
//# sourceMappingURL=oracle.js.map