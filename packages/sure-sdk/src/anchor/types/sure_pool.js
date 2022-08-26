"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.IDL = void 0;
exports.IDL = {
    "version": "0.0.1",
    "name": "sure_pool",
    "instructions": [
        {
            "name": "initializePool",
            "docs": [
                "Create an insurance pool for a smart contract",
                "also create an associated vault to hold the tokens",
                "",
                "# Arguments",
                "* ctx:",
                "* insurance_fee: fee taken on each insurance bought. In basis points (1bp = 0.01%)",
                "* range_size: The size of the ranges in which users can provide insurance",
                "* name: [optional] Name of the pool"
            ],
            "accounts": [
                {
                    "name": "creator",
                    "isMut": true,
                    "isSigner": true,
                    "docs": [
                        "Pool creator"
                    ]
                },
                {
                    "name": "pool",
                    "isMut": true,
                    "isSigner": false,
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "type": "string",
                                "value": "sure-insurance-pool"
                            },
                            {
                                "kind": "account",
                                "type": "publicKey",
                                "path": "smart_contract"
                            }
                        ]
                    }
                },
                {
                    "name": "feePackage",
                    "isMut": false,
                    "isSigner": false
                },
                {
                    "name": "smartContract",
                    "isMut": false,
                    "isSigner": false,
                    "docs": [
                        "that is to be insured."
                    ]
                },
                {
                    "name": "tokenMint0",
                    "isMut": false,
                    "isSigner": false
                },
                {
                    "name": "tokenMint1",
                    "isMut": false,
                    "isSigner": false
                },
                {
                    "name": "vault0",
                    "isMut": true,
                    "isSigner": false,
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "type": "string",
                                "value": "sure-liquidity-vault"
                            },
                            {
                                "kind": "account",
                                "type": "publicKey",
                                "account": "Pool",
                                "path": "pool"
                            },
                            {
                                "kind": "account",
                                "type": "publicKey",
                                "account": "Mint",
                                "path": "token_mint_0"
                            }
                        ]
                    }
                },
                {
                    "name": "vault1",
                    "isMut": true,
                    "isSigner": false,
                    "pda": {
                        "seeds": [
                            {
                                "kind": "const",
                                "type": "string",
                                "value": "sure-premium-vault"
                            },
                            {
                                "kind": "account",
                                "type": "publicKey",
                                "account": "Pool",
                                "path": "pool"
                            },
                            {
                                "kind": "account",
                                "type": "publicKey",
                                "account": "Mint",
                                "path": "token_mint_1"
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
                    "isSigner": false,
                    "docs": [
                        "Sysvar for Associated Token Account"
                    ]
                },
                {
                    "name": "systemProgram",
                    "isMut": false,
                    "isSigner": false,
                    "docs": [
                        "Provide the system program"
                    ]
                }
            ],
            "args": [
                {
                    "name": "name",
                    "type": "string"
                },
                {
                    "name": "tickSpacing",
                    "type": "u16"
                }
            ]
        }
    ],
    "accounts": [
        {
            "name": "bitMap",
            "docs": [
                "Bitmap used to keep track of liquidity at each tick"
            ],
            "type": {
                "kind": "struct",
                "fields": [
                    {
                        "name": "bump",
                        "docs": [
                            "Bump"
                        ],
                        "type": "u8"
                    },
                    {
                        "name": "wordPos",
                        "type": "i16"
                    },
                    {
                        "name": "spacing",
                        "type": "u16"
                    },
                    {
                        "name": "word",
                        "docs": [
                            "Map"
                        ],
                        "type": {
                            "array": [
                                "u64",
                                4
                            ]
                        }
                    }
                ]
            }
        },
        {
            "name": "coveragePosition",
            "docs": [
                "--- Pool insurance contract ---",
                "<POOL>",
                "Accumulation of all insurance contracts for a user in",
                "a given pool."
            ],
            "type": {
                "kind": "struct",
                "fields": [
                    {
                        "name": "bump",
                        "docs": [
                            "The bump"
                        ],
                        "type": "u8"
                    },
                    {
                        "name": "expiryTs",
                        "docs": [
                            "Contract expiry"
                        ],
                        "type": "i64"
                    },
                    {
                        "name": "insuredAmount",
                        "docs": [
                            "Contract Amount"
                        ],
                        "type": "u64"
                    },
                    {
                        "name": "positionMint",
                        "docs": [
                            "Position mint representing the postiion"
                        ],
                        "type": "publicKey"
                    },
                    {
                        "name": "tokenMint",
                        "docs": [
                            "token mint"
                        ],
                        "type": "publicKey"
                    },
                    {
                        "name": "owner",
                        "docs": [
                            "Owner of contract"
                        ],
                        "type": "publicKey"
                    }
                ]
            }
        },
        {
            "name": "insuranceTickContract",
            "docs": [
                "--- Insurance Contract --",
                "<TICK>",
                "Holds state about an insurance contract for a specific tick"
            ],
            "type": {
                "kind": "struct",
                "fields": [
                    {
                        "name": "bump",
                        "docs": [
                            "The bump identity of the PDA"
                        ],
                        "type": "u8"
                    },
                    {
                        "name": "insuredAmount",
                        "docs": [
                            "Amount insured"
                        ],
                        "type": "u64"
                    },
                    {
                        "name": "premium",
                        "docs": [
                            "Premium"
                        ],
                        "type": "u64"
                    },
                    {
                        "name": "endTs",
                        "docs": [
                            "The end time of the contract"
                        ],
                        "type": "i64"
                    },
                    {
                        "name": "startTs",
                        "docs": [
                            "Start time of contract"
                        ],
                        "type": "i64"
                    },
                    {
                        "name": "pool",
                        "docs": [
                            "Insured pool"
                        ],
                        "type": "publicKey"
                    },
                    {
                        "name": "liquidityTickInfo",
                        "docs": [
                            "Tick Account used to buy from"
                        ],
                        "type": "publicKey"
                    },
                    {
                        "name": "tokenMint",
                        "type": "publicKey"
                    },
                    {
                        "name": "active",
                        "docs": [
                            "Is the insurance contract active"
                        ],
                        "type": "bool"
                    },
                    {
                        "name": "updatedTs",
                        "docs": [
                            "Updated"
                        ],
                        "type": "i64"
                    },
                    {
                        "name": "createdTs",
                        "docs": [
                            "Created"
                        ],
                        "type": "i64"
                    }
                ]
            }
        },
        {
            "name": "feePackage",
            "type": {
                "kind": "struct",
                "fields": [
                    {
                        "name": "bump",
                        "type": "u8"
                    },
                    {
                        "name": "owner",
                        "type": "publicKey"
                    },
                    {
                        "name": "feeRate",
                        "type": "u16"
                    },
                    {
                        "name": "foundersFee",
                        "type": "u16"
                    },
                    {
                        "name": "protocolFee",
                        "type": "u16"
                    }
                ]
            }
        },
        {
            "name": "liquidityPosition",
            "docs": [
                "-- Liquidity Position --",
                "",
                "Holds information about liquidity at a given tick",
                ""
            ],
            "type": {
                "kind": "struct",
                "fields": [
                    {
                        "name": "bump",
                        "docs": [
                            "Bump Identity"
                        ],
                        "type": "u8"
                    },
                    {
                        "name": "liquidity",
                        "docs": [
                            "The amount of liquidity provided in lamports"
                        ],
                        "type": "u128"
                    },
                    {
                        "name": "usedLiquidity",
                        "docs": [
                            "the amount of liquidity used"
                        ],
                        "type": "u128"
                    },
                    {
                        "name": "pool",
                        "docs": [
                            "Liquidity Pool"
                        ],
                        "type": "publicKey"
                    },
                    {
                        "name": "positionMint",
                        "docs": [
                            "NFT mint. The mint representing the position",
                            "The NFT is the owner of the position."
                        ],
                        "type": "publicKey"
                    },
                    {
                        "name": "tickIndexLower",
                        "docs": [
                            "Id in the tick pool"
                        ],
                        "type": "i32"
                    },
                    {
                        "name": "tickIndexUpper",
                        "docs": [
                            "The tick that the liquidity is at"
                        ],
                        "type": "i32"
                    },
                    {
                        "name": "owedFees",
                        "docs": [
                            "Outstanding Rewards"
                        ],
                        "type": "u32"
                    },
                    {
                        "name": "owedPremium",
                        "type": "u32"
                    }
                ]
            }
        },
        {
            "name": "protocolOwner",
            "docs": [
                "Owner of the Sure Protocol",
                "",
                "# Capabilities",
                "",
                "* Adjust pool fee",
                "* Mint tokens",
                ""
            ],
            "type": {
                "kind": "struct",
                "fields": [
                    {
                        "name": "bump",
                        "docs": [
                            "Bump"
                        ],
                        "type": "u8"
                    },
                    {
                        "name": "owner",
                        "docs": [
                            "Owner of the protocol"
                        ],
                        "type": "publicKey"
                    }
                ]
            }
        },
        {
            "name": "poolManager",
            "docs": [
                "Account describing the pool manager",
                ""
            ],
            "type": {
                "kind": "struct",
                "fields": [
                    {
                        "name": "owner",
                        "type": "publicKey"
                    },
                    {
                        "name": "bump",
                        "type": "u8"
                    }
                ]
            }
        },
        {
            "name": "surePools",
            "docs": [
                "SurePools holds information on which programs are",
                "insured by Sure"
            ],
            "type": {
                "kind": "struct",
                "fields": [
                    {
                        "name": "bump",
                        "type": "u8"
                    },
                    {
                        "name": "pools",
                        "docs": [
                            "Vec of insured programs"
                        ],
                        "type": {
                            "vec": "publicKey"
                        }
                    }
                ]
            }
        },
        {
            "name": "pool",
            "docs": [
                "Pool Account (PDA) contains information describing the",
                "insurance pool"
            ],
            "type": {
                "kind": "struct",
                "fields": [
                    {
                        "name": "bump",
                        "docs": [
                            "Bump to identify the PDA"
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
                            "",
                            "Name of pool visible to the user"
                        ],
                        "type": "string"
                    },
                    {
                        "name": "tickSpacing",
                        "type": "u16"
                    },
                    {
                        "name": "tickSpacingArray",
                        "type": {
                            "array": [
                                "u8",
                                2
                            ]
                        }
                    },
                    {
                        "name": "founder",
                        "type": "publicKey"
                    },
                    {
                        "name": "feeRate",
                        "docs": [
                            "fees",
                            "100th of a basis point"
                        ],
                        "type": "u16"
                    },
                    {
                        "name": "protocolFee",
                        "docs": [
                            "(1/x)% of fee_rate"
                        ],
                        "type": "u16"
                    },
                    {
                        "name": "foundersFee",
                        "type": "u16"
                    },
                    {
                        "name": "insuranceFee",
                        "docs": [
                            "Fee paid when buying insurance.",
                            "in basis points"
                        ],
                        "type": "u16"
                    },
                    {
                        "name": "smartContract",
                        "docs": [
                            "The public key of the smart contract that is",
                            "insured"
                        ],
                        "type": "publicKey"
                    },
                    {
                        "name": "tokenPools",
                        "docs": [
                            "Vec of token Pools"
                        ],
                        "type": {
                            "vec": "publicKey"
                        }
                    },
                    {
                        "name": "locked",
                        "docs": [
                            "Whether the insurance pool is locked"
                        ],
                        "type": "bool"
                    },
                    {
                        "name": "tokenMint0",
                        "type": "publicKey"
                    },
                    {
                        "name": "vault0",
                        "type": "publicKey"
                    },
                    {
                        "name": "tokenMint1",
                        "docs": [
                            "Token mint B of pool"
                        ],
                        "type": "publicKey"
                    },
                    {
                        "name": "vault1",
                        "type": "publicKey"
                    },
                    {
                        "name": "usedLiquidity",
                        "docs": [
                            "Used liquidity"
                        ],
                        "type": "u128"
                    }
                ]
            }
        },
        {
            "name": "tickArray",
            "docs": [
                "Tick Array",
                "",
                "An array of Ticks with infor",
                ""
            ],
            "type": {
                "kind": "struct",
                "fields": [
                    {
                        "name": "startTickIndex",
                        "type": "i32"
                    },
                    {
                        "name": "ticks",
                        "type": {
                            "array": [
                                {
                                    "defined": "Tick"
                                },
                                64
                            ]
                        }
                    },
                    {
                        "name": "pool",
                        "type": "publicKey"
                    }
                ]
            }
        }
    ],
    "types": [
        {
            "name": "Tick",
            "docs": [
                "Tick"
            ],
            "type": {
                "kind": "struct",
                "fields": [
                    {
                        "name": "bump",
                        "type": "u8"
                    },
                    {
                        "name": "liquidityGross",
                        "docs": [
                            "Amount of liquidity added (removed, if neg)",
                            "when the tick is crossed going left to right."
                        ],
                        "type": "u128"
                    },
                    {
                        "name": "liquidityUsed",
                        "docs": [
                            "Locked liquidity indicates how much of the",
                            "liquidity is locked in long term commitments"
                        ],
                        "type": "u128"
                    }
                ]
            }
        },
        {
            "name": "ProductType",
            "type": {
                "kind": "enum",
                "variants": [
                    {
                        "name": "Coverage"
                    },
                    {
                        "name": "AMM"
                    }
                ]
            }
        }
    ],
    "events": [
        {
            "name": "NewLiquidityPosition",
            "fields": [
                {
                    "name": "tick",
                    "type": "u16",
                    "index": false
                },
                {
                    "name": "liquidity",
                    "type": "u64",
                    "index": false
                }
            ]
        },
        {
            "name": "CreatePool",
            "fields": [
                {
                    "name": "name",
                    "type": "string",
                    "index": true
                },
                {
                    "name": "smartContract",
                    "type": "publicKey",
                    "index": false
                }
            ]
        },
        {
            "name": "ChangeProtocolOwner",
            "fields": [
                {
                    "name": "owner",
                    "type": "publicKey",
                    "index": false
                },
                {
                    "name": "oldOwner",
                    "type": "publicKey",
                    "index": false
                }
            ]
        },
        {
            "name": "CreatePool",
            "fields": [
                {
                    "name": "name",
                    "type": "string",
                    "index": true
                },
                {
                    "name": "smartContract",
                    "type": "publicKey",
                    "index": false
                },
                {
                    "name": "insuranceFee",
                    "type": "u16",
                    "index": false
                }
            ]
        },
        {
            "name": "InitializeTokenPool",
            "fields": []
        },
        {
            "name": "InitializedManager",
            "fields": [
                {
                    "name": "owner",
                    "type": "publicKey",
                    "index": true
                }
            ]
        }
    ],
    "errors": [
        {
            "code": 6000,
            "name": "InvalidMint",
            "msg": "Invalid mint"
        },
        {
            "code": 6001,
            "name": "InvalidRangeSize",
            "msg": "Invalid Range size"
        },
        {
            "code": 6002,
            "name": "InvalidTick",
            "msg": "Invalid tick to provide liquidity to"
        },
        {
            "code": 6003,
            "name": "InvalidAmount",
            "msg": "Invalid Amount"
        },
        {
            "code": 6004,
            "name": "LiquidityFilled",
            "msg": "All of the liquidity is used"
        },
        {
            "code": 6005,
            "name": "InvalidPoolCreator",
            "msg": "Invalid Pool creator provided. Are you sure you are the protocol owner?"
        },
        {
            "code": 6006,
            "name": "CouldNotProvideLiquidity",
            "msg": "Could not provide liquidity"
        },
        {
            "code": 6007,
            "name": "TickAccountNotEmpty",
            "msg": "Not empty Tick account"
        },
        {
            "code": 6008,
            "name": "InvalidTimestamp",
            "msg": "Invalid timestamp"
        },
        {
            "code": 6009,
            "name": "InsuranceContractExpired",
            "msg": "Insurance Contract has expired"
        },
        {
            "code": 6010,
            "name": "InsuranceContractIsNotActive",
            "msg": "Insurance Contract is not active"
        },
        {
            "code": 6011,
            "name": "TickOutsideSpacing",
            "msg": "Invalid Tick: Between tick spaces"
        },
        {
            "code": 6012,
            "name": "TickOutOfRange",
            "msg": "Tick index is not within the tick array range"
        },
        {
            "code": 6013,
            "name": "TickLtTickArray",
            "msg": "Tick is below tick array "
        },
        {
            "code": 6014,
            "name": "InvalidTickSpacing",
            "msg": "Invalid tick spacing. Tick spacing might be 0."
        },
        {
            "code": 6015,
            "name": "InvalidTickArrayIndexInTickArrayPool",
            "msg": "Tick array not found in tick array pool"
        },
        {
            "code": 6016,
            "name": "LiquidityTooLarge",
            "msg": "Provided Liquidity is too large"
        },
        {
            "code": 6017,
            "name": "LiquidityHaveToBeGreaterThan0",
            "msg": "The provided liquidity have to be greater than 0"
        },
        {
            "code": 6018,
            "name": "PoolsInProductPoolExceeded",
            "msg": "Number of pools in product pools is exceeded"
        },
        {
            "code": 6019,
            "name": "ProductPoolIsEmpty",
            "msg": "Product Pool is empty"
        },
        {
            "code": 6020,
            "name": "MaxFeeRateExceeded",
            "msg": "Fee rate exceeds the max of 10 000bp = 100%"
        },
        {
            "code": 6021,
            "name": "MaxProtocolFeeRateExceeded",
            "msg": "The Protocol fee rate exceeded 3 200bp=33%"
        },
        {
            "code": 6022,
            "name": "InvalidSubFeeRates",
            "msg": "The sum of the sub fee rates exceeds the fee_rate"
        },
        {
            "code": 6023,
            "name": "MaxFoundersFeeRateExceeded",
            "msg": "The max founders fee is exceeded"
        },
        {
            "code": 6024,
            "name": "TooLowLiquidityProviderFeeRate",
            "msg": "The Liquidity Provider fee rate is too low"
        },
        {
            "code": 6025,
            "name": "SqrtRatioNotWithinRange",
            "msg": "Square root price ratio is not within ranges"
        },
        {
            "code": 6026,
            "name": "WrongTokenMintOrder",
            "msg": "The ordering of token mint are wrong"
        },
        {
            "code": 6027,
            "name": "TooLargeWordPosition",
            "msg": "The word position is too large"
        },
        {
            "code": 6028,
            "name": "TooSmallWordPosition",
            "msg": "The word position is too small"
        },
        {
            "code": 6029,
            "name": "InvalidTickArrayWord",
            "msg": "The specified word does not match the given tick array"
        },
        {
            "code": 6030,
            "name": "InvalidTickIndexProvided",
            "msg": "Invalid upper and lower tick provided"
        },
        {
            "code": 6031,
            "name": "InvalidUpperTickIndexProvided",
            "msg": "Invalid upper tick provided"
        },
        {
            "code": 6032,
            "name": "InvalidLowerTickIndexProvided",
            "msg": "Invalid lower tick provided"
        },
        {
            "code": 6033,
            "name": "LowerTickgtUpperTick",
            "msg": "Lower tick gt upper tick"
        },
        {
            "code": 6034,
            "name": "InvalidOwner",
            "msg": "Not a valid owner. The expected user does not have ownership over the account"
        },
        {
            "code": 6035,
            "name": "CouldNotUpdatePoolLiquidity",
            "msg": "Could not update the pool liquidity"
        },
        {
            "code": 6036,
            "name": "LiquidityOverflow",
            "msg": "Liquidity change causes total liquidity to overflow"
        },
        {
            "code": 6037,
            "name": "LiquidityUnderflow",
            "msg": "Liquidity change causes total liquidity to underflow"
        },
        {
            "code": 6038,
            "name": "InvalidFeeGrowthSubtraction",
            "msg": "Invalid fee growth subtraction"
        },
        {
            "code": 6039,
            "name": "MultiplictationQ3232Overflow",
            "msg": "Q32.32 multiplication overflow"
        },
        {
            "code": 6040,
            "name": "DivisionQ3232Error",
            "msg": "Q32.32 division error"
        },
        {
            "code": 6041,
            "name": "SubtractionQ3232Error",
            "msg": "Q32.32 Substraction error"
        },
        {
            "code": 6042,
            "name": "AdditionQ3232OverflowError",
            "msg": "Q32.32 Addition overflow"
        },
        {
            "code": 6043,
            "name": "OverflowU64",
            "msg": "U64 overflow"
        },
        {
            "code": 6044,
            "name": "InvalidProductTypeId",
            "msg": "Invalid product type id"
        }
    ]
};
//# sourceMappingURL=sure_pool.js.map