import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { ZoomiSmartContract } from "../target/types/zoomi_smart_contract";
import { ASSOCIATED_PROGRAM_ID, TOKEN_PROGRAM_ID } from "@coral-xyz/anchor/dist/cjs/utils/token";
import {
  createMint,
  MINT_SIZE,
  getAssociatedTokenAddressSync,
  getMinimumBalanceForRentExemptMint,
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
  
  // Network configuration - change this to switch between networks
  const NETWORK = 'localnet'; // 'localnet', 'devnet', or 'mainnet'

  const program = anchor.workspace.zoomiSmartContract as Program<ZoomiSmartContract>;

  const tokenProgram = TOKEN_PROGRAM_ID;
  const associatedTokenProgram = ASSOCIATED_PROGRAM_ID;

  // USDC Mint
  let mintUsdc: PublicKey;

  // Zoomi Accounts
  const zoomiAccount = anchor.web3.PublicKey.findProgramAddressSync([Buffer.from("zoomi"), admin.publicKey.toBuffer()], program.programId)[0];
  let treasury: PublicKey;
  
  // Scooter Accounts
  const zoomiDevicePubkey = admin.publicKey;
  const scooterAccount = anchor.web3.PublicKey.findProgramAddressSync([Buffer.from("scooter"), zoomiDevicePubkey.toBuffer()], program.programId)[0];

  // Rider Accounts
  const riderAccount = anchor.web3.PublicKey.findProgramAddressSync([Buffer.from("rider"), admin.publicKey.toBuffer()], program.programId)[0];

  // Rental Accounts
  const rentalAccount = anchor.web3.PublicKey.findProgramAddressSync([riderAccount.toBuffer(), scooterAccount.toBuffer()], program.programId)[0];
  let vault: PublicKey;


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

  it('Setup USDC mint and ATAs based on network', async () => {
    if (NETWORK === 'localnet') {

      // Airdrop SOL to admin account for localnet testing
      const tx = await connection.requestAirdrop(admin.publicKey, 2 * LAMPORTS_PER_SOL);
      await connection.confirmTransaction(tx);
      console.log("Airdropped SOL to admin account:", admin.publicKey.toString());
      console.log("Admin balance: ", await provider.connection.getBalance(admin.publicKey));
      
      // Create a USDC-like mint
      mintUsdc = await createMint(
        connection,
        payer.payer, // payer
        admin.publicKey, // mint authority
        null, // freeze authority (null = no freeze)
        6 // decimals (same as real USDC)
      );
      
      console.log("Created USDC-like mint for localnet:", mintUsdc.toString());
      
      // Create admin's USDC token account and mint some test tokens
      const adminUsdcAccount = await getOrCreateAssociatedTokenAccount(
        connection,
        payer.payer,
        mintUsdc,
        admin.publicKey,
        true
      );
      
      // Mint 1000 USDC to admin for testing (1000 * 10^6 = 1,000,000,000 micro-USDC)
      await mintTo(
        connection,
        payer.payer,
        mintUsdc,
        adminUsdcAccount.address,
        admin,
        1000 * 10**6 // 1000 USDC
      );
      
    } else if (NETWORK === 'devnet') {
      // For devnet: Use the devnet USDC mint
      mintUsdc = new PublicKey("4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU");
      console.log("Using devnet USDC mint:", mintUsdc.toString());
      
    } else if (NETWORK === 'mainnet') {
      // For mainnet: Use the real USDC mint
      mintUsdc = new PublicKey("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
      console.log("Using mainnet USDC mint:", mintUsdc.toString());
    }
    
    // Calculate treasury ATA address
    treasury = getAssociatedTokenAddressSync(mintUsdc, zoomiAccount, true);

    // Calculate vault ATA address
    vault = getAssociatedTokenAddressSync(mintUsdc, rentalAccount, true);
  });

  it("Initialize Zoomi", async () => {
    // Add your test here.
    const tx = await program.methods.initializeZoomi(5, 100)
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

  it("Register Scooter", async () => {
    const tx = await program.methods.registerScooter(admin.publicKey, 1, 123, 2)
      .accountsPartial({
        shopkeeper: admin.publicKey,
        scooterAccount,
        systemProgram: SystemProgram.programId,
      })
      .signers([admin])
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

  it("Register Rider", async () => {
    const tx = await program.methods.registerRider()
      .accountsPartial({
        rider: admin.publicKey,
        riderAccount,
        systemProgram: SystemProgram.programId,
      })
      .signers([admin])
      .rpc();
    const riderAccountFetched = await program.account.rider.fetch(riderAccount);
    console.log("\nRider Account:", riderAccount.toBase58());
    console.log(" - ID Status:", riderAccountFetched.idStatus.toString());
    console.log(" - Is Renting:", riderAccountFetched.isRenting.toString());
    console.log(" - Points:", riderAccountFetched.points.toString());
    console.log(" - Penalties:", riderAccountFetched.penalties.toString());
    console.log("Your transaction signature", tx);
  });

  it("Start Rental", async () => {
    const tx = await program.methods.startRental(1, 100)
      .accountsPartial({
        rider: admin.publicKey,
        riderAccount,
        scooterAccount,
        rentalAccount,
        mintUsdc,
        vault,
        tokenProgram,
        associatedTokenProgram,
        systemProgram: SystemProgram.programId,
      })
      .signers([admin])
      .rpc();
    const rentalAccountFetched = await program.account.rental.fetch(rentalAccount);
    console.log("\nRental Account:", rentalAccount.toBase58());
    console.log(" - Rider:", rentalAccountFetched.rider.toBase58());
    console.log(" - Scooter ID:", rentalAccountFetched.scooterId.toString());
    console.log(" - Start Time:", rentalAccountFetched.startTime.toString());
    console.log(" - Rental Period:", rentalAccountFetched.rentalPeriod.toString());
    console.log(" - Total Amount:", rentalAccountFetched.totalAmount.toString());
    console.log(" - Penalty Time:", rentalAccountFetched.penaltyTime.toString());
    console.log(" - Status:", getRentalStatus(rentalAccountFetched.status));
    console.log("Your transaction signature", tx);
  });

  it("Extend Rental Period", async () => {
    const tx = await program.methods.extendRentalPeriod(1, 100)
      .accountsPartial({
        rider: admin.publicKey,
        riderAccount,
        scooterAccount,
        rentalAccount,
      })
      .signers([admin])
      .rpc();
      const rentalAccountFetched = await program.account.rental.fetch(rentalAccount);
      console.log("\nRental Account:", rentalAccount.toBase58());
      console.log(" - Rider:", rentalAccountFetched.rider.toBase58());
      console.log(" - Scooter ID:", rentalAccountFetched.scooterId.toString());
      console.log(" - Start Time:", rentalAccountFetched.startTime.toString());
      console.log(" - Rental Period:", rentalAccountFetched.rentalPeriod.toString());
      console.log(" - Total Amount:", rentalAccountFetched.totalAmount.toString());
      console.log(" - Penalty Time:", rentalAccountFetched.penaltyTime.toString());
      console.log(" - Status:", getRentalStatus(rentalAccountFetched.status));
    console.log("Your transaction signature", tx);
  });

  it("End Rental", async () => {
    const tx = await program.methods.endRental()
      .accountsPartial({
        rider: admin.publicKey,
        riderAccount,
        scooterAccount,
        rentalAccount,
        zoomiAccount,
        mintUsdc,
        vault,
        treasury,
        tokenProgram,
        systemProgram: SystemProgram.programId,
      })
      .signers([admin])
      .rpc();
    const rentalAccountFetched = await program.account.rental.fetch(rentalAccount);
    console.log("\nRental Account:", rentalAccount.toBase58());
    console.log(" - Rider:", rentalAccountFetched.rider.toBase58());
    console.log(" - Scooter ID:", rentalAccountFetched.scooterId.toString());
    console.log(" - Start Time:", rentalAccountFetched.startTime.toString());
    console.log(" - Rental Period:", rentalAccountFetched.rentalPeriod.toString());
    console.log(" - Total Amount:", rentalAccountFetched.totalAmount.toString());
    console.log(" - Penalty Time:", rentalAccountFetched.penaltyTime.toString());
    console.log(" - Status:", getRentalStatus(rentalAccountFetched.status));
    const scooterAccountFetched = await program.account.scooter.fetch(scooterAccount);
    console.log("\nScooter Account:", scooterAccount.toBase58());
    console.log(" - Status:", getScooterStatus(scooterAccountFetched.status));
    const riderAccountFetched = await program.account.rider.fetch(riderAccount);
    console.log("\nRider Account:", riderAccount.toBase58());
    console.log(" - Is Renting:", riderAccountFetched.isRenting.toString());
    console.log(" - Points:", riderAccountFetched.points.toString());
    console.log(" - Penalties:", riderAccountFetched.penalties.toString());
    console.log("Your transaction signature", tx);
  });

});
