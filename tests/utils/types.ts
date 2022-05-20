import {PublicKey,AccountInfo,ParsedAccountData} from "@solana/web3.js"
import * as anchor from "@project-serum/anchor"
// Account representing an spl-token
export interface TokenAccount{
    pubkey: PublicKey;
    account: AccountInfo<ParsedAccountData>;
}

