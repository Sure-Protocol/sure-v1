import { SurePool } from "../../target/types/sure_pool";
import {PublicKey} from "@solana/web3.js"
import * as anchor from "@project-serum/anchor"
import { Program } from "@project-serum/anchor";
const {SystemProgram} =anchor.web3;

const program = anchor.workspace.SurePool as Program<SurePool>
export const POOL_SEED =anchor.utils.bytes.utf8.encode("sure-insurance-pool")
export const TOKEN_VAULT_SEED = anchor.utils.bytes.utf8.encode("sure-ata")
export const SURE_BITMAP = anchor.utils.bytes.utf8.encode("sure-bitmap")
export const SURE_LIQUIDITY_POSITION = anchor.utils.bytes.utf8.encode("sure-lp");
export const SURE_TICK_SEED = anchor.utils.bytes.utf8.encode("sure-tick")
export const SURE_NFT_MINT_SEED = anchor.utils.bytes.utf8.encode("sure-nft");
export const SURE_TOKEN_ACCOUNT_SEED = anchor.utils.bytes.utf8.encode("sure-token-account");


export const getPoolPDA = async (smartContractToInsure: PublicKey,token_mint:PublicKey,program: anchor.Program<SurePool>): Promise<[pda: anchor.web3.PublicKey,bump:number]> => {
    const POOL_SEED =anchor.utils.bytes.utf8.encode("sure-insurance-pool")
    return await PublicKey.findProgramAddress(
        [
            POOL_SEED,
            token_mint.toBytes(),
            smartContractToInsure.toBytes()
        ],
        program.programId
    )
}

export const getBitmapPDA = async (poolPDA: PublicKey,token_mint: PublicKey,program: anchor.Program<SurePool>): Promise<[pda: anchor.web3.PublicKey,bump:number]> => {
    return await PublicKey.findProgramAddress(
        [
            SURE_BITMAP,
            poolPDA.toBytes(),
            token_mint.toBytes(),
        ],
        program.programId,
    )
    
}

export const getTickPDA = async (poolPDA: PublicKey,tokenMint: PublicKey,tickBpn:number): Promise<PublicKey> =>{
    let tickBp = new anchor.BN(tickBpn)
    const [tickAccountPDA,tickAccountBump] = await PublicKey.findProgramAddress(
        [
            SURE_TICK_SEED,
            poolPDA.toBytes(),
            tokenMint.toBytes(),
            tickBp.toArrayLike(Buffer,"le",8)
        ],
        program.programId,
    )
    return tickAccountPDA
}

export const getNextTickPosition = async (poolPDA: PublicKey,tokenMint: PublicKey,tickBpn:number): Promise<number> => {
    const tickPDA = await getTickPDA(poolPDA,tokenMint,tickBpn);
    try {
        const tickAccount = await program.account.tick.fetch(tickPDA);
        return tickAccount.lastLiquidityPositionIdx
    }catch(e){
        throw new Error("Tick account does not exist. Cause: "+e)
    }
}

/// Check if tick account exists for the pool, 
/// if not, create the account. 

export const createTickAccount = async (poolPDA: PublicKey,tokenMint: PublicKey,tickBpn: number,creator: PublicKey): Promise<PublicKey> => {
    const tickAccountPDA = await getTickPDA(poolPDA,tokenMint,tickBpn);

   try{
    let tickBp = new anchor.BN(tickBpn)
    await program.rpc.initializeTick(poolPDA,tokenMint,tickBp, {
        accounts: {
            creator:creator,
            tickAccount: tickAccountPDA,
            systemProgram: SystemProgram.programId,
        },
    })
   } catch(e){
       throw new Error("Could not create tick account: " +e)
   }

   return tickAccountPDA
}

export const getOrCreateTickAccount = async (owner: PublicKey,poolPDA: PublicKey, tokenMint: PublicKey, tickBp: number): Promise<anchor.web3.PublicKey> => {
    const [tickAccountPDA,tickAccountBump] = await PublicKey.findProgramAddress(
        [
            SURE_TICK_SEED,
            poolPDA.toBytes(),
            tokenMint.toBytes(),
            anchor.utils.bytes.utf8.encode(tickBp.toString()),
        ],
        program.programId,
    )
    let account;
    try {
        account = await program.account.tick.fetch(tickAccountPDA)
    } catch (e){
        console.log("erere")
        // create account
        try {
            await createTickAccount(poolPDA,tokenMint,tickBp,owner);
        }catch (e){
            throw new Error("could not create tick account. cause: "+e)
        }
       account = await program.account.tick.fetch(tickAccountPDA)
    }   
    return account
}

export const getProtocolOwner = async (): Promise<[PublicKey,number]> => {
   return await PublicKey.findProgramAddress(
        [],
        program.programId,
    )
}