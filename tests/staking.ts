import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { PublicKey, SystemProgram, Transaction, TransactionInstruction } from "@solana/web3.js";
import { Staking } from "../target/types/staking";

// Hardcoded Token-2022 and Associated Token-2022 program IDs
const TOKEN_2022_PROGRAM_ID = new PublicKey("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb");
const ASSOCIATED_TOKEN_2022_PROGRAM_ID = new PublicKey("ATokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb");

// Helper: create a Token-2022 token account at a PDA
async function createToken2022AccountAtPDA({
  connection,
  payer,
  newAccountPubkey,
  mint,
  owner,
  programId
}: {
  connection: anchor.web3.Connection,
  payer: PublicKey,
  newAccountPubkey: PublicKey,
  mint: PublicKey,
  owner: PublicKey,
  programId: PublicKey
}) {
  const lamports = await connection.getMinimumBalanceForRentExemption(165);
  // Create the account at the PDA
  const createIx = SystemProgram.createAccount({
    fromPubkey: payer,
    newAccountPubkey,
    lamports,
    space: 165,
    programId,
  });
  // Initialize the token account (raw instruction for Token-2022)
  // Instruction layout: https://github.com/solana-labs/solana-program-library/blob/master/token/program-2022/src/instruction.rs
  // 1 = InitializeAccount2, then mint, owner, rent sysvar
  const data = Buffer.from([1]);
  const keys = [
    { pubkey: newAccountPubkey, isSigner: false, isWritable: true },
    { pubkey: mint, isSigner: false, isWritable: false },
    { pubkey: owner, isSigner: false, isWritable: false },
    { pubkey: anchor.web3.SYSVAR_RENT_PUBKEY, isSigner: false, isWritable: false },
  ];
  const initIx = new TransactionInstruction({
    programId,
    keys,
    data,
  });
  const tx = new Transaction().add(createIx, initIx);
  await connection.sendTransaction(tx, [await anchor.AnchorProvider.env().wallet.payer]);
}

// Helper: get token account balance
async function getTokenAccountBalance(connection: anchor.web3.Connection, pubkey: PublicKey) {
  try {
    const acc = await connection.getTokenAccountBalance(pubkey);
    return acc.value.uiAmountString;
  } catch (e) {
    return "0";
  }
}

// Helper: get stake state
async function getStakeState(program: Program<Staking>, stakeState: PublicKey) {
  try {
    const state = await program.account.stakeState.fetch(stakeState);
    return state;
  } catch (e) {
    return null;
  }
}

describe("staking", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.staking as Program<Staking>;

  // Token-2022 Mint (same as used in the real program)
  const mint = new PublicKey("45qA6AB2EZa3wUfBGwifw31Qt3iajAwnduLrMMjdcakm");

  // User's Token-2022 token account (ATA)
  const userTokenAccount = new PublicKey("9HjDYnpPGEhdQePXwyL3CMsUM5vevXt8vsNTKK7hWZFh");

  // Vault PDA (seeds = [b"vault"])
  const [vaultPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("vault")],
    program.programId
  );

  // PDA for stake state
  let stakeState: PublicKey;

  it("Creates vault PDA token account (Token-2022)", async () => {
    // Check if the account exists
    const accInfo = await provider.connection.getAccountInfo(vaultPda);
    if (!accInfo) {
      await createToken2022AccountAtPDA({
        connection: provider.connection,
        payer: provider.wallet.publicKey,
        newAccountPubkey: vaultPda,
        mint,
        owner: vaultPda,
        programId: TOKEN_2022_PROGRAM_ID,
      });
      console.log("✅ Created vault PDA token account:", vaultPda.toBase58());
    } else {
      console.log("Vault PDA token account already exists:", vaultPda.toBase58());
    }
    console.log("User token balance:", await getTokenAccountBalance(provider.connection, userTokenAccount));
    console.log("Vault token balance:", await getTokenAccountBalance(provider.connection, vaultPda));
  });

  it("Initializes stake", async () => {
    const [pda, bump] = PublicKey.findProgramAddressSync(
      [Buffer.from("stake"), provider.wallet.publicKey.toBuffer()],
      program.programId
    );
    stakeState = pda;

    const tx = await program.methods
      .initializeStake(new anchor.BN(1_000_000_000)) // 1 token (9 decimals)
      .accounts({
        user: provider.wallet.publicKey,
        userTokenAccount,
        stakeState,
        stakeVault: vaultPda,
        mint,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      } as any)
      .rpc();

    console.log("✅ initializeStake tx:", tx);
    console.log("User token balance:", await getTokenAccountBalance(provider.connection, userTokenAccount));
    console.log("Vault token balance:", await getTokenAccountBalance(provider.connection, vaultPda));
    console.log("Stake state:", await getStakeState(program, stakeState));
  });

  it("Claims rewards", async () => {
    const tx = await program.methods
      .claimRewards()
      .accounts({
        user: provider.wallet.publicKey,
        userTokenAccount,
        stakeState,
        stakeVault: vaultPda,
        mint,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      } as any)
      .rpc();

    console.log("✅ claimRewards tx:", tx);
    console.log("User token balance:", await getTokenAccountBalance(provider.connection, userTokenAccount));
    console.log("Vault token balance:", await getTokenAccountBalance(provider.connection, vaultPda));
    console.log("Stake state:", await getStakeState(program, stakeState));
  });

  it("Unstakes tokens", async () => {
    const tx = await program.methods
      .unstake()
      .accounts({
        user: provider.wallet.publicKey,
        userTokenAccount,
        stakeState,
        stakeVault: vaultPda,
        mint,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      } as any)
      .rpc();

    console.log("✅ unstake tx:", tx);
    console.log("User token balance:", await getTokenAccountBalance(provider.connection, userTokenAccount));
    console.log("Vault token balance:", await getTokenAccountBalance(provider.connection, vaultPda));
    console.log("Stake state:", await getStakeState(program, stakeState));
  });
});