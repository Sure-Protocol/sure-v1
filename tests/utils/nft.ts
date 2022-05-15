import {PublicKey,AccountInfo,ParsedAccountData} from "@solana/web3.js"
import {getMint, TOKEN_PROGRAM_ID} from "@solana/spl-token"
import * as anchor from "@project-serum/anchor"
import {TokenAccount} from "./types"
import * as sureSdk from "./index"



export const getSureNfts = async (connection:anchor.web3.Connection, wallet: PublicKey): Promise< Array<TokenAccount>> => {
    // Get all tokens held by wallet 
    const tokensOwnedByWallet = await connection.getParsedTokenAccountsByOwner(
        wallet, { programId: TOKEN_PROGRAM_ID }
    )
    
    const [sureMintAuthority,_] =await  sureSdk.getProtocolOwner()
    const sureNfts = tokensOwnedByWallet.value.filter(async token => {
        if(token.account.data.parsed?.info?.mint){
            const tokenMint = new PublicKey(token.account.data.parsed.info.mint)
            const tokenMintAccount = await getMint(connection,tokenMint)
            return tokenMintAccount.mintAuthority.toBase58() == sureMintAuthority.toBase58()
        }
        return false
    })

    return sureNfts

}