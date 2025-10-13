import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { StarkPaySplit } from "../target/types/stark_pay_split";
import {
  createMint,
  getOrCreateAssociatedTokenAccount,
  mintTo,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import { assert } from "chai";
import { BN } from "bn.js";

describe("stark_pay_split", () => {
  // Setup
  const connection = new anchor.web3.Connection("http://127.0.0.1:8899", "confirmed");
  const wallet = anchor.web3.Keypair.generate();

  let provider: anchor.AnchorProvider;
  let program: anchor.Program<StarkPaySplit>;
  let mint: anchor.web3.PublicKey;
  let payerTokenAcct: any;

  before(async () => {
    // Airdrop SOL to the wallet for fees
    const sig = await connection.requestAirdrop(wallet.publicKey, 2e9);
    await connection.confirmTransaction({ signature: sig, ...(await connection.getLatestBlockhash()) });

    provider = new anchor.AnchorProvider(
      connection,
      new anchor.Wallet(wallet),
      { commitment: "confirmed" }
    );
    anchor.setProvider(provider);
    program = anchor.workspace.StarkPaySplit as Program<StarkPaySplit>;

    // Create a mint (6 decimals)
    mint = await createMint(connection, wallet, wallet.publicKey, null, 6);

    // Create token account for the payer and mint tokens
    payerTokenAcct = await getOrCreateAssociatedTokenAccount(connection, wallet, mint, wallet.publicKey);
    await mintTo(connection, wallet, mint, payerTokenAcct.address, wallet.publicKey, 1_000_000_000); // 1000 tokens
  });

  it("splits token amounts correctly", async function () {
    this.timeout(60000);

    const receiver1 = anchor.web3.Keypair.generate();
    const receiver2 = anchor.web3.Keypair.generate();

    await Promise.all([
      connection.requestAirdrop(receiver1.publicKey, 1e9),
      connection.requestAirdrop(receiver2.publicKey, 1e9),
    ]);

    const receiverAcct1 = await getOrCreateAssociatedTokenAccount(connection, wallet, mint, receiver1.publicKey);
    const receiverAcct2 = await getOrCreateAssociatedTokenAccount(connection, wallet, mint, receiver2.publicKey);

    const amounts = [new BN(300_000_000), new BN(700_000_000)]; // 300 + 700 = 1000 tokens

    const tx = await program.methods
      .split(amounts)
      .accounts({
        payer: wallet.publicKey,
        payerTokenAccount: payerTokenAcct.address,
        mint,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .remainingAccounts([
        { pubkey: receiverAcct1.address, isWritable: true, isSigner: false },
        { pubkey: receiverAcct2.address, isWritable: true, isSigner: false },
      ])
      .signers([wallet])
      .rpc();

    console.log("Split tx:", tx);

    const acct1 = await connection.getTokenAccountBalance(receiverAcct1.address);
    const acct2 = await connection.getTokenAccountBalance(receiverAcct2.address);
    const payerBalance = await connection.getTokenAccountBalance(payerTokenAcct.address);

    assert.strictEqual(Number(acct1.value.amount), 300_000_000);
    assert.strictEqual(Number(acct2.value.amount), 700_000_000);
    assert.strictEqual(Number(payerBalance.value.amount), 0);
  });

  it("splits token amounts by percentage", async function () {
    this.timeout(60000);

    // Refill payer with tokens for the test
    await mintTo(connection, wallet, mint, payerTokenAcct.address, wallet.publicKey, 1_000_000_000);

    const receiver1 = anchor.web3.Keypair.generate();
    const receiver2 = anchor.web3.Keypair.generate();

    await Promise.all([
      connection.requestAirdrop(receiver1.publicKey, 1e9),
      connection.requestAirdrop(receiver2.publicKey, 1e9),
    ]);

    const receiverAcct1 = await getOrCreateAssociatedTokenAccount(connection, wallet, mint, receiver1.publicKey);
    const receiverAcct2 = await getOrCreateAssociatedTokenAccount(connection, wallet, mint, receiver2.publicKey);

    const amount = new BN(1_000_000_000);
    const percentages = [new BN(25), new BN(75)];

    const tx = await program.methods
      .splitPercentage(amount, percentages)
      .accounts({
        payer: wallet.publicKey,
        payerTokenAccount: payerTokenAcct.address,
        mint,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .remainingAccounts([
        { pubkey: receiverAcct1.address, isWritable: true, isSigner: false },
        { pubkey: receiverAcct2.address, isWritable: true, isSigner: false },
      ])
      .signers([wallet])
      .rpc();

    console.log("Split by percentage tx:", tx);

    const acct1 = await connection.getTokenAccountBalance(receiverAcct1.address);
    const acct2 = await connection.getTokenAccountBalance(receiverAcct2.address);
    const payerBalance = await connection.getTokenAccountBalance(payerTokenAcct.address);

    assert.strictEqual(Number(acct1.value.amount), (1_000_000_000 * 25) / 100);
    assert.strictEqual(Number(acct2.value.amount), (1_000_000_000 * 75) / 100);
    assert.strictEqual(Number(payerBalance.value.amount), 0);
  });
});
