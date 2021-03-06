export type SurePool = {
  "version": "0.0.1",
  "name": "sure_pool",
  "instructions": [
    {
      "name": "initializeProtocol",
      "accounts": [
        {
          "name": "owner",
          "isMut": true,
          "isSigner": true
        },
        {
          "name": "protocolOwner",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "pools",
          "isMut": true,
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
      "name": "initializePoolManager",
      "accounts": [
        {
          "name": "initialManager",
          "isMut": true,
          "isSigner": true
        },
        {
          "name": "manager",
          "isMut": true,
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
      "name": "initializePolicyHolder",
      "accounts": [
        {
          "name": "signer",
          "isMut": true,
          "isSigner": true
        },
        {
          "name": "insuranceContracts",
          "isMut": true,
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
      "name": "createPool",
      "accounts": [
        {
          "name": "poolCreator",
          "isMut": true,
          "isSigner": true
        },
        {
          "name": "protocolOwner",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "pool",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "pools",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "smartContract",
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
          "name": "insuranceFee",
          "type": "u16"
        },
        {
          "name": "name",
          "type": "string"
        }
      ]
    },
    {
      "name": "initializeTokenPool",
      "accounts": [
        {
          "name": "creator",
          "isMut": true,
          "isSigner": true
        },
        {
          "name": "pool",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "poolVaultTokenMint",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "poolVault",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "premiumVault",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "poolLiquidityTickBitmap",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "tokenPool",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "rent",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "tokenProgram",
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
      "name": "depositLiquidity",
      "accounts": [
        {
          "name": "liquidityProvider",
          "isMut": true,
          "isSigner": true
        },
        {
          "name": "protocolOwner",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "liquidityProviderAta",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "pool",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "tokenPool",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "poolVault",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "metadataAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "metadataProgram",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "liquidityPosition",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "liquidityPositionNftMint",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "liquidityPositionNftAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "poolLiquidityTickBitmap",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "liquidityTickInfo",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "rent",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "systemProgram",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "associatedTokenProgram",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "tick",
          "type": "u16"
        },
        {
          "name": "tickPos",
          "type": "u64"
        },
        {
          "name": "amount",
          "type": "u64"
        }
      ]
    },
    {
      "name": "redeemLiquidity",
      "accounts": [
        {
          "name": "nftHolder",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "pool",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "tokenPool",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "liquidityPositionNftAccount",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "protocolOwner",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "liquidityPosition",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "liquidityProviderAta",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "poolVault",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "poolLiquidityTickBitmap",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "liquidityTickInfo",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "metadataAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "metadataProgram",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "tokenProgram",
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
      "name": "initializeUserPoolInsuranceContract",
      "accounts": [
        {
          "name": "signer",
          "isMut": true,
          "isSigner": true
        },
        {
          "name": "pool",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "tokenMint",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "insuranceContracts",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "poolInsuranceContractBitmap",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "poolInsuranceContractInfo",
          "isMut": true,
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
      "name": "initializeInsuranceContract",
      "accounts": [
        {
          "name": "owner",
          "isMut": true,
          "isSigner": true
        },
        {
          "name": "pool",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "tokenMint",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "liquidityTickInfo",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "insuranceTickContract",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "poolInsuranceContractInfo",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "poolInsuranceContractBitmap",
          "isMut": true,
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
      "name": "updateInsuranceTickContract",
      "accounts": [
        {
          "name": "buyer",
          "isMut": true,
          "isSigner": true
        },
        {
          "name": "tokenAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "pool",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "tokenPool",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "liquidityTickInfo",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "liquidityTickBitmap",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "premiumVault",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "insuranceTickContract",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "poolInsuranceContractInfo",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "tokenProgram",
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
          "name": "newInsuredAmountOnTick",
          "type": "u64"
        },
        {
          "name": "newExpiryTs",
          "type": "i64"
        }
      ]
    },
    {
      "name": "initializePoolLiquidityTick",
      "accounts": [
        {
          "name": "creator",
          "isMut": true,
          "isSigner": true
        },
        {
          "name": "liquidityTickInfo",
          "isMut": true,
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
          "name": "pool",
          "type": "publicKey"
        },
        {
          "name": "token",
          "type": "publicKey"
        },
        {
          "name": "tickBp",
          "type": "u16"
        }
      ]
    },
    {
      "name": "closePoolLiquidityTick",
      "accounts": [
        {
          "name": "recipient",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "liquidityTickInfo",
          "isMut": true,
          "isSigner": false
        }
      ],
      "args": []
    }
  ],
  "accounts": [
    {
      "name": "bitMap",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "bump",
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
      "name": "insuranceContracts",
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
            "name": "pools",
            "type": {
              "vec": "publicKey"
            }
          }
        ]
      }
    },
    {
      "name": "poolInsuranceContract",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "bump",
            "type": "u8"
          },
          {
            "name": "expiryTs",
            "type": "i64"
          },
          {
            "name": "insuredAmount",
            "type": "u64"
          },
          {
            "name": "tokenMint",
            "type": "publicKey"
          },
          {
            "name": "owner",
            "type": "publicKey"
          }
        ]
      }
    },
    {
      "name": "insuranceTickContract",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "bump",
            "type": "u8"
          },
          {
            "name": "insuredAmount",
            "type": "u64"
          },
          {
            "name": "premium",
            "type": "u64"
          },
          {
            "name": "endTs",
            "type": "i64"
          },
          {
            "name": "startTs",
            "type": "i64"
          },
          {
            "name": "pool",
            "type": "publicKey"
          },
          {
            "name": "liquidityTickInfo",
            "type": "publicKey"
          },
          {
            "name": "tokenMint",
            "type": "publicKey"
          },
          {
            "name": "active",
            "type": "bool"
          },
          {
            "name": "updatedTs",
            "type": "i64"
          },
          {
            "name": "createdTs",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "liquidityPosition",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "bump",
            "type": "u8"
          },
          {
            "name": "liquidity",
            "type": "u64"
          },
          {
            "name": "usedLiquidity",
            "type": "u64"
          },
          {
            "name": "pool",
            "type": "publicKey"
          },
          {
            "name": "tokenMint",
            "type": "publicKey"
          },
          {
            "name": "nftAccount",
            "type": "publicKey"
          },
          {
            "name": "nftMint",
            "type": "publicKey"
          },
          {
            "name": "createdAt",
            "type": "i64"
          },
          {
            "name": "tickId",
            "type": "u8"
          },
          {
            "name": "tick",
            "type": "u16"
          },
          {
            "name": "outstandingRewards",
            "type": "u32"
          }
        ]
      }
    },
    {
      "name": "protocolOwner",
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
          }
        ]
      }
    },
    {
      "name": "poolManager",
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
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "bump",
            "type": "u8"
          },
          {
            "name": "pools",
            "type": {
              "vec": "publicKey"
            }
          }
        ]
      }
    },
    {
      "name": "poolAccount",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "bump",
            "type": "u8"
          },
          {
            "name": "name",
            "type": "string"
          },
          {
            "name": "insuranceFee",
            "type": "u16"
          },
          {
            "name": "smartContract",
            "type": "publicKey"
          },
          {
            "name": "tokenPools",
            "type": {
              "vec": "publicKey"
            }
          },
          {
            "name": "locked",
            "type": "bool"
          }
        ]
      }
    },
    {
      "name": "tokenPool",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "bump",
            "type": "u8"
          },
          {
            "name": "tokenMint",
            "type": "publicKey"
          },
          {
            "name": "pool",
            "type": "publicKey"
          },
          {
            "name": "liquidity",
            "type": "u64"
          },
          {
            "name": "usedLiquidity",
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "tick",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "bump",
            "type": "u8"
          },
          {
            "name": "liquidity",
            "type": "u64"
          },
          {
            "name": "usedLiquidity",
            "type": "u64"
          },
          {
            "name": "tokenMint",
            "type": "publicKey"
          },
          {
            "name": "lastUpdated",
            "type": "i64"
          },
          {
            "name": "tick",
            "type": "u16"
          },
          {
            "name": "active",
            "type": "bool"
          },
          {
            "name": "liquidityPositionId",
            "type": {
              "array": [
                "u8",
                255
              ]
            }
          },
          {
            "name": "liquidityPositionAccumulated",
            "type": {
              "array": [
                "u64",
                255
              ]
            }
          },
          {
            "name": "liquidityPositionRewards",
            "type": {
              "array": [
                "u64",
                255
              ]
            }
          },
          {
            "name": "lastLiquidityPositionIdx",
            "type": "u8"
          }
        ]
      }
    }
  ],
  "types": [
    {
      "name": "TickError",
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "NotEnoughLiquidity"
          },
          {
            "name": "CannotExitInsurancePosition"
          },
          {
            "name": "NoMoreLiquiditySpots"
          },
          {
            "name": "RewardsMustBeWithdrawn"
          },
          {
            "name": "LiquidityPositionInUse"
          },
          {
            "name": "CouldNotGetTickTimestamp"
          }
        ]
      }
    },
    {
      "name": "SureError",
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "InvalidMint"
          },
          {
            "name": "InvalidRangeSize"
          },
          {
            "name": "InvalidTick"
          },
          {
            "name": "InvalidAmount"
          },
          {
            "name": "LiquidityFilled"
          },
          {
            "name": "InvalidPoolCreator"
          },
          {
            "name": "CouldNotProvideLiquidity"
          },
          {
            "name": "TickAccountNotEmpty"
          },
          {
            "name": "InvalidTimestamp"
          },
          {
            "name": "InsuranceContractExpired"
          },
          {
            "name": "InsuranceContractIsNotActive"
          }
        ]
      }
    }
  ],
  "events": [
    {
      "name": "ReduceInsuredAmountForTick",
      "fields": [
        {
          "name": "owner",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "tick",
          "type": "u16",
          "index": false
        },
        {
          "name": "updatedInsuredAmount",
          "type": "u64",
          "index": false
        }
      ]
    },
    {
      "name": "InitializePolicyHolderEvent",
      "fields": [
        {
          "name": "owner",
          "type": "publicKey",
          "index": false
        }
      ]
    },
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
      "name": "CouldNotUpdateTimestamp",
      "msg": "could not update timestamp"
    }
  ]
};

export const IDL: SurePool = {
  "version": "0.0.1",
  "name": "sure_pool",
  "instructions": [
    {
      "name": "initializeProtocol",
      "accounts": [
        {
          "name": "owner",
          "isMut": true,
          "isSigner": true
        },
        {
          "name": "protocolOwner",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "pools",
          "isMut": true,
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
      "name": "initializePoolManager",
      "accounts": [
        {
          "name": "initialManager",
          "isMut": true,
          "isSigner": true
        },
        {
          "name": "manager",
          "isMut": true,
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
      "name": "initializePolicyHolder",
      "accounts": [
        {
          "name": "signer",
          "isMut": true,
          "isSigner": true
        },
        {
          "name": "insuranceContracts",
          "isMut": true,
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
      "name": "createPool",
      "accounts": [
        {
          "name": "poolCreator",
          "isMut": true,
          "isSigner": true
        },
        {
          "name": "protocolOwner",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "pool",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "pools",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "smartContract",
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
          "name": "insuranceFee",
          "type": "u16"
        },
        {
          "name": "name",
          "type": "string"
        }
      ]
    },
    {
      "name": "initializeTokenPool",
      "accounts": [
        {
          "name": "creator",
          "isMut": true,
          "isSigner": true
        },
        {
          "name": "pool",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "poolVaultTokenMint",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "poolVault",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "premiumVault",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "poolLiquidityTickBitmap",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "tokenPool",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "rent",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "tokenProgram",
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
      "name": "depositLiquidity",
      "accounts": [
        {
          "name": "liquidityProvider",
          "isMut": true,
          "isSigner": true
        },
        {
          "name": "protocolOwner",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "liquidityProviderAta",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "pool",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "tokenPool",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "poolVault",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "metadataAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "metadataProgram",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "liquidityPosition",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "liquidityPositionNftMint",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "liquidityPositionNftAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "poolLiquidityTickBitmap",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "liquidityTickInfo",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "rent",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "systemProgram",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "associatedTokenProgram",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "tick",
          "type": "u16"
        },
        {
          "name": "tickPos",
          "type": "u64"
        },
        {
          "name": "amount",
          "type": "u64"
        }
      ]
    },
    {
      "name": "redeemLiquidity",
      "accounts": [
        {
          "name": "nftHolder",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "pool",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "tokenPool",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "liquidityPositionNftAccount",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "protocolOwner",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "liquidityPosition",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "liquidityProviderAta",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "poolVault",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "poolLiquidityTickBitmap",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "liquidityTickInfo",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "metadataAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "metadataProgram",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "tokenProgram",
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
      "name": "initializeUserPoolInsuranceContract",
      "accounts": [
        {
          "name": "signer",
          "isMut": true,
          "isSigner": true
        },
        {
          "name": "pool",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "tokenMint",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "insuranceContracts",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "poolInsuranceContractBitmap",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "poolInsuranceContractInfo",
          "isMut": true,
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
      "name": "initializeInsuranceContract",
      "accounts": [
        {
          "name": "owner",
          "isMut": true,
          "isSigner": true
        },
        {
          "name": "pool",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "tokenMint",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "liquidityTickInfo",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "insuranceTickContract",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "poolInsuranceContractInfo",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "poolInsuranceContractBitmap",
          "isMut": true,
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
      "name": "updateInsuranceTickContract",
      "accounts": [
        {
          "name": "buyer",
          "isMut": true,
          "isSigner": true
        },
        {
          "name": "tokenAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "pool",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "tokenPool",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "liquidityTickInfo",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "liquidityTickBitmap",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "premiumVault",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "insuranceTickContract",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "poolInsuranceContractInfo",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "tokenProgram",
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
          "name": "newInsuredAmountOnTick",
          "type": "u64"
        },
        {
          "name": "newExpiryTs",
          "type": "i64"
        }
      ]
    },
    {
      "name": "initializePoolLiquidityTick",
      "accounts": [
        {
          "name": "creator",
          "isMut": true,
          "isSigner": true
        },
        {
          "name": "liquidityTickInfo",
          "isMut": true,
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
          "name": "pool",
          "type": "publicKey"
        },
        {
          "name": "token",
          "type": "publicKey"
        },
        {
          "name": "tickBp",
          "type": "u16"
        }
      ]
    },
    {
      "name": "closePoolLiquidityTick",
      "accounts": [
        {
          "name": "recipient",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "liquidityTickInfo",
          "isMut": true,
          "isSigner": false
        }
      ],
      "args": []
    }
  ],
  "accounts": [
    {
      "name": "bitMap",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "bump",
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
      "name": "insuranceContracts",
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
            "name": "pools",
            "type": {
              "vec": "publicKey"
            }
          }
        ]
      }
    },
    {
      "name": "poolInsuranceContract",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "bump",
            "type": "u8"
          },
          {
            "name": "expiryTs",
            "type": "i64"
          },
          {
            "name": "insuredAmount",
            "type": "u64"
          },
          {
            "name": "tokenMint",
            "type": "publicKey"
          },
          {
            "name": "owner",
            "type": "publicKey"
          }
        ]
      }
    },
    {
      "name": "insuranceTickContract",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "bump",
            "type": "u8"
          },
          {
            "name": "insuredAmount",
            "type": "u64"
          },
          {
            "name": "premium",
            "type": "u64"
          },
          {
            "name": "endTs",
            "type": "i64"
          },
          {
            "name": "startTs",
            "type": "i64"
          },
          {
            "name": "pool",
            "type": "publicKey"
          },
          {
            "name": "liquidityTickInfo",
            "type": "publicKey"
          },
          {
            "name": "tokenMint",
            "type": "publicKey"
          },
          {
            "name": "active",
            "type": "bool"
          },
          {
            "name": "updatedTs",
            "type": "i64"
          },
          {
            "name": "createdTs",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "liquidityPosition",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "bump",
            "type": "u8"
          },
          {
            "name": "liquidity",
            "type": "u64"
          },
          {
            "name": "usedLiquidity",
            "type": "u64"
          },
          {
            "name": "pool",
            "type": "publicKey"
          },
          {
            "name": "tokenMint",
            "type": "publicKey"
          },
          {
            "name": "nftAccount",
            "type": "publicKey"
          },
          {
            "name": "nftMint",
            "type": "publicKey"
          },
          {
            "name": "createdAt",
            "type": "i64"
          },
          {
            "name": "tickId",
            "type": "u8"
          },
          {
            "name": "tick",
            "type": "u16"
          },
          {
            "name": "outstandingRewards",
            "type": "u32"
          }
        ]
      }
    },
    {
      "name": "protocolOwner",
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
          }
        ]
      }
    },
    {
      "name": "poolManager",
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
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "bump",
            "type": "u8"
          },
          {
            "name": "pools",
            "type": {
              "vec": "publicKey"
            }
          }
        ]
      }
    },
    {
      "name": "poolAccount",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "bump",
            "type": "u8"
          },
          {
            "name": "name",
            "type": "string"
          },
          {
            "name": "insuranceFee",
            "type": "u16"
          },
          {
            "name": "smartContract",
            "type": "publicKey"
          },
          {
            "name": "tokenPools",
            "type": {
              "vec": "publicKey"
            }
          },
          {
            "name": "locked",
            "type": "bool"
          }
        ]
      }
    },
    {
      "name": "tokenPool",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "bump",
            "type": "u8"
          },
          {
            "name": "tokenMint",
            "type": "publicKey"
          },
          {
            "name": "pool",
            "type": "publicKey"
          },
          {
            "name": "liquidity",
            "type": "u64"
          },
          {
            "name": "usedLiquidity",
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "tick",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "bump",
            "type": "u8"
          },
          {
            "name": "liquidity",
            "type": "u64"
          },
          {
            "name": "usedLiquidity",
            "type": "u64"
          },
          {
            "name": "tokenMint",
            "type": "publicKey"
          },
          {
            "name": "lastUpdated",
            "type": "i64"
          },
          {
            "name": "tick",
            "type": "u16"
          },
          {
            "name": "active",
            "type": "bool"
          },
          {
            "name": "liquidityPositionId",
            "type": {
              "array": [
                "u8",
                255
              ]
            }
          },
          {
            "name": "liquidityPositionAccumulated",
            "type": {
              "array": [
                "u64",
                255
              ]
            }
          },
          {
            "name": "liquidityPositionRewards",
            "type": {
              "array": [
                "u64",
                255
              ]
            }
          },
          {
            "name": "lastLiquidityPositionIdx",
            "type": "u8"
          }
        ]
      }
    }
  ],
  "types": [
    {
      "name": "TickError",
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "NotEnoughLiquidity"
          },
          {
            "name": "CannotExitInsurancePosition"
          },
          {
            "name": "NoMoreLiquiditySpots"
          },
          {
            "name": "RewardsMustBeWithdrawn"
          },
          {
            "name": "LiquidityPositionInUse"
          },
          {
            "name": "CouldNotGetTickTimestamp"
          }
        ]
      }
    },
    {
      "name": "SureError",
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "InvalidMint"
          },
          {
            "name": "InvalidRangeSize"
          },
          {
            "name": "InvalidTick"
          },
          {
            "name": "InvalidAmount"
          },
          {
            "name": "LiquidityFilled"
          },
          {
            "name": "InvalidPoolCreator"
          },
          {
            "name": "CouldNotProvideLiquidity"
          },
          {
            "name": "TickAccountNotEmpty"
          },
          {
            "name": "InvalidTimestamp"
          },
          {
            "name": "InsuranceContractExpired"
          },
          {
            "name": "InsuranceContractIsNotActive"
          }
        ]
      }
    }
  ],
  "events": [
    {
      "name": "ReduceInsuredAmountForTick",
      "fields": [
        {
          "name": "owner",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "tick",
          "type": "u16",
          "index": false
        },
        {
          "name": "updatedInsuredAmount",
          "type": "u64",
          "index": false
        }
      ]
    },
    {
      "name": "InitializePolicyHolderEvent",
      "fields": [
        {
          "name": "owner",
          "type": "publicKey",
          "index": false
        }
      ]
    },
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
      "name": "CouldNotUpdateTimestamp",
      "msg": "could not update timestamp"
    }
  ]
};
