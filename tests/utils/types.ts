import {PublicKey,AccountInfo,ParsedAccountData} from "@solana/web3.js"
// Account representing an spl-token
export interface TokenAccount{
    pubkey: PublicKey;
    account: AccountInfo<ParsedAccountData>;
}