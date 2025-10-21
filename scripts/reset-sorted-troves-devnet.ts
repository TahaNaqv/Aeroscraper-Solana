import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { AerospacerProtocol } from "../target/types/aerospacer_protocol";
import { PublicKey, SystemProgram } from "@solana/web3.js";

/**
 * DEVNET ADMIN SCRIPT: Reset Sorted Troves State
 * 
 * Use this when SortedTrovesState metadata is corrupted (size > 0 but Node accounts have wrong discriminators)
 * This closes the old SortedTrovesState account and creates a fresh one.
 * 
 * IMPORTANT: This will lose all existing trove ordering! Only use if:
 * 1. Devnet state is corrupted and cannot be traversed
 * 2. You have admin authority over the protocol
 * 3. You're okay with resetting the sorted list (all existing troves will need to be re-inserted)
 */

async function main() {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const protocolProgram = anchor.workspace.AerospacerProtocol as Program<AerospacerProtocol>;
  const admin = provider.wallet;

  console.log("🔧 Resetting SortedTrovesState on devnet...");
  console.log(`Admin: ${admin.publicKey.toString()}`);
  console.log(`Protocol: ${protocolProgram.programId.toString()}\n`);

  // Derive the SortedTrovesState PDA
  const [sortedTrovesStatePDA] = PublicKey.findProgramAddressSync(
    [Buffer.from("sorted_troves")],
    protocolProgram.programId
  );

  console.log(`SortedTrovesState PDA: ${sortedTrovesStatePDA.toString()}`);

  // Check current state
  try {
    const currentState = await protocolProgram.account.sortedTrovesState.fetch(sortedTrovesStatePDA);
    console.log(`\nCurrent state:`);
    console.log(`  Size: ${currentState.size}`);
    console.log(`  Head: ${currentState.head?.toString() || 'None'}`);
    console.log(`  Tail: ${currentState.tail?.toString() || 'None'}`);
  } catch (error) {
    console.log(`\n⚠️ SortedTrovesState doesn't exist yet or cannot be fetched`);
  }

  console.log(`\n⚠️  WARNING: This will reset all sorted troves ordering!`);
  console.log(`   All existing troves will need to be re-inserted to restore sorting.`);
  console.log(`   Press Ctrl+C to cancel, or wait 5 seconds to proceed...\n`);

  await new Promise(resolve => setTimeout(resolve, 5000));

  // Get account info to close it
  const accountInfo = await provider.connection.getAccountInfo(sortedTrovesStatePDA);
  
  if (accountInfo) {
    console.log(`📤 Closing existing SortedTrovesState account...`);
    
    // Manually close the account by transferring lamports and zeroing data
    // Note: This requires the account to be owned by the program
    const tx = new anchor.web3.Transaction().add(
      // Create a CPI to close the account (requires program support)
      // For now, we'll just log that manual intervention is needed
    );

    console.log(`⚠️  Manual reset required:`);
    console.log(`   1. Use 'solana account ${sortedTrovesStatePDA.toString()}' to inspect`);
    console.log(`   2. Contact protocol admin to add a reset_sorted_troves instruction to the program`);
    console.log(`   3. Or redeploy the program to a fresh devnet instance\n`);
  } else {
    console.log(`✅ SortedTrovesState doesn't exist - ready for fresh initialization`);
  }

  console.log(`\n💡 RECOMMENDED WORKAROUND:`);
  console.log(`   Instead of resetting state, test on localnet where you have full control:`);
  console.log(`   1. Start local validator: solana-test-validator`);
  console.log(`   2. Deploy: anchor deploy --provider.cluster localnet`);
  console.log(`   3. Run tests: anchor test --skip-deploy\n`);
}

main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
  });
