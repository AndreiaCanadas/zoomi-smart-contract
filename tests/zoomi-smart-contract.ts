import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { ZoomiSmartContract } from "../target/types/zoomi_smart_contract";
import { ASSOCIATED_PROGRAM_ID, TOKEN_PROGRAM_ID } from "@coral-xyz/anchor/dist/cjs/utils/token";
import {
  createMint,
  getAssociatedTokenAddressSync,
  getOrCreateAssociatedTokenAccount,
  mintTo,
} from "@solana/spl-token";
import wallet from "../Turbin3-wallet.json";
import NodeWallet from "@coral-xyz/anchor/dist/cjs/nodewallet";
import { Keypair, PublicKey, SystemProgram, LAMPORTS_PER_SOL } from "@solana/web3.js";

describe("zoomi-smart-contract", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const connection = provider.connection;
  const payer = provider.wallet as NodeWallet;

  const admin = Keypair.fromSecretKey(new Uint8Array(wallet));

  // Create deterministic keypairs from seeds
  const rider1 = Keypair.fromSeed(new Uint8Array(32).fill(233453));
  const rider2 = Keypair.fromSeed(new Uint8Array(32).fill(885581237));
  const shopkeeper = Keypair.fromSeed(new Uint8Array(32).fill(1293578));

  const program = anchor.workspace.zoomiSmartContract as Program<ZoomiSmartContract>;

  const tokenProgram = TOKEN_PROGRAM_ID;
  const associatedTokenProgram = ASSOCIATED_PROGRAM_ID;

  // USDC Mint and Token Accounts
  let mintUsdc: PublicKey;
  let adminUsdcAccount: any;
  let rider1UsdcAccount: any;
  let rider2UsdcAccount: any;
  let shopkeeperUsdcAccount: any;

  // Zoomi Accounts
  const zoomiAccount = anchor.web3.PublicKey.findProgramAddressSync([Buffer.from("zoomi"), admin.publicKey.toBuffer()], program.programId)[0];
  let treasury: PublicKey;
  
  // Scooter Accounts
  const zoomiDevicePubkey = shopkeeper.publicKey;
  const scooterAccount = anchor.web3.PublicKey.findProgramAddressSync([Buffer.from("scooty"), zoomiDevicePubkey.toBuffer()], program.programId)[0];

  // Rider Accounts
  const rider1Account = anchor.web3.PublicKey.findProgramAddressSync([Buffer.from("rider"), rider1.publicKey.toBuffer()], program.programId)[0];
  const rider2Account = anchor.web3.PublicKey.findProgramAddressSync([Buffer.from("rider"), rider2.publicKey.toBuffer()], program.programId)[0];

  // Rental Accounts
  const rental1Account = anchor.web3.PublicKey.findProgramAddressSync([rider1Account.toBuffer(), scooterAccount.toBuffer()], program.programId)[0];
  const rental2Account = anchor.web3.PublicKey.findProgramAddressSync([rider2Account.toBuffer(), scooterAccount.toBuffer()], program.programId)[0];
  let vault1: PublicKey;
  let vault2: PublicKey;


  function getRentalStatus(status: any): string {
    if (status.active !== undefined) return "Active";
    if (status.completed !== undefined) return "Completed";
    if (status.cancelled !== undefined) return "Cancelled";
    return "Unknown";
  }

  function getScooterStatus(status: any): string {
    if (status.available !== undefined) return "Available";
    if (status.rented !== undefined) return "Rented";
    if (status.booked !== undefined) return "Booked";
    if (status.maintenance !== undefined) return "Maintenance";
    return "Unknown";
  }

  // Helper function to transfer SOL between accounts
  async function transferSol(from: Keypair, to: PublicKey, amount: number): Promise<string> {
    const transaction = new anchor.web3.Transaction().add(
      SystemProgram.transfer({
        fromPubkey: from.publicKey,
        toPubkey: to,
        lamports: amount,
      })
    );
    
    const signature = await anchor.web3.sendAndConfirmTransaction(
      connection,
      transaction,
      [from]
    );
    
    return signature;
  }

  // Run only once to setup USDC mint
  xit('Create USDC mint', async () => {

    // Airdrop SOL to admin account for localnet testing
    // const tx = await connection.requestAirdrop(admin.publicKey, 2 * LAMPORTS_PER_SOL);
    // await connection.confirmTransaction(tx);
    // console.log("Airdropped SOL to admin account:", admin.publicKey.toString());
    // console.log("Admin SOL balance: ", await provider.connection.getBalance(admin.publicKey));
    
    // Create a USDC-like mint
    mintUsdc = await createMint(
      connection,
      payer.payer, // payer
      admin.publicKey, // mint authority
      null, // freeze authority (null = no freeze)
      6 // decimals (same as real USDC)
    );
    
    console.log("Created USDC-like mint:", mintUsdc.toString());
  });

  // If mint was already created - Setup all accounts and fund them
  it('Setup USDC mint and ATAs', async () => {
    mintUsdc = new PublicKey("7DJoVBdHiG6H8KMryNZ8av6xiX8MM8erMh8VztwYaGFR");
    
    // Transfer SOL from admin to rider and shopkeeper for transaction fees
    // const transferAmount = 0.3 * LAMPORTS_PER_SOL;
    // await transferSol(admin, rider1.publicKey, transferAmount);
    // await transferSol(admin, rider2.publicKey, transferAmount);
    // await transferSol(admin, shopkeeper.publicKey, transferAmount);

    // Create USDC token accounts for all parties
    adminUsdcAccount = await getOrCreateAssociatedTokenAccount(
      connection,
      payer.payer,
      mintUsdc,
      admin.publicKey
    );

    rider1UsdcAccount = await getOrCreateAssociatedTokenAccount(
      connection,
      payer.payer,
      mintUsdc,
      rider1.publicKey
    );

    rider2UsdcAccount = await getOrCreateAssociatedTokenAccount(
      connection,
      payer.payer,
      mintUsdc,
      rider2.publicKey
    );

    shopkeeperUsdcAccount = await getOrCreateAssociatedTokenAccount(
      connection,
      payer.payer,
      mintUsdc,
      shopkeeper.publicKey
    );

    // Mint USDC to riders for rental payments (500 USDC each)
    // await mintTo(
    //   connection,
    //   payer.payer,
    //   mintUsdc,
    //   rider1UsdcAccount.address,
    //   admin, // Mint authority
    //   500 * 10**6 // 500 USDC
    // );

    // await mintTo(
    //   connection,
    //   payer.payer,
    //   mintUsdc,
    //   rider2UsdcAccount.address,
    //   admin, // Mint authority
    //   500 * 10**6 // 500 USDC
    // );

    // Calculate treasury ATA address
    treasury = getAssociatedTokenAddressSync(mintUsdc, zoomiAccount, true);

    // Calculate vault ATA addresses
    vault1 = getAssociatedTokenAddressSync(mintUsdc, rental1Account, true);
    vault2 = getAssociatedTokenAddressSync(mintUsdc, rental2Account, true);

    console.log("\n✅ Setup complete:");
    console.log("Addresses:");
    console.log("  Admin:", admin.publicKey.toString());
    console.log("  Rider1:", rider1.publicKey.toString());
    console.log("  Rider2:", rider2.publicKey.toString());
    console.log("  Shopkeeper:", shopkeeper.publicKey.toString());
    console.log("USDC Balances:");
    console.log("  Rider1:", (await connection.getTokenAccountBalance(rider1UsdcAccount.address)).value.amount);
    console.log("  Rider2:", (await connection.getTokenAccountBalance(rider2UsdcAccount.address)).value.amount);
    console.log("  Shopkeeper:", (await connection.getTokenAccountBalance(shopkeeperUsdcAccount.address)).value.amount);
  });

  xit("Initialize Zoomi", async () => {
    // Add your test here.
    const tx = await program.methods.initializeZoomi(5, new anchor.BN(10000000), new anchor.BN(100000000))
      .accountsPartial({
        admin: admin.publicKey,
        zoomiAccount,
        mintUsdc,
        treasury,
        tokenProgram,
        associatedTokenProgram,
        systemProgram: SystemProgram.programId,
      })
      .signers([admin])
      .rpc();
    console.log("\nZoomi Account:", zoomiAccount.toBase58());
    console.log("Your transaction signature", tx);
  });

  xit("Register Scooter", async () => {
    const tx = await program.methods.registerScooter(shopkeeper.publicKey, 1, 123, new anchor.BN(2000000))
      .accountsPartial({
        shopkeeper: shopkeeper.publicKey,
        scooterAccount,
        systemProgram: SystemProgram.programId,
      })
      .signers([shopkeeper])
      .rpc();
    const scooterAccountFetched = await program.account.scooter.fetch(scooterAccount);
    console.log("\nScooter Account:", scooterAccount.toBase58());
    console.log(" - Zoomi Device Pubkey:", scooterAccountFetched.zoomiDevicePubkey.toBase58());
    console.log(" - ID:", scooterAccountFetched.id.toString());
    console.log(" - Shopkeeper ID:", scooterAccountFetched.shopkeeperId.toString());
    console.log(" - Hourly Rate:", scooterAccountFetched.hourlyRate.toString());
    console.log(" - Status:", getScooterStatus(scooterAccountFetched.status));
    console.log("Your transaction signature", tx);
  });

  xit("Register Rider 1", async () => {
    const tx = await program.methods.registerRider()
      .accountsPartial({
        rider: rider1.publicKey,
        riderAccount: rider1Account,
        systemProgram: SystemProgram.programId,
      })
      .signers([rider1])
      .rpc({skipPreflight: true});
    const riderAccountFetched = await program.account.rider.fetch(rider1Account);
    console.log("\nRider Account:", rider1Account.toBase58());
    console.log(" - ID Status:", riderAccountFetched.idStatus.toString());
    console.log(" - Is Renting:", riderAccountFetched.isRenting.toString());
    console.log(" - Points:", riderAccountFetched.points.toString());
    console.log(" - Penalties:", riderAccountFetched.penalties.toString());
    console.log("Your transaction signature", tx);
  });

  xit("Register Rider 2", async () => {
    const tx = await program.methods.registerRider()
      .accountsPartial({
        rider: rider2.publicKey,
        riderAccount: rider2Account,
        systemProgram: SystemProgram.programId,
      })
      .signers([rider2])
      .rpc();
    
    const riderAccountFetched = await program.account.rider.fetch(rider2Account);
    console.log("\nRider 2 Account:", rider2Account.toBase58());
    console.log(" - ID Status:", riderAccountFetched.idStatus.toString());
    console.log(" - Is Renting:", riderAccountFetched.isRenting.toString());
    console.log(" - Points:", riderAccountFetched.points.toString());
    console.log(" - Penalties:", riderAccountFetched.penalties.toString());
    console.log("Your transaction signature", tx);
  });

  xit("Start Rental for Rider 1", async () => {
    const tx = await program.methods.startRental(3)
      .accountsPartial({
        rider: rider1.publicKey,
        riderAccount: rider1Account,
        scooterAccount,
        rentalAccount: rental1Account,
        zoomiAccount,
        mintUsdc,
        vault: vault1,
        tokenProgram,
        associatedTokenProgram,
        systemProgram: SystemProgram.programId,
      })
      .signers([rider1])
      .rpc();
    const rentalAccountFetched = await program.account.rental.fetch(rental1Account);
    console.log("\nRental Account:", rental1Account.toBase58());
    console.log(" - Rider:", rentalAccountFetched.rider.toBase58());
    console.log(" - Scooter ID:", rentalAccountFetched.scooterId.toString());
    console.log(" - Start Time:", rentalAccountFetched.startTime.toString());
    console.log(" - Rental Period:", rentalAccountFetched.rentalPeriod.toString());
    console.log(" - Total Amount:", rentalAccountFetched.totalAmount.toString());
    console.log(" - Status:", getRentalStatus(rentalAccountFetched.status));
    console.log("Vault balance:", (await connection.getTokenAccountBalance(vault1)).value.amount);
    console.log("Rider 1 balance:", (await connection.getTokenAccountBalance(rider1UsdcAccount.address)).value.amount);
    console.log("Your transaction signature", tx);
  });

  xit("Extend Rental Period for Rider 1", async () => {
    const tx = await program.methods.extendRentalPeriod(2)
      .accountsPartial({
        rider: rider1.publicKey,
        riderAccount: rider1Account,
        scooterAccount,
        rentalAccount: rental1Account,
        zoomiAccount,
        mintUsdc,
        vault: vault1,
        tokenProgram,
        systemProgram: SystemProgram.programId,
      })
      .signers([rider1])
      .rpc();
    const rentalAccountFetched = await program.account.rental.fetch(rental1Account);
    console.log("\nRental Account:", rental1Account.toBase58());
    console.log(" - Rider:", rentalAccountFetched.rider.toBase58());
    console.log(" - Scooter ID:", rentalAccountFetched.scooterId.toString());
    console.log(" - Start Time:", rentalAccountFetched.startTime.toString());
    console.log(" - Rental Period:", rentalAccountFetched.rentalPeriod.toString());
    console.log(" - Total Amount:", rentalAccountFetched.totalAmount.toString());
    console.log(" - Status:", getRentalStatus(rentalAccountFetched.status));
    console.log("Vault balance:", (await connection.getTokenAccountBalance(vault1)).value.amount);
    console.log("Rider 1 balance:", (await connection.getTokenAccountBalance(rider1UsdcAccount.address)).value.amount);
    console.log("Your transaction signature", tx);
  });

  it("End Rental for Rider 1", async () => {
    const tx = await program.methods.returnScooter()
      .accountsPartial({
        rider: rider1.publicKey,
        riderAccount: rider1Account,
        scooterAccount,
        rentalAccount: rental1Account,
        shopkeeper: shopkeeper.publicKey,
        zoomiAccount,
        mintUsdc,
        vault: vault1,
        treasury,
        tokenProgram,
      })
      .signers([rider1])
      .rpc();
    const rentalAccountFetched = await program.account.rental.fetch(rental1Account);
    console.log("\nRental Account:", rental1Account.toBase58());
    console.log(" - Rider:", rentalAccountFetched.rider.toBase58());
    console.log(" - Scooter ID:", rentalAccountFetched.scooterId.toString());
    console.log(" - Start Time:", rentalAccountFetched.startTime.toString());
    console.log(" - Rental Period:", rentalAccountFetched.rentalPeriod.toString());
    console.log(" - Total Amount:", rentalAccountFetched.totalAmount.toString());
    console.log(" - Status:", getRentalStatus(rentalAccountFetched.status));
    const scooterAccountFetched = await program.account.scooter.fetch(scooterAccount);
    console.log("\nScooter Account:", scooterAccount.toBase58());
    console.log(" - Status:", getScooterStatus(scooterAccountFetched.status));
    const riderAccountFetched = await program.account.rider.fetch(rider1Account);
    console.log("\nRider Account:", rider1Account.toBase58());
    console.log(" - Is Renting:", riderAccountFetched.isRenting.toString());
    console.log(" - Points:", riderAccountFetched.points.toString());
    console.log(" - Penalties:", riderAccountFetched.penalties.toString());
    console.log("Vault balance:", (await connection.getTokenAccountBalance(vault1)).value.amount);
    console.log("Rider 1 balance:", (await connection.getTokenAccountBalance(rider1UsdcAccount.address)).value.amount);
    console.log("Shopkeeper balance:", (await connection.getTokenAccountBalance(shopkeeperUsdcAccount.address)).value.amount);
    console.log("Treasury balance:", (await connection.getTokenAccountBalance(treasury)).value.amount);
    console.log("Your transaction signature", tx);
  });

  xit("Close Rental - Good Condition for Rider 1", async () => {
    const tx = await program.methods.closeRental(85) // inspection_score: 85 (good condition - full collateral refund)
      .accountsPartial({
        shopkeeper: shopkeeper.publicKey,
        zoomiAccount,
        scooterAccount,
        riderAccount: rider1Account,
        rider: rider1.publicKey,
        rentalAccount: rental1Account,
        mintUsdc,
        vault: vault1,
        treasury,
        tokenProgram,
        systemProgram: SystemProgram.programId,
      })
      .signers([shopkeeper])
      .rpc();
      console.log("Close Rental - Good Condition for Rider 1");
      const scooterAccountFetched = await program.account.scooter.fetch(scooterAccount);
      console.log("\nScooter Account:", scooterAccount.toBase58());
      console.log(" - Status:", getScooterStatus(scooterAccountFetched.status));
      const riderAccountFetched = await program.account.rider.fetch(rider1Account);
      console.log("\nRider Account:", rider1Account.toBase58());
      console.log(" - Is Renting:", riderAccountFetched.isRenting.toString());
      console.log(" - Points:", riderAccountFetched.points.toString());
      console.log(" - Penalties:", riderAccountFetched.penalties.toString());
      console.log("Rider 1 balance:", (await connection.getTokenAccountBalance(rider1UsdcAccount.address)).value.amount);
      console.log("Shopkeeper balance:", (await connection.getTokenAccountBalance(shopkeeperUsdcAccount.address)).value.amount);
      console.log("Treasury balance:", (await connection.getTokenAccountBalance(treasury)).value.amount);

      console.log("Your transaction signature", tx);
  });

  xit("Start Rental for Rider 2", async () => {
    const tx = await program.methods.startRental(3)
      .accountsPartial({
        rider: rider2.publicKey,
        riderAccount: rider2Account,
        scooterAccount,
        rentalAccount: rental2Account,
        zoomiAccount,
        mintUsdc,
        vault: vault1,
        tokenProgram,
        associatedTokenProgram,
        systemProgram: SystemProgram.programId,
      })
      .signers([rider2])
      .rpc();
    const rentalAccountFetched = await program.account.rental.fetch(rental2Account);
    console.log("\nRental Account:", rental2Account.toBase58());
    console.log(" - Rider:", rentalAccountFetched.rider.toBase58());
    console.log(" - Scooter ID:", rentalAccountFetched.scooterId.toString());
    console.log(" - Start Time:", rentalAccountFetched.startTime.toString());
    console.log(" - Rental Period:", rentalAccountFetched.rentalPeriod.toString());
    console.log(" - Total Amount:", rentalAccountFetched.totalAmount.toString());
    console.log(" - Status:", getRentalStatus(rentalAccountFetched.status));
    console.log("Vault balance:", (await connection.getTokenAccountBalance(vault1)).value.amount);
    console.log("Rider 2 balance:", (await connection.getTokenAccountBalance(rider2UsdcAccount.address)).value.amount);
    console.log("Your transaction signature", tx);
  });

  xit("Close Rental - Poor Condition", async () => {
    // This test shows what happens when scooter is returned in poor condition
    const tx = await program.methods.closeRental(30) // inspection_score: 30 (poor condition - no collateral refund)
      .accountsPartial({
        shopkeeper: shopkeeper.publicKey,
        shopkeeperAta: shopkeeperUsdcAccount,
        zoomiAccount,
        scooterAccount,
        riderAccount: rider1Account,
        rider: rider1.publicKey,
        riderAta: rider1UsdcAccount,
        rentalAccount: rental1Account,
        mintUsdc,
        vault: vault1,
        treasury,
        tokenProgram,
        systemProgram: SystemProgram.programId,
      })
      .signers([shopkeeper])
      .rpc();

      console.log("Close Rental with poor inspection score (30) - collateral goes to shopkeeper");
      const scooterAccountFetched = await program.account.scooter.fetch(scooterAccount);
      console.log("\nScooter Account:", scooterAccount.toBase58());
      console.log(" - Status:", getScooterStatus(scooterAccountFetched.status));
      const riderAccountFetched = await program.account.rider.fetch(rider1Account);
      console.log("\nRider Account:", rider1Account.toBase58());
      console.log(" - Is Renting:", riderAccountFetched.isRenting.toString());
      console.log(" - Points:", riderAccountFetched.points.toString());
      console.log(" - Penalties:", riderAccountFetched.penalties.toString());
      console.log("Rider 1 balance:", (await connection.getTokenAccountBalance(rider1UsdcAccount.address)).value.amount);
      console.log("Shopkeeper balance:", (await connection.getTokenAccountBalance(shopkeeperUsdcAccount.address)).value.amount);
      console.log("Treasury balance:", (await connection.getTokenAccountBalance(treasury)).value.amount);
      console.log("Your transaction signature", tx);
  });

  xit("Close Rental - Test", async () => {
    const tx = await program.methods.closeRentalTest()
      .accountsPartial({
        rider: rider1.publicKey,
        riderAccount: rider1Account,
        scooterAccount,
        rentalAccount: rental1Account,
        mintUsdc,
        vault: vault1,
        tokenProgram,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
  });

  xit("Close Zoomi - Test", async () => {
    const tx = await program.methods.closeZoomi()
      .accountsPartial({
        admin: admin.publicKey,
        zoomiAccount,
        mintUsdc,
        treasury,
        tokenProgram,
        associatedTokenProgram,
        systemProgram: SystemProgram.programId,
      })
      .signers([admin])
      .rpc();
    console.log("Your transaction signature", tx);
  });

  xit("Update Scooter Location", async () => {
    const tx = await program.methods.updateScooterLocation(1, 2)
      .accountsPartial({
        scooterDevice: shopkeeper.publicKey, // Scooter device is owned by shopkeeper
        scooterAccount,
      })
      .signers([shopkeeper])
      .rpc();

      const scooterAccountFetched = await program.account.scooter.fetch(scooterAccount);
      console.log("\nScooter Account:", scooterAccount.toBase58());
      console.log(" - Location: ", scooterAccountFetched.locationLat.toString(), scooterAccountFetched.locationLong.toString());
      console.log(" - Status:", getScooterStatus(scooterAccountFetched.status));
  });

});
