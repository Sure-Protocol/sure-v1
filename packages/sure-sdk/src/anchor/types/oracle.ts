export type Oracle = {
  "version": "0.1.0",
  "name": "oracle",
  "instructions": [
    {
      "name": "initialize",
      "accounts": [],
      "args": []
    }
  ],
  "accounts": [
    {
      "name": "proposal",
      "type": {
        "kind": "struct",
        "fields": [
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
            "name": "name",
            "docs": [
              "name of vote"
            ],
            "type": "string"
          },
          {
            "name": "description",
            "docs": [
              "description of vote"
            ],
            "type": "string"
          },
          {
            "name": "proposer",
            "docs": [
              "user who proposed the vote"
            ],
            "type": "publicKey"
          },
          {
            "name": "tokenMintReward",
            "docs": [
              "Token mint to distribute rewards"
            ],
            "type": "publicKey"
          },
          {
            "name": "proposedStaked",
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
            "name": "votes",
            "docs": [
              "Current votes given in basis points",
              "1 vote = 1 veToken@",
              "Q64.0"
            ],
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
            "name": "scaleParameter",
            "docs": [
              "Scale parameter in exp(L)",
              "Q16.16"
            ],
            "type": "u32"
          },
          {
            "name": "instructions",
            "docs": [
              "Instruction to be exectued if passed"
            ],
            "type": {
              "array": [
                {
                  "defined": "VoteInstruction"
                },
                32
              ]
            }
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
                1240
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
      "name": "voteInstruction",
      "docs": [
        "Invoked if a vote is successful"
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "programId",
            "type": "publicKey"
          },
          {
            "name": "keys",
            "type": {
              "array": [
                {
                  "defined": "AccountKeys"
                },
                24
              ]
            }
          },
          {
            "name": "data",
            "type": {
              "array": [
                "u8",
                24
              ]
            }
          }
        ]
      }
    },
    {
      "name": "accountKeys",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "accountPubkey",
            "type": "publicKey"
          },
          {
            "name": "isSigner",
            "type": "bool"
          },
          {
            "name": "isWritable",
            "type": "bool"
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
            "name": "Failed"
          },
          {
            "name": "InActive"
          }
        ]
      }
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
      "name": "InvalidSalt",
      "msg": "Invalid salt resulted in invalid vote_hash"
    },
    {
      "code": 6005,
      "name": "FullRevealList",
      "msg": "Revealed vote list full"
    },
    {
      "code": 6006,
      "name": "VoteNotRevealed",
      "msg": "Vote hasn't been revealed"
    },
    {
      "code": 6007,
      "name": "OverflowU64",
      "msg": "U64 overflow"
    },
    {
      "code": 6008,
      "name": "OverflowU32",
      "msg": "U32 overflow"
    }
  ]
};

export const IDL: Oracle = {
  "version": "0.1.0",
  "name": "oracle",
  "instructions": [
    {
      "name": "initialize",
      "accounts": [],
      "args": []
    }
  ],
  "accounts": [
    {
      "name": "proposal",
      "type": {
        "kind": "struct",
        "fields": [
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
            "name": "name",
            "docs": [
              "name of vote"
            ],
            "type": "string"
          },
          {
            "name": "description",
            "docs": [
              "description of vote"
            ],
            "type": "string"
          },
          {
            "name": "proposer",
            "docs": [
              "user who proposed the vote"
            ],
            "type": "publicKey"
          },
          {
            "name": "tokenMintReward",
            "docs": [
              "Token mint to distribute rewards"
            ],
            "type": "publicKey"
          },
          {
            "name": "proposedStaked",
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
            "name": "votes",
            "docs": [
              "Current votes given in basis points",
              "1 vote = 1 veToken@",
              "Q64.0"
            ],
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
            "name": "scaleParameter",
            "docs": [
              "Scale parameter in exp(L)",
              "Q16.16"
            ],
            "type": "u32"
          },
          {
            "name": "instructions",
            "docs": [
              "Instruction to be exectued if passed"
            ],
            "type": {
              "array": [
                {
                  "defined": "VoteInstruction"
                },
                32
              ]
            }
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
                1240
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
      "name": "voteInstruction",
      "docs": [
        "Invoked if a vote is successful"
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "programId",
            "type": "publicKey"
          },
          {
            "name": "keys",
            "type": {
              "array": [
                {
                  "defined": "AccountKeys"
                },
                24
              ]
            }
          },
          {
            "name": "data",
            "type": {
              "array": [
                "u8",
                24
              ]
            }
          }
        ]
      }
    },
    {
      "name": "accountKeys",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "accountPubkey",
            "type": "publicKey"
          },
          {
            "name": "isSigner",
            "type": "bool"
          },
          {
            "name": "isWritable",
            "type": "bool"
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
            "name": "Failed"
          },
          {
            "name": "InActive"
          }
        ]
      }
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
      "name": "InvalidSalt",
      "msg": "Invalid salt resulted in invalid vote_hash"
    },
    {
      "code": 6005,
      "name": "FullRevealList",
      "msg": "Revealed vote list full"
    },
    {
      "code": 6006,
      "name": "VoteNotRevealed",
      "msg": "Vote hasn't been revealed"
    },
    {
      "code": 6007,
      "name": "OverflowU64",
      "msg": "U64 overflow"
    },
    {
      "code": 6008,
      "name": "OverflowU32",
      "msg": "U32 overflow"
    }
  ]
};
