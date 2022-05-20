import { SurePool } from "../../target/types/sure_pool";
import {PublicKey,Connection} from "@solana/web3.js"

import { Program } from "@project-serum/anchor";
import * as anchor from "@project-serum/anchor"
const program = anchor.workspace.SurePool as Program<SurePool>

export type BitmapType = {
    bump: number,
    wordPos: number,
    spacing: number, 
    word: (anchor.BN)[]
}


export class Bitmap {
    protected bump: number;
    protected wordPos: number;
    protected spacing: number;
    protected word: anchor.BN[];


    constructor(
        bump: number,
        wordPos: number,
        spacing: number,
        word: (anchor.BN)[]
    ){
      this.bump=bump;
      this.wordPos = wordPos;
      this.spacing = spacing;
      this.word = word;
    }

    static new(bitmap: BitmapType): Bitmap{
        return new Bitmap(bitmap.bump,bitmap.wordPos,bitmap.spacing,bitmap.word)
    }

    getLowestBit(): number{
        const u256 = this.word.flatMap((word) => {
            return word.toString(2,64).split("").reverse().join("")
        })[0]
       
        return u256.indexOf("1")
    }

    getHighestBit(): number {
        const u256 = this.word.flatMap((word) => {
            return word.toString(2,64).split("").reverse().join("")
        })[0]

        return u256.lastIndexOf("1")
    }

    getTickFromBit(bit: number): number {
        return 0 + this.spacing*bit
    } 

    getBitFromTick(tick: number): number {
        return tick/this.spacing
    }

    getNextTick(tick: number): number {
        const bit = this.getBitFromTick(tick);
    
        const u256 = this.word.flatMap((word) => {
            return word.toString(2,64).split("").reverse().join("")
        })[0]
    
        const remainingBitmap = u256.slice(bit+1)
        const subBit = remainingBitmap.indexOf("1")
        if (subBit === -1){
            return -1
        }
        const nextBit = subBit + bit+1
    
        return this.getTickFromBit(nextBit)
    }

    getPreviousTick(tick:number):number {
        const bit = this.getBitFromTick(tick);
    
        const u256 = this.word.flatMap((word) => {
            return word.toString(2,64).split("").reverse().join("")
        })[0]

        const priorBitmap = u256.slice(0,tick-1)
        const lastBit = priorBitmap.lastIndexOf("1")
        if (lastBit === -1){
            return lastBit
        }

        return this.getTickFromBit(lastBit)

    }
}


/**
     * Current tick position in tick pool
     * get the lowest tick pool with available liquidity
     *
     * @param poolPDA PDA for pool 
     * @param tokenMint The mint of the token to be supplied to the pool. This could be USDC
     * @return lowest bit
     */
 export const getLowestBit = async (bitmapPDA: PublicKey): Promise<number> => {
    const bitmap =await program.account.bitMap.fetch(bitmapPDA)

    const u256 = bitmap.word.flatMap((word) => {
        return word.toString(2,64).split("").reverse().join("")
    })[0]
    const firstBit = u256.indexOf("1")
   
    return firstBit
}

export const getTickBasisPoint = (bitPosition: number, tickSpacing: number): number => {
    return 0 + tickSpacing*bitPosition
}

export const getBitFromTick = (tick: number, tickSpacing: number): number => {
    return tick/tickSpacing
}

/**
     * Get the next tick position in tick pool
     * NOT USED!!!!
     * 
     * @param poolPDA PDA for pool 
     * @param tokenMint The mint of the token to be supplied to the pool. This could be USDC
     * @return Next tick position
     */
export const getNextBit = async (bitmapPDA: PublicKey,prevBit:number): Promise<number> => {
    const bitmap =await program.account.bitMap.fetch(bitmapPDA)

    const u256 = bitmap.word.flatMap((word) => {
        return word.toString(2,64).split("").reverse().join("")
    })[0]

    return 0
}

export const getNextTick = async (prevTick: number,bitmapPDA: PublicKey,tickSpacing: number ): Promise<number> => {
    const prevBit = getBitFromTick(prevTick,tickSpacing);
    const bitmap =await program.account.bitMap.fetch(bitmapPDA)

    const u256 = bitmap.word.flatMap((word) => {
        return word.toString(2,64).split("").reverse().join("")
    })[0]

    //console.log("u256: ",u256)
    const remainingBitmap = u256.slice(prevBit+1)
    const subBit = remainingBitmap.indexOf("1")
    if (subBit === -1){
        return -1
    }
    const nextBit = subBit + prevBit+1

    return getTickBasisPoint(nextBit,tickSpacing)

}