import { expect } from 'chai';
import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { SurePool } from "../target/types/sure_pool";

describe("Initialize Sure Pool",() => {
    anchor.setProvider(anchor.Provider.env())

    const program = anchor.workspace.SurePool as Program<SurePool>

    it("create Sure pool manager",async () => {
        const poolManager = anchor.web3.Keypair.generate();
    })
})