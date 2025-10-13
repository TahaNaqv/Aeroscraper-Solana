import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { AerospacerOracle } from "../target/types/aerospacer_oracle";
import { PublicKey } from "@solana/web3.js";
import * as fs from "fs";

/**
 * Test Real-Time Pyth Price Queries on Devnet
 * 
 * This script queries real Pyth price feeds for SOL, ETH, and BTC on devnet
 * and displays the current prices with proper formatting.
 */

async function main() {
  console.log("\n🔍 Testing Real-Time Pyth Price Feeds on Devnet...\n");

  // Configure provider for devnet
  const provider = anchor.AnchorProvider.local("https://api.devnet.solana.com");
  anchor.setProvider(provider);

  const program = anchor.workspace.AerospacerOracle as Program<AerospacerOracle>;
  
  // Load state account from saved config
  let stateAccountPubkey: PublicKey;
  
  try {
    const configData = fs.readFileSync("scripts/.oracle-devnet-config.json", "utf-8");
    const config = JSON.parse(configData);
    stateAccountPubkey = new PublicKey(config.stateAccount);
  } catch (error) {
    console.error("❌ Error: Could not load oracle config.");
    console.error("   Please run 'npm run init_oracle_devnet' and 'npm run add_assets_devnet' first!");
    process.exit(1);
  }

  // Pyth price accounts on devnet
  const priceFeeds = [
    {
      denom: "SOL",
      pythAccount: new PublicKey("J83w4HKfqxwcq3BEMMkPFSppX3gqekLyLJBexebFVkix"),
      pair: "SOL/USD",
    },
    {
      denom: "ETH",
      pythAccount: new PublicKey("EdVCmQ9FSPcVe5YySXDPCRmc8aDQLKJ9xvYBMZPie1Vw"),
      pair: "ETH/USD",
    },
    {
      denom: "BTC",
      pythAccount: new PublicKey("HovQMDrbAgAYPCmHVSrezcSmkMtXSSUsLDFANExrZh2J"),
      pair: "BTC/USD",
    },
  ];

  console.log("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
  console.log("           REAL-TIME PYTH PRICE FEEDS");
  console.log("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

  for (const feed of priceFeeds) {
    try {
      console.log(`📊 Querying ${feed.pair}...`);

      const priceResponse = await program.methods
        .getPrice({ denom: feed.denom })
        .accounts({
          state: stateAccountPubkey,
          pythPriceAccount: feed.pythAccount,
          clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
        })
        .view();

      // Calculate human-readable price
      const price = Number(priceResponse.price);
      const exponent = priceResponse.exponent;
      const confidence = Number(priceResponse.confidence);
      const humanPrice = price * Math.pow(10, exponent);
      const humanConfidence = confidence * Math.pow(10, exponent);

      // Format timestamp
      const timestamp = new Date(Number(priceResponse.timestamp) * 1000);
      const now = new Date();
      const ageSeconds = Math.floor((now.getTime() - timestamp.getTime()) / 1000);

      console.log(`   ✅ Price: $${humanPrice.toFixed(2)} ± $${humanConfidence.toFixed(2)}`);
      console.log(`   📈 Raw: ${price} × 10^${exponent}`);
      console.log(`   🔒 Confidence: ${confidence}`);
      console.log(`   ⏰ Updated: ${ageSeconds}s ago`);
      console.log(`   📅 Timestamp: ${timestamp.toISOString()}`);
      console.log(`   🔢 Decimal: ${priceResponse.decimal}`);
      console.log("");

    } catch (error: any) {
      console.error(`   ❌ Failed to query ${feed.pair}:`, error.message);
      console.log("");
    }
  }

  console.log("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
  console.log("✅ Price Feed Test Complete!");
  console.log("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

  console.log("🔍 Testing get_all_prices (batch query)...\n");

  try {
    const allPrices = await program.methods
      .getAllPrices({})
      .accounts({
        state: stateAccountPubkey,
        clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
      })
      .remainingAccounts(
        priceFeeds.map((feed) => ({
          pubkey: feed.pythAccount,
          isSigner: false,
          isWritable: false,
        }))
      )
      .view();

    console.log(`✅ Retrieved ${allPrices.length} prices in a single batch query:\n`);

    allPrices.forEach((priceData: any) => {
      const price = Number(priceData.price);
      const exponent = priceData.exponent;
      const humanPrice = price * Math.pow(10, exponent);
      
      console.log(`   ${priceData.denom}: $${humanPrice.toFixed(2)}`);
    });

    console.log("");
    console.log("🎉 All Pyth integrations working correctly on devnet!");
    console.log("");
    console.log("🎯 Ready for comprehensive testing:");
    console.log("   Run: npm run test-oracle-devnet");
    console.log("");

  } catch (error: any) {
    console.error("❌ Batch query failed:", error.message);
    console.log("");
  }
}

main()
  .then(() => {
    console.log("✨ Done!\n");
    process.exit(0);
  })
  .catch((error) => {
    console.error(error);
    process.exit(1);
  });
