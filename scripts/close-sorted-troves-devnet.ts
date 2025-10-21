import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { AerospacerProtocol } from "../target/types/aerospacer_protocol";
import { PublicKey } from "@solana/web3.js";

/**
 * DEVNET CLEANUP SCRIPT: Close Corrupted Sorted Troves State
 * 
 * This script calls the reset_sorted_troves admin instruction to close the corrupted
 * SortedTrovesState account. The next openTrove call will automatically reinitialize
 * a fresh, empty sorted list.
 * 
 * USAGE:
 *   ANCHOR_PROVIDER_URL=https://api.devnet.solana.com \
 *   ANCHOR_WALLET=~/.config/solana/id.json \
 *   npx ts-node scripts/close-sorted-troves-devnet.ts
 * 
 * REQUIREMENTS:
 *   - You must be the protocol authority (admin wallet)
 *   - The program must be deployed with the reset_sorted_troves instruction
 */

async function main() {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const protocolProgram = anchor.workspace.AerospacerProtocol as Program<AerospacerProtocol>;
  const admin = provider.wallet;

  console.log("🧹 Closing corrupted SortedTrovesState on devnet...");
  console.log(`Admin: ${admin.publicKey.toString()}`);
  console.log(`Protocol: ${protocolProgram.programId.toString()}\n`);

  // Derive PDAs
  const [statePDA] = PublicKey.findProgramAddressSync(
    [Buffer.from("state")],
    protocolProgram.programId
  );

  const [sortedTrovesStatePDA] = PublicKey.findProgramAddressSync(
    [Buffer.from("sorted_troves")],
    protocolProgram.programId
  );

  console.log(`State PDA: ${statePDA.toString()}`);
  console.log(`SortedTrovesState PDA: ${sortedTrovesStatePDA.toString()}\n`);

  // Check current state before closing
  try {
    const currentState = await protocolProgram.account.sortedTrovesState.fetch(sortedTrovesStatePDA);
    console.log("Current SortedTrovesState:");
    console.log(`  Size: ${currentState.size}`);
    console.log(`  Head: ${currentState.head?.toString() || 'None'}`);
    console.log(`  Tail: ${currentState.tail?.toString() || 'None'}\n`);

    if (currentState.size === 0) {
      console.log("⚠️  State is already empty - no need to reset");
      console.log("   If you're still seeing errors, the Node accounts may be corrupted");
      console.log("   This script will still close and reinitialize the account\n");
    }
  } catch (error) {
    console.log("⚠️  SortedTrovesState doesn't exist or can't be fetched");
    console.log("   Nothing to clean up - ready for fresh initialization\n");
    return;
  }

  console.log("⚠️  WARNING: This will close the SortedTrovesState account!");
  console.log("   - All trove ordering will be reset");
  console.log("   - Next openTrove will create fresh state");
  console.log("   - Existing troves won't be in the sorted list until they call openTrove again");
  console.log("\n   Press Ctrl+C to cancel, or waiting 3 seconds to proceed...\n");

  await new Promise(resolve => setTimeout(resolve, 3000));

  try {
    console.log("📤 Calling reset_sorted_troves instruction...");
    
    const tx = await protocolProgram.methods
      .resetSortedTroves()
      .accounts({
        sortedTrovesState: sortedTrovesStatePDA,
        state: statePDA,
        authority: admin.publicKey,
      })
      .rpc();

    console.log("✅ Success! Transaction signature:", tx);
    console.log("\nSortedTrovesState account closed successfully!");
    console.log("✅ Next openTrove call will create fresh, empty sorted list");
    console.log("\n💡 You can now run your tests:");
    console.log("   ANCHOR_PROVIDER_URL=https://api.devnet.solana.com \\");
    console.log("   ANCHOR_WALLET=~/.config/solana/id.json \\");
    console.log("   npx ts-mocha -p ./tsconfig.json -t 1000000 'tests/**/protocol-core.ts'\n");

  } catch (error) {
    console.error("❌ Error calling reset_sorted_troves:");
    console.error(error);
    console.log("\n💡 Troubleshooting:");
    console.log("   1. Ensure you've redeployed the program with the reset_sorted_troves instruction:");
    console.log("      anchor build && anchor deploy --provider.cluster devnet");
    console.log("   2. Verify you're using the correct admin wallet (protocol authority)");
    console.log("   3. Check devnet RPC is responsive: solana cluster-version --url devnet\n");
    throw error;
  }
}

main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
  });
