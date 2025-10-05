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

  // Zoomi PDAs
  const zoomiAccount = anchor.web3.PublicKey.findProgramAddressSync([Buffer.from("zoomi"), admin.publicKey.toBuffer()], program.programId)[0];
  let treasury: PublicKey;
  
  const zoomiDevicePubkey = admin.publicKey;
  const scooterAccount = anchor.web3.PublicKey.findProgramAddressSync([Buffer.from("scooter"), zoomiDevicePubkey.toBuffer()], program.programId)[0];


  before('Setup USDC mint based on network', async () => {
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
  });

  xit("Initialized Zoomi!", async () => {
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
    console.log("Your transaction signature", tx);
  });



});
