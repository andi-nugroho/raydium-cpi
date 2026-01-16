import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { RaydiumCpi } from "../target/types/raydium_cpi";
import {
  AddressLookupTableProgram,
  ComputeBudgetProgram,
  Connection,
  Keypair,
  PublicKey,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
  Transaction,
  TransactionMessage,
  VersionedTransaction,
} from "@solana/web3.js";
import {
  RaydiumLaunchpad,
  IDL as RaydiumLaunchpadIDL,
} from "../idl-types/raydium_launchpad";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  createAssociatedTokenAccountIdempotentInstructionWithDerivation,
  getAssociatedTokenAddressSync,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import { BN } from "bn.js";

async function createAndPopulateALT(
  connection: Connection,
  payer: Keypair,
  addresses: PublicKey[]
): Promise<PublicKey> {
  const slot = await connection.getSlot();

  const [createIx, lookupTableAddress] =
    AddressLookupTableProgram.createLookupTable({
      authority: payer.publicKey,
      payer: payer.publicKey,
      recentSlot: slot,
    });

  const extendIx = AddressLookupTableProgram.extendLookupTable({
    payer: payer.publicKey,
    authority: payer.publicKey,
    lookupTable: lookupTableAddress,
    addresses: addresses,
  });

  const tx = new Transaction().add(createIx, extendIx);
  await connection.sendTransaction(tx, [payer]);

  await new Promise((resolve) => setTimeout(resolve, 1000));

  return lookupTableAddress;
}

describe("raydium-cpi", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.raydiumCpi as Program<RaydiumCpi>;
  const raydiumProgram = new Program<RaydiumLaunchpad>(
    RaydiumLaunchpadIDL,
    program.provider
  );
  const platformConfig = new PublicKey(
    "BuM6KDpWiTcxvrpXywWFiw45R2RNH8WURdvqoTDV1BW4"
  );

  const baseTokenKeypair = new Keypair();
  const baseTokenMint = baseTokenKeypair.publicKey;

  const quoteTokenMint = new PublicKey(
    "USD1ttGY1N17NEEHLmELoaybftRBUSErhqYiQzvEmuB"
  );

  it("Create!", async () => {
    const name = "Raydium CPI Launch Token";
    const symbol = "RAYCPI";
    const uri =
      "https://gateway.pinata.cloud/ipfs/bafkreigvfdqkujdxm6eyii4zxyxzfephwbxdfdighf5534w6cmot3a5uji";

    const computeUnitIx = ComputeBudgetProgram.setComputeUnitLimit({
      units: 500_000,
    });

    const tx = await program.methods
      .create({
        name,
        symbol,
        uri,
      })
      .accounts({
        platformConfig,
        baseTokenMint,
        quoteTokenMint,
      })
      .preInstructions([computeUnitIx])
      .signers([baseTokenKeypair])
      .rpc();

    console.log(`https://explorer.solana.com/tx/${tx}?cluster=custom`);
  });

  it("Buy!", async () => {
    const amountIn = new BN(100_000_000);
    const minimumAmountOut = new BN(0);
    const shareFeeRate = new BN(0);

    const [platformFeeVault] = PublicKey.findProgramAddressSync(
      [platformConfig.toBuffer(), quoteTokenMint.toBuffer()],
      raydiumProgram.programId
    );

    const creator = program.provider.wallet.publicKey;

    const [creatorFeeVault] = PublicKey.findProgramAddressSync(
      [creator.toBuffer(), quoteTokenMint.toBuffer()],
      raydiumProgram.programId
    );

    const createBaseAta =
      createAssociatedTokenAccountIdempotentInstructionWithDerivation(
        creator,
        creator,
        baseTokenMint
      );

    const remainingAccounts = [
      {
        pubkey: SystemProgram.programId,
        isWritable: false,
        isSigner: false,
      },
      {
        pubkey: platformFeeVault,
        isWritable: true,
        isSigner: false,
      },
      {
        pubkey: creatorFeeVault,
        isWritable: true,
        isSigner: false,
      },
    ];

    const tx = await program.methods
      .buy(amountIn, minimumAmountOut, shareFeeRate)
      .accounts({
        platformConfig,
        baseTokenMint,
        quoteTokenMint,
      })
      .preInstructions([createBaseAta])
      .remainingAccounts(remainingAccounts)
      .rpc();

    console.log(`https://explorer.solana.com/tx/${tx}?cluster=custom`);
  });

  it("Create and Buy! : atomic", async () => {
    const baseTokenKeypair = new Keypair();
    const baseTokenMint = baseTokenKeypair.publicKey;
    const productId = 100;
    const name = "Raydium Atoic CPI Token";
    const symbol = "RAYCPIA";
    const uri =
      "https://gateway.pinata.cloud/ipfs/bafkreigvfdqkujdxm6eyii4zxyxzfephwbxdfdighf5534w6cmot3a5uji";

    const amountIn = new BN(100_000_000);
    const minimumAmountOut = new BN(0);
    const shareFeeRate = new BN(0);

    const creator = program.provider.wallet.publicKey;

    const [platformFeeVault] = PublicKey.findProgramAddressSync(
      [platformConfig.toBuffer(), quoteTokenMint.toBuffer()],
      raydiumProgram.programId
    );

    const [creatorFeeVault] = PublicKey.findProgramAddressSync(
      [creator.toBuffer(), quoteTokenMint.toBuffer()],
      raydiumProgram.programId
    );

    // Get all the PDAs and accounts
    const [authority] = PublicKey.findProgramAddressSync(
      [Buffer.from("vault_auth_seed")],
      raydiumProgram.programId
    );

    const [globalConfig] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("global_config"),
        quoteTokenMint.toBuffer(),
        Buffer.from([0]),
        Buffer.from([0, 0]),
      ],
      raydiumProgram.programId
    );

    const [poolState] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("pool"),
        baseTokenMint.toBuffer(),
        quoteTokenMint.toBuffer(),
      ],
      raydiumProgram.programId
    );

    const [baseVault] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("pool_vault"),
        poolState.toBuffer(),
        baseTokenMint.toBuffer(),
      ],
      raydiumProgram.programId
    );

    const [quoteVault] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("pool_vault"),
        poolState.toBuffer(),
        quoteTokenMint.toBuffer(),
      ],
      raydiumProgram.programId
    );

    const [eventAuthority] = PublicKey.findProgramAddressSync(
      [Buffer.from("__event_authority")],
      raydiumProgram.programId
    );

    const userBaseAta = getAssociatedTokenAddressSync(baseTokenMint, creator);

    const userQuoteAta = getAssociatedTokenAddressSync(quoteTokenMint, creator);

    // Create ALT with all frequently used addresses
    const addressesToStore = [
      raydiumProgram.programId,
      TOKEN_PROGRAM_ID,
      ASSOCIATED_TOKEN_PROGRAM_ID,
      SystemProgram.programId,
      SYSVAR_RENT_PUBKEY,
      authority,
      globalConfig,
      platformConfig,
      poolState,
      baseVault,
      quoteVault,
      eventAuthority,
      baseTokenMint,
      quoteTokenMint,
      platformFeeVault,
      creatorFeeVault,
      userBaseAta,
      userQuoteAta,
    ];

    console.log("Creating ALT...");
    const lookupTableAddress = await createAndPopulateALT(
      program.provider.connection,
      program.provider.wallet.payer,
      addressesToStore
    );

    // Fetch the lookup table
    const lookupTableAccount =
      await program.provider.connection.getAddressLookupTable(
        lookupTableAddress
      );

    const createBaseAta =
      createAssociatedTokenAccountIdempotentInstructionWithDerivation(
        creator,
        creator,
        baseTokenMint
      );

    const remainingAccounts = [
      {
        pubkey: SystemProgram.programId,
        isWritable: false,
        isSigner: false,
      },
      {
        pubkey: platformFeeVault,
        isWritable: true,
        isSigner: false,
      },
      {
        pubkey: creatorFeeVault,
        isWritable: true,
        isSigner: false,
      },
    ];

    const computeUnitIx = ComputeBudgetProgram.setComputeUnitLimit({
      units: 500_000,
    });

    const createIx = await program.methods
      .atomic({
        name,
        symbol,
        uri,
      })
      .accounts({
        platformConfig,
        baseTokenMint,
        quoteTokenMint,
      } as any)
      .instruction();

    const buyIx = await program.methods
      .buy(amountIn, minimumAmountOut, shareFeeRate)
      .accounts({
        platformConfig,
        baseTokenMint,
        quoteTokenMint,
      } as any)
      .remainingAccounts(remainingAccounts)
      .instruction();

    // Create versioned transaction with ALT
    const { blockhash } =
      await program.provider.connection.getLatestBlockhash();

    const messageV0 = new TransactionMessage({
      payerKey: creator,
      recentBlockhash: blockhash,
      instructions: [computeUnitIx, createIx, createBaseAta, buyIx],
    }).compileToV0Message([lookupTableAccount.value]);

    const transaction = new VersionedTransaction(messageV0);
    transaction.sign([program.provider.wallet.payer, baseTokenKeypair]);

    const tx = await program.provider.connection.sendTransaction(transaction, {
      skipPreflight: false,
    });

    await program.provider.connection.confirmTransaction(tx, "finalized");

    console.log(`https://explorer.solana.com/tx/${tx}?cluster=custom`);
  });
});
