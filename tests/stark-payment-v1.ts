import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { SplitMethod } from "../target/types/split_method";
import { createMint, createAccount, getOrCreateAssociatedTokenAccount, mintTo, TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { assert } from "chai";


describe("split_method", async () => {

  const connection = new anchor.web3.Connection("http://127.0.0.1:8899", "confirmed");

  const wallet = anchor.web3.Keypair.generate();
  await connection.requestAirdrop(wallet.publicKey, 1e9);

  const provider = new anchor.AnchorProvider(connection, new anchor.Wallet(wallet), {
    commitment: "confirmed",
  });

  anchor.setProvider(provider);

const program = anchor.workspace.MyProgram as anchor.Program<SplitMethod>;

  it("splits token amounts correctly", async () => {

    const payer = provider.wallet; 
    const mint = await createMint(
      provider.connection,
      payer.payer, 
      payer.publicKey, 
      null, 
      6 
    );

    const payerTokenAcct = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      payer.payer,
      mint,
      payer.publicKey
    );

    await mintTo(
      provider.connection,
      payer.payer,
      mint,
      payerTokenAcct.address,
      payer.publicKey,
      1000 
    );

    const receiver1 = anchor.web3.Keypair.generate();
    const receiver2 = anchor.web3.Keypair.generate();

    await provider.connection.requestAirdrop(receiver1.publicKey, 1e9);
    await provider.connection.requestAirdrop(receiver2.publicKey, 1e9);

    const receiverAcct1 = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      payer.payer,
      mint,
      receiver1.publicKey
    );
    const receiverAcct2 = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      payer.payer,
      mint,
      receiver2.publicKey
    );

    const amounts = [300, 700];
    const tx = await program.methods
      .split(amounts)
      .accounts({
        payer: payer.publicKey,
        payerTokenAccount: payerTokenAcct.address,
        mint: mint,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .remainingAccounts([
        { pubkey: receiverAcct1.address, isWritable: true, isSigner: false },
        { pubkey: receiverAcct2.address, isWritable: true, isSigner: false },
      ])
      .rpc();

    console.log("Split tx:", tx);

    const acct1 = await program.provider.connection.getTokenAccountBalance(receiverAcct1.address);
    const acct2 = await program.provider.connection.getTokenAccountBalance(receiverAcct2.address);
    const payerBalance = await program.provider.connection.getTokenAccountBalance(payerTokenAcct.address);

    assert.strictEqual(acct1.value.uiAmount, 300);
    assert.strictEqual(acct2.value.uiAmount, 700);
    assert.strictEqual(payerBalance.value.uiAmount, 0);
  });
});