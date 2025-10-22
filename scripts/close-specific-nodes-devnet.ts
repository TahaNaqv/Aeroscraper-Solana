import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { AerospacerProtocol } from "../target/types/aerospacer_protocol";
import { PublicKey } from "@solana/web3.js";

async function main() {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const protocolProgram = anchor.workspace.AerospacerProtocol as Program<AerospacerProtocol>;
  const admin = provider.wallet as anchor.Wallet;

  console.log("🧹 Closing specific Node accounts from error logs...");
  console.log("Admin:", admin.publicKey.toString());
  console.log("Protocol:", protocolProgram.programId.toString());
  console.log();

  const [statePDA] = PublicKey.findProgramAddressSync(
    [Buffer.from("state")],
    protocolProgram.programId
  );

  console.log("State PDA:", statePDA.toString());
  console.log();

  // These are the exact Node PDAs from the error logs
  const nodeAddresses = [
    "ELpjGAaB9P6eLVe8aNnCaMzX6t6RoSyEEeRDyFn6R6Rb", // user1 from first error
    "EsjHpf14Yu6aYMsgUSUAyyDRydTJRcuPZg8YxdX2BMXc", // user2 from second error
  ];

  for (const nodeAddressStr of nodeAddresses) {
    try {
      const nodePubkey = new PublicKey(nodeAddressStr);
      
      console.log(`\nChecking Node at ${nodeAddressStr}...`);

      const nodeAccountInfo = await provider.connection.getAccountInfo(nodePubkey);
      
      if (!nodeAccountInfo) {
        console.log(`  ✓ Node doesn't exist (already clean)`);
        continue;
      }

      console.log(`  ⚠️  Node exists with ${nodeAccountInfo.data.length} bytes`);
      console.log(`  Owner: ${nodeAccountInfo.owner.toString()}`);

      try {
        const nodeData = await protocolProgram.account.node.fetch(nodePubkey);
        console.log(`  Current Node data:`);
        console.log(`    ID (user): ${nodeData.id.toString()}`);
        console.log(`    Prev: ${nodeData.prevId?.toString() || 'null'}`);
        console.log(`    Next: ${nodeData.nextId?.toString() || 'null'}`);

        // Now we know the user ID, verify PDA derivation
        const [derivedNodePDA, bump] = PublicKey.findProgramAddressSync(
          [Buffer.from("node"), nodeData.id.toBuffer()],
          protocolProgram.programId
        );
        
        console.log(`  Verification:`);
        console.log(`    Derived PDA: ${derivedNodePDA.toString()}`);
        console.log(`    Actual PDA:  ${nodePubkey.toString()}`);
        console.log(`    Match: ${derivedNodePDA.equals(nodePubkey)}`);
        console.log(`    Bump: ${bump}`);

        // Try to close this node
        console.log(`  📤 Calling close_node instruction...`);

        const tx = await protocolProgram.methods
          .closeNode(nodeData.id)
          .accounts({
            node: nodePubkey,
            state: statePDA,
            authority: admin.publicKey,
            systemProgram: anchor.web3.SystemProgram.programId,
          } as any)
          .rpc();

        console.log(`  ✅ Success! Transaction: ${tx}`);
      } catch (e) {
        console.log(`  ⚠️  Failed to decode/close Node: ${e.message}`);
        if (e.logs) {
          console.log(`  Transaction logs:`, e.logs);
        }
      }
    } catch (error) {
      console.error(`  ❌ Error processing Node ${nodeAddressStr}:`, error.message);
    }
  }

  console.log("\n✅ Cleanup attempt complete!");
  console.log();
  console.log("💡 You can now run your tests:");
  console.log("   ANCHOR_PROVIDER_URL=https://api.devnet.solana.com \\");
  console.log("   ANCHOR_WALLET=~/.config/solana/id.json \\");
  console.log("   npx ts-mocha -p ./tsconfig.json -t 1000000 'tests/**/protocol-core.ts'");
}

main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
  });
