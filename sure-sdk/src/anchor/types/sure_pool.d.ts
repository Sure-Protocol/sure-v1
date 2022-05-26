export declare type SurePool = {
    "version": "0.0.1";
    "name": "sure_pool";
    "instructions": [
        {
            "name": "initializeProtocol";
            "accounts": [
                {
                    "name": "owner";
                    "isMut": true;
                    "isSigner": true;
                },
                {
                    "name": "protocolOwner";
                    "isMut": true;
                    "isSigner": false;
                },
                {
                    "name": "pools";
                    "isMut": true;
                    "isSigner": false;
                },
                {
                    "name": "systemProgram";
                    "isMut": false;
                    "isSigner": false;
                }
            ];
            "args": [];
        },
        {
            "name": "initializePoolManager";
            "accounts": [
                {
                    "name": "manager";
                    "isMut": true;
                    "isSigner": false;
                },
                {
                    "name": "initialManager";
                    "isMut": true;
                    "isSigner": true;
                },
                {
                    "name": "systemProgram";
                    "isMut": false;
                    "isSigner": false;
                }
            ];
            "args": [];
        },
        {
            "name": "createPool";
            "accounts": [
                {
                    "name": "poolCreator";
                    "isMut": true;
                    "isSigner": true;
                },
                {
                    "name": "protocolOwner";
                    "isMut": false;
                    "isSigner": false;
                },
                {
                    "name": "pool";
                    "isMut": true;
                    "isSigner": false;
                },
                {
                    "name": "surePools";
                    "isMut": true;
                    "isSigner": false;
                },
                {
                    "name": "insuredTokenAccount";
                    "isMut": false;
                    "isSigner": false;
                },
                {
                    "name": "rent";
                    "isMut": false;
                    "isSigner": false;
                },
                {
                    "name": "systemProgram";
                    "isMut": false;
                    "isSigner": false;
                }
            ];
            "args": [
                {
                    "name": "insuranceFee";
                    "type": "u16";
                },
                {
                    "name": "tickSpacing";
                    "type": "u16";
                },
                {
                    "name": "name";
                    "type": "string";
                }
            ];
        },
        {
            "name": "createPoolVaults";
            "accounts": [
                {
                    "name": "creator";
                    "isMut": true;
                    "isSigner": true;
                },
                {
                    "name": "pool";
                    "isMut": false;
                    "isSigner": false;
                },
                {
                    "name": "tokenMint";
                    "isMut": false;
                    "isSigner": false;
                },
                {
                    "name": "liquidityVault";
                    "isMut": true;
                    "isSigner": false;
                },
                {
                    "name": "premiumVault";
                    "isMut": true;
                    "isSigner": false;
                },
                {
                    "name": "bitmap";
                    "isMut": true;
                    "isSigner": false;
                },
                {
                    "name": "rent";
                    "isMut": false;
                    "isSigner": false;
                },
                {
                    "name": "tokenProgram";
                    "isMut": false;
                    "isSigner": false;
                },
                {
                    "name": "systemProgram";
                    "isMut": false;
                    "isSigner": false;
                }
            ];
            "args": [];
        },
        {
            "name": "depositLiquidity";
            "accounts": [
                {
                    "name": "liquidityProvider";
                    "isMut": true;
                    "isSigner": true;
                },
                {
                    "name": "protocolOwner";
                    "isMut": false;
                    "isSigner": false;
                },
                {
                    "name": "liquidityProviderAccount";
                    "isMut": true;
                    "isSigner": false;
                },
                {
                    "name": "pool";
                    "isMut": true;
                    "isSigner": false;
                },
                {
                    "name": "vault";
                    "isMut": true;
                    "isSigner": false;
                },
                {
                    "name": "nftMint";
                    "isMut": true;
                    "isSigner": false;
                },
                {
                    "name": "metadataAccount";
                    "isMut": true;
                    "isSigner": false;
                },
                {
                    "name": "metadataProgram";
                    "isMut": false;
                    "isSigner": false;
                },
                {
                    "name": "liquidityPosition";
                    "isMut": true;
                    "isSigner": false;
                },
                {
                    "name": "nftAccount";
                    "isMut": true;
                    "isSigner": false;
                },
                {
                    "name": "bitmap";
                    "isMut": true;
                    "isSigner": false;
                },
                {
                    "name": "tickAccount";
                    "isMut": true;
                    "isSigner": false;
                },
                {
                    "name": "rent";
                    "isMut": false;
                    "isSigner": false;
                },
                {
                    "name": "tokenProgram";
                    "isMut": false;
                    "isSigner": false;
                },
                {
                    "name": "systemProgram";
                    "isMut": false;
                    "isSigner": false;
                },
                {
                    "name": "associatedTokenProgram";
                    "isMut": false;
                    "isSigner": false;
                }
            ];
            "args": [
                {
                    "name": "tick";
                    "type": "u16";
                },
                {
                    "name": "tickPos";
                    "type": "u64";
                },
                {
                    "name": "amount";
                    "type": "u64";
                }
            ];
        },
        {
            "name": "redeemLiquidity";
            "accounts": [
                {
                    "name": "nftHolder";
                    "isMut": false;
                    "isSigner": true;
                },
                {
                    "name": "nftAccount";
                    "isMut": false;
                    "isSigner": false;
                },
                {
                    "name": "protocolOwner";
                    "isMut": false;
                    "isSigner": false;
                },
                {
                    "name": "liquidityPosition";
                    "isMut": true;
                    "isSigner": false;
                },
                {
                    "name": "tokenAccount";
                    "isMut": true;
                    "isSigner": false;
                },
                {
                    "name": "vault";
                    "isMut": true;
                    "isSigner": false;
                },
                {
                    "name": "tickAccount";
                    "isMut": true;
                    "isSigner": false;
                },
                {
                    "name": "metadataAccount";
                    "isMut": true;
                    "isSigner": false;
                },
                {
                    "name": "metadataProgram";
                    "isMut": false;
                    "isSigner": false;
                },
                {
                    "name": "pool";
                    "isMut": true;
                    "isSigner": false;
                },
                {
                    "name": "tokenProgram";
                    "isMut": false;
                    "isSigner": false;
                },
                {
                    "name": "systemProgram";
                    "isMut": false;
                    "isSigner": false;
                }
            ];
            "args": [];
        },
        {
            "name": "initializeInsuranceContract";
            "accounts": [
                {
                    "name": "owner";
                    "isMut": true;
                    "isSigner": true;
                },
                {
                    "name": "pool";
                    "isMut": false;
                    "isSigner": false;
                },
                {
                    "name": "tokenMint";
                    "isMut": false;
                    "isSigner": false;
                },
                {
                    "name": "tickAccount";
                    "isMut": false;
                    "isSigner": false;
                },
                {
                    "name": "insuranceContract";
                    "isMut": true;
                    "isSigner": false;
                },
                {
                    "name": "insuranceContracts";
                    "isMut": true;
                    "isSigner": false;
                },
                {
                    "name": "systemProgram";
                    "isMut": false;
                    "isSigner": false;
                }
            ];
            "args": [];
        },
        {
            "name": "initializeUserInsuranceContracts";
            "accounts": [
                {
                    "name": "signer";
                    "isMut": true;
                    "isSigner": true;
                },
                {
                    "name": "pool";
                    "isMut": false;
                    "isSigner": false;
                },
                {
                    "name": "insuranceContracts";
                    "isMut": true;
                    "isSigner": false;
                },
                {
                    "name": "systemProgram";
                    "isMut": false;
                    "isSigner": false;
                }
            ];
            "args": [];
        },
        {
            "name": "buyInsuranceForTick";
            "accounts": [
                {
                    "name": "buyer";
                    "isMut": true;
                    "isSigner": true;
                },
                {
                    "name": "tokenAccount";
                    "isMut": true;
                    "isSigner": false;
                },
                {
                    "name": "pool";
                    "isMut": true;
                    "isSigner": false;
                },
                {
                    "name": "tickAccount";
                    "isMut": true;
                    "isSigner": false;
                },
                {
                    "name": "premiumVault";
                    "isMut": true;
                    "isSigner": false;
                },
                {
                    "name": "insuranceContract";
                    "isMut": true;
                    "isSigner": false;
                },
                {
                    "name": "tokenProgram";
                    "isMut": false;
                    "isSigner": false;
                },
                {
                    "name": "systemProgram";
                    "isMut": false;
                    "isSigner": false;
                }
            ];
            "args": [
                {
                    "name": "insuredAmount";
                    "type": "u64";
                },
                {
                    "name": "endTs";
                    "type": "i64";
                }
            ];
        },
        {
            "name": "initializeTick";
            "accounts": [
                {
                    "name": "creator";
                    "isMut": true;
                    "isSigner": true;
                },
                {
                    "name": "tickAccount";
                    "isMut": true;
                    "isSigner": false;
                },
                {
                    "name": "systemProgram";
                    "isMut": false;
                    "isSigner": false;
                }
            ];
            "args": [
                {
                    "name": "pool";
                    "type": "publicKey";
                },
                {
                    "name": "token";
                    "type": "publicKey";
                },
                {
                    "name": "tickBp";
                    "type": "u16";
                }
            ];
        },
        {
            "name": "closeTick";
            "accounts": [
                {
                    "name": "recipient";
                    "isMut": false;
                    "isSigner": false;
                },
                {
                    "name": "tickAccount";
                    "isMut": true;
                    "isSigner": false;
                }
            ];
            "args": [];
        }
    ];
    "accounts": [
        {
            "name": "bitMap";
            "type": {
                "kind": "struct";
                "fields": [
                    {
                        "name": "bump";
                        "type": "u8";
                    },
                    {
                        "name": "wordPos";
                        "type": "i16";
                    },
                    {
                        "name": "spacing";
                        "type": "u16";
                    },
                    {
                        "name": "word";
                        "type": {
                            "array": [
                                "u64",
                                4
                            ];
                        };
                    }
                ];
            };
        },
        {
            "name": "poolInsuranceContract";
            "type": {
                "kind": "struct";
                "fields": [
                    {
                        "name": "bump";
                        "type": "u8";
                    },
                    {
                        "name": "endTime";
                        "type": "i64";
                    },
                    {
                        "name": "owner";
                        "type": "publicKey";
                    },
                    {
                        "name": "insuranceContracts";
                        "type": "publicKey";
                    }
                ];
            };
        },
        {
            "name": "insuranceContract";
            "type": {
                "kind": "struct";
                "fields": [
                    {
                        "name": "bump";
                        "type": "u8";
                    },
                    {
                        "name": "insuredAmount";
                        "type": "u64";
                    },
                    {
                        "name": "timeLockedInsuredAmount";
                        "type": "u64";
                    },
                    {
                        "name": "premium";
                        "type": "u64";
                    },
                    {
                        "name": "periodTs";
                        "type": "i64";
                    },
                    {
                        "name": "endTs";
                        "type": "i64";
                    },
                    {
                        "name": "startTs";
                        "type": "i64";
                    },
                    {
                        "name": "timeLockEnd";
                        "type": "i64";
                    },
                    {
                        "name": "pool";
                        "type": "publicKey";
                    },
                    {
                        "name": "tickAccount";
                        "type": "publicKey";
                    },
                    {
                        "name": "tokenMint";
                        "type": "publicKey";
                    },
                    {
                        "name": "owner";
                        "type": "publicKey";
                    },
                    {
                        "name": "active";
                        "type": "bool";
                    },
                    {
                        "name": "updatedTs";
                        "type": "i64";
                    },
                    {
                        "name": "createdTs";
                        "type": "i64";
                    }
                ];
            };
        },
        {
            "name": "liquidityPosition";
            "type": {
                "kind": "struct";
                "fields": [
                    {
                        "name": "bump";
                        "type": "u8";
                    },
                    {
                        "name": "liquidity";
                        "type": "u64";
                    },
                    {
                        "name": "usedLiquidity";
                        "type": "u64";
                    },
                    {
                        "name": "pool";
                        "type": "publicKey";
                    },
                    {
                        "name": "tokenMint";
                        "type": "publicKey";
                    },
                    {
                        "name": "nftAccount";
                        "type": "publicKey";
                    },
                    {
                        "name": "nftMint";
                        "type": "publicKey";
                    },
                    {
                        "name": "createdAt";
                        "type": "i64";
                    },
                    {
                        "name": "tickId";
                        "type": "u8";
                    },
                    {
                        "name": "tick";
                        "type": "u16";
                    },
                    {
                        "name": "outstandingRewards";
                        "type": "u32";
                    }
                ];
            };
        },
        {
            "name": "protocolOwner";
            "type": {
                "kind": "struct";
                "fields": [
                    {
                        "name": "bump";
                        "type": "u8";
                    },
                    {
                        "name": "owner";
                        "type": "publicKey";
                    }
                ];
            };
        },
        {
            "name": "poolManager";
            "type": {
                "kind": "struct";
                "fields": [
                    {
                        "name": "owner";
                        "type": "publicKey";
                    },
                    {
                        "name": "bump";
                        "type": "u8";
                    }
                ];
            };
        },
        {
            "name": "surePools";
            "type": {
                "kind": "struct";
                "fields": [
                    {
                        "name": "bump";
                        "type": "u8";
                    },
                    {
                        "name": "pools";
                        "type": {
                            "vec": "publicKey";
                        };
                    }
                ];
            };
        },
        {
            "name": "poolAccount";
            "type": {
                "kind": "struct";
                "fields": [
                    {
                        "name": "bump";
                        "type": "u8";
                    },
                    {
                        "name": "name";
                        "type": "string";
                    },
                    {
                        "name": "insuranceFee";
                        "type": "u16";
                    },
                    {
                        "name": "tickSpacing";
                        "type": "u16";
                    },
                    {
                        "name": "liquidity";
                        "type": "u64";
                    },
                    {
                        "name": "usedLiquidity";
                        "type": "u64";
                    },
                    {
                        "name": "bitmap";
                        "type": "publicKey";
                    },
                    {
                        "name": "premiumRate";
                        "type": "u64";
                    },
                    {
                        "name": "smartContract";
                        "type": "publicKey";
                    },
                    {
                        "name": "locked";
                        "type": "bool";
                    }
                ];
            };
        },
        {
            "name": "tick";
            "type": {
                "kind": "struct";
                "fields": [
                    {
                        "name": "bump";
                        "type": "u8";
                    },
                    {
                        "name": "liquidity";
                        "type": "u64";
                    },
                    {
                        "name": "usedLiquidity";
                        "type": "u64";
                    },
                    {
                        "name": "tokenMint";
                        "type": "publicKey";
                    },
                    {
                        "name": "lastUpdated";
                        "type": "i64";
                    },
                    {
                        "name": "tick";
                        "type": "u16";
                    },
                    {
                        "name": "active";
                        "type": "bool";
                    },
                    {
                        "name": "liquidityPositionId";
                        "type": {
                            "array": [
                                "u8",
                                255
                            ];
                        };
                    },
                    {
                        "name": "liquidityPositionAccumulated";
                        "type": {
                            "array": [
                                "u64",
                                255
                            ];
                        };
                    },
                    {
                        "name": "liquidityPositionRewards";
                        "type": {
                            "array": [
                                "u64",
                                255
                            ];
                        };
                    },
                    {
                        "name": "lastLiquidityPositionIdx";
                        "type": "u8";
                    }
                ];
            };
        }
    ];
    "types": [
        {
            "name": "TickError";
            "type": {
                "kind": "enum";
                "variants": [
                    {
                        "name": "NotEnoughLiquidity";
                    },
                    {
                        "name": "CannotExitInsurancePosition";
                    },
                    {
                        "name": "NoMoreLiquiditySpots";
                    },
                    {
                        "name": "RewardsMustBeWithdrawn";
                    },
                    {
                        "name": "LiquidityPositionInUse";
                    },
                    {
                        "name": "CouldNotGetTickTimestamp";
                    }
                ];
            };
        },
        {
            "name": "SureError";
            "type": {
                "kind": "enum";
                "variants": [
                    {
                        "name": "InvalidMint";
                    },
                    {
                        "name": "InvalidRangeSize";
                    },
                    {
                        "name": "InvalidTick";
                    },
                    {
                        "name": "InvalidAmount";
                    },
                    {
                        "name": "LiquidityFilled";
                    },
                    {
                        "name": "InvalidPoolCreator";
                    },
                    {
                        "name": "CouldNotProvideLiquidity";
                    },
                    {
                        "name": "TickAccountNotEmpty";
                    },
                    {
                        "name": "InvalidTimestamp";
                    },
                    {
                        "name": "InsuranceContractExpired";
                    },
                    {
                        "name": "InsuranceContractIsNotActive";
                    }
                ];
            };
        }
    ];
    "events": [
        {
            "name": "ReduceInsuredAmountForTick";
            "fields": [
                {
                    "name": "owner";
                    "type": "publicKey";
                    "index": false;
                },
                {
                    "name": "tick";
                    "type": "u16";
                    "index": false;
                },
                {
                    "name": "updatedInsuredAmount";
                    "type": "u64";
                    "index": false;
                }
            ];
        },
        {
            "name": "NewLiquidityPosition";
            "fields": [
                {
                    "name": "tick";
                    "type": "u16";
                    "index": false;
                },
                {
                    "name": "liquidity";
                    "type": "u64";
                    "index": false;
                }
            ];
        },
        {
            "name": "ChangeProtocolOwner";
            "fields": [
                {
                    "name": "owner";
                    "type": "publicKey";
                    "index": false;
                },
                {
                    "name": "oldOwner";
                    "type": "publicKey";
                    "index": false;
                }
            ];
        },
        {
            "name": "InitializedPool";
            "fields": [
                {
                    "name": "name";
                    "type": "string";
                    "index": true;
                },
                {
                    "name": "smartContract";
                    "type": "publicKey";
                    "index": false;
                }
            ];
        },
        {
            "name": "CreatePoolVaults";
            "fields": [];
        },
        {
            "name": "InitializedManager";
            "fields": [
                {
                    "name": "owner";
                    "type": "publicKey";
                    "index": true;
                }
            ];
        }
    ];
    "errors": [
        {
            "code": 6000;
            "name": "CouldNotUpdateTimestamp";
            "msg": "could not update timestamp";
        }
    ];
};
export declare const IDL: SurePool;
//# sourceMappingURL=sure_pool.d.ts.map