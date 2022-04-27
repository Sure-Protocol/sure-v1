import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { SurePool } from "../target/types/sure_pool";

describe("sure-pool", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.Provider.env());

  const program = anchor.workspace.SurePool as Program<SurePool>;

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.rpc.initialize({});
    console.log("Your transaction signature", tx);
  });
});
