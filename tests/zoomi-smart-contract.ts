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
  const zoomiDevicePubkey = admin.publicKey;
  const scooterAccount = anchor.web3.PublicKey.findProgramAddressSync([Buffer.from("scooty"), zoomiDevicePubkey.toBuffer()], program.programId)[0];
  console.log("Scooter Account:", scooterAccount.toString());

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
    console.log("  Rider1: $", ((await connection.getTokenAccountBalance(rider1UsdcAccount.address)).value.amount / 1_000_000).toFixed(2));
    console.log("  Rider2: $", ((await connection.getTokenAccountBalance(rider2UsdcAccount.address)).value.amount / 1_000_000).toFixed(2));
    console.log("  Shopkeeper: $", ((await connection.getTokenAccountBalance(shopkeeperUsdcAccount.address)).value.amount / 1_000_000).toFixed(2));
  });

  xit("Initialize Zoomi", async () => {
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
    
    const zoomiAccountFetched = await program.account.zoomi.fetch(zoomiAccount);
    console.log("\n:: Zoomi Platform Initialized");
    console.log("Protocol Settings:");
    console.log("  Base Rate: $" + (zoomiAccountFetched.baseRate.toNumber() / 1_000_000).toFixed(2));
    console.log("  Collateral: $" + (zoomiAccountFetched.collateral.toNumber() / 1_000_000).toFixed(2));
    console.log("  Protocol Fee: " + zoomiAccountFetched.protocolFee + "%");
  });

  xit("Register Scooter", async () => {
    const tx = await program.methods.registerScooter(admin.publicKey, 1, 123, new anchor.BN(2000000))
      .accountsPartial({
        shopkeeper: shopkeeper.publicKey,
        scooterAccount,
        systemProgram: SystemProgram.programId,
      })
      .signers([shopkeeper])
      .rpc();
    
    const scooterAccountFetched = await program.account.scooter.fetch(scooterAccount);
    console.log("\n:: Scooter Registered");
    console.log("Scooter Details:");
    console.log("  ID: #" + scooterAccountFetched.id);
    console.log("  Shopkeeper ID: " + scooterAccountFetched.shopkeeperId);
    console.log("  Hourly Rate: $" + (scooterAccountFetched.hourlyRate.toNumber() / 1_000_000).toFixed(2));
    console.log("  Status: " + getScooterStatus(scooterAccountFetched.status));
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
    
    console.log("\n:: Rider 1 Registered");
    console.log("Account Created: " + rider1Account.toBase58().slice(0, 8) + "...");
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
    
    console.log("\n:: Rider 2 Registered");
    console.log("Account Created: " + rider2Account.toBase58().slice(0, 8) + "...");
  });

  it("Start Rental for Rider 1", async () => {
    const balanceBefore = (await connection.getTokenAccountBalance(rider1UsdcAccount.address)).value.amount;
    
    const tx = await program.methods.startRental(1)
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
    const scooterAccountFetched = await program.account.scooter.fetch(scooterAccount);
    const vaultBalance = (await connection.getTokenAccountBalance(vault1)).value.amount;
    const balanceAfter = (await connection.getTokenAccountBalance(rider1UsdcAccount.address)).value.amount;
    
    console.log("\n:: Rental Started - Rider 1");
    console.log("Rental Details:");
    console.log("  Period: " + rentalAccountFetched.rentalPeriod + " hours");
    console.log("  Rental Amount: $" + (rentalAccountFetched.rentalAmount.toNumber() / 1_000_000).toFixed(2));
    console.log("  Scooter Status: " + getScooterStatus(scooterAccountFetched.status));
    console.log("USDC Balances:");
    console.log("  Rider: $" + (parseInt(balanceAfter) / 1_000_000).toFixed(2) + " (paid: $" + ((parseInt(balanceBefore) - parseInt(balanceAfter)) / 1_000_000).toFixed(2) + ")");
    console.log("  Vault (Escrow): $" + (parseInt(vaultBalance) / 1_000_000).toFixed(2));
  });

  xit("Extend Rental Period for Rider 1", async () => {
    const balanceBefore = (await connection.getTokenAccountBalance(rider1UsdcAccount.address)).value.amount;
    
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
    const vaultBalance = (await connection.getTokenAccountBalance(vault1)).value.amount;
    const balanceAfter = (await connection.getTokenAccountBalance(rider1UsdcAccount.address)).value.amount;
    
    console.log("\n:: Rental Extended - Rider 1");
    console.log("Updated Details:");
    console.log("  New Period: " + rentalAccountFetched.rentalPeriod + " hours");
    console.log("  Total Rental Amount: $" + (rentalAccountFetched.rentalAmount.toNumber() / 1_000_000).toFixed(2));
    console.log("USDC Balances:");
    console.log("  Rider: $" + (parseInt(balanceAfter) / 1_000_000).toFixed(2) + " (additional paid: $" + ((parseInt(balanceBefore) - parseInt(balanceAfter)) / 1_000_000).toFixed(2) + ")");
    console.log("  Vault (Escrow): $" + (parseInt(vaultBalance) / 1_000_000).toFixed(2));
  });

  xit("End Rental for Rider 1", async () => {
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
    const scooterAccountFetched = await program.account.scooter.fetch(scooterAccount);
    const riderAccountFetched = await program.account.rider.fetch(rider1Account);
    const vaultBalance = (await connection.getTokenAccountBalance(vault1)).value.amount;
    const riderBalance = (await connection.getTokenAccountBalance(rider1UsdcAccount.address)).value.amount;
    const shopkeeperBalance = (await connection.getTokenAccountBalance(shopkeeperUsdcAccount.address)).value.amount;
    const treasuryBalance = (await connection.getTokenAccountBalance(treasury)).value.amount;
    
    console.log("\n:: Rental Ended - Rider 1");
    console.log("Rental Status: " + getRentalStatus(rentalAccountFetched.status));
    console.log("Scooter Status: " + getScooterStatus(scooterAccountFetched.status));
    console.log("Rider Stats:");
    console.log("  Points Earned: " + riderAccountFetched.points);
    console.log("  Penalties: " + riderAccountFetched.penalties);
    console.log("USDC Balances:");
    console.log("  Vault (Remaining): $" + (parseInt(vaultBalance) / 1_000_000).toFixed(2));
    console.log("  Rider: $" + (parseInt(riderBalance) / 1_000_000).toFixed(2));
    console.log("  Shopkeeper: $" + (parseInt(shopkeeperBalance) / 1_000_000).toFixed(2));
    console.log("  Treasury (Fees): $" + (parseInt(treasuryBalance) / 1_000_000).toFixed(2));
  });

  xit("Close Rental for Rider 1 - Scooter is OK (Good Condition)", async () => {
    const riderBalanceBefore = (await connection.getTokenAccountBalance(rider1UsdcAccount.address)).value.amount;
    
    const tx = await program.methods.closeRental(true) // Scooter OK
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
    
    const riderBalanceAfter = (await connection.getTokenAccountBalance(rider1UsdcAccount.address)).value.amount;
    const collateralRefunded = (parseInt(riderBalanceAfter) - parseInt(riderBalanceBefore)) / 1_000_000;
    
    console.log("\n:: Rental Closed - Scooter OK");
    console.log("Inspection Result: Good Condition");
    console.log("Collateral Refunded: $" + collateralRefunded.toFixed(2));
    console.log("Final USDC Balances:");
    console.log("  Rider: $" + (parseInt(riderBalanceAfter) / 1_000_000).toFixed(2));
    console.log("  Shopkeeper: $" + ((await connection.getTokenAccountBalance(shopkeeperUsdcAccount.address)).value.amount / 1_000_000).toFixed(2));
    console.log("  Treasury: $" + ((await connection.getTokenAccountBalance(treasury)).value.amount / 1_000_000).toFixed(2));
  });

  xit("Set Scooter Status to Available", async () => {
    const scooterStatus = {available: {}};
    const tx = await program.methods.setScooterStatus(scooterStatus)
      .accountsPartial({
        shopkeeper: shopkeeper.publicKey,
        scooterAccount,
      })
      .signers([shopkeeper])
      .rpc();
    
    console.log("\n:: Scooter Status Updated: Available");
  });

  xit("Start Rental for Rider 2", async () => {
    const balanceBefore = (await connection.getTokenAccountBalance(rider2UsdcAccount.address)).value.amount;
    
    const tx = await program.methods.startRental(5)
      .accountsPartial({
        rider: rider2.publicKey,
        riderAccount: rider2Account,
        scooterAccount,
        rentalAccount: rental2Account,
        zoomiAccount,
        mintUsdc,
        vault: vault2,
        tokenProgram,
        associatedTokenProgram,
        systemProgram: SystemProgram.programId,
      })
      .signers([rider2])
      .rpc();

    const rentalAccountFetched = await program.account.rental.fetch(rental2Account);
    const scooterAccountFetched = await program.account.scooter.fetch(scooterAccount);
    const vaultBalance = (await connection.getTokenAccountBalance(vault2)).value.amount;
    const balanceAfter = (await connection.getTokenAccountBalance(rider2UsdcAccount.address)).value.amount;
    
    console.log("\n:: Rental Started - Rider 2");
    console.log("Rental Details:");
    console.log("  Period: " + rentalAccountFetched.rentalPeriod + " hours");
    console.log("  Rental Amount: $" + (rentalAccountFetched.rentalAmount.toNumber() / 1_000_000).toFixed(2));
    console.log("  Scooter Status: " + getScooterStatus(scooterAccountFetched.status));
    console.log("USDC Balances:");
    console.log("  Rider: $" + (parseInt(balanceAfter) / 1_000_000).toFixed(2) + " (paid: $" + ((parseInt(balanceBefore) - parseInt(balanceAfter)) / 1_000_000).toFixed(2) + ")");
    console.log("  Vault (Escrow): $" + (parseInt(vaultBalance) / 1_000_000).toFixed(2));
  });

  xit("End Rental for Rider 2", async () => {
    const tx = await program.methods.returnScooter()
      .accountsPartial({
        rider: rider2.publicKey,
        riderAccount: rider2Account,
        scooterAccount,
        rentalAccount: rental2Account,
        shopkeeper: shopkeeper.publicKey,
        zoomiAccount,
        mintUsdc,
        vault: vault2,
        treasury,
        tokenProgram,
      })
      .signers([rider2])
      .rpc();
    
    const rentalAccountFetched = await program.account.rental.fetch(rental2Account);
    const scooterAccountFetched = await program.account.scooter.fetch(scooterAccount);
    const riderAccountFetched = await program.account.rider.fetch(rider2Account);
    const vaultBalance = (await connection.getTokenAccountBalance(vault2)).value.amount;
    const riderBalance = (await connection.getTokenAccountBalance(rider2UsdcAccount.address)).value.amount;
    const shopkeeperBalance = (await connection.getTokenAccountBalance(shopkeeperUsdcAccount.address)).value.amount;
    const treasuryBalance = (await connection.getTokenAccountBalance(treasury)).value.amount;
    
    console.log("\n:: Rental Ended - Rider 2");
    console.log("Rental Status: " + getRentalStatus(rentalAccountFetched.status));
    console.log("Scooter Status: " + getScooterStatus(scooterAccountFetched.status));
    console.log("Rider Stats:");
    console.log("  Points Earned: " + riderAccountFetched.points);
    console.log("  Penalties: " + riderAccountFetched.penalties);
    console.log("USDC Balances:");
    console.log("  Vault (Remaining): $" + (parseInt(vaultBalance) / 1_000_000).toFixed(2));
    console.log("  Rider: $" + (parseInt(riderBalance) / 1_000_000).toFixed(2));
    console.log("  Shopkeeper: $" + (parseInt(shopkeeperBalance) / 1_000_000).toFixed(2));
    console.log("  Treasury (Fees): $" + (parseInt(treasuryBalance) / 1_000_000).toFixed(2));
  });

  xit("Close Rental for Rider 2 - Scooter is not OK (Bad Condition)", async () => {
    const shopkeeperBalanceBefore = (await connection.getTokenAccountBalance(shopkeeperUsdcAccount.address)).value.amount;
    
    const tx = await program.methods.closeRental(false) // Scooter NOT OK
      .accountsPartial({
        shopkeeper: shopkeeper.publicKey,
        zoomiAccount,
        scooterAccount,
        riderAccount: rider2Account,
        rider: rider2.publicKey,
        rentalAccount: rental2Account,
        mintUsdc,
        vault: vault2,
        treasury,
        tokenProgram,
        systemProgram: SystemProgram.programId,
      })
      .signers([shopkeeper])
      .rpc();
    
    const shopkeeperBalanceAfter = (await connection.getTokenAccountBalance(shopkeeperUsdcAccount.address)).value.amount;
    const collateralToShopkeeper = (parseInt(shopkeeperBalanceAfter) - parseInt(shopkeeperBalanceBefore)) / 1_000_000;
    
    console.log("\n:: Rental Closed - Scooter Damaged");
    console.log("Inspection Result: Bad Condition");
    console.log("Collateral Forfeited to Shopkeeper: $" + collateralToShopkeeper.toFixed(2));
    console.log("Final USDC Balances:");
    console.log("  Rider: $" + ((await connection.getTokenAccountBalance(rider2UsdcAccount.address)).value.amount / 1_000_000).toFixed(2));
    console.log("  Shopkeeper: $" + (parseInt(shopkeeperBalanceAfter) / 1_000_000).toFixed(2));
    console.log("  Treasury: $" + ((await connection.getTokenAccountBalance(treasury)).value.amount / 1_000_000).toFixed(2));
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
    console.log("\n:: Scooter Location Updated");
    console.log("New Coordinates: (" + scooterAccountFetched.locationLat + ", " + scooterAccountFetched.locationLong + ")");
    console.log("Status: " + getScooterStatus(scooterAccountFetched.status));
  });

});
