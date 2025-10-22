# Staking Address for Aerospacer Fee Contract

## Overview
This directory contains the keypair for the staking/stability pool address used in the Aerospacer fee contract.

## Address Information
- **Public Key:** `CUdX27XaXCGeYLwRVssXE63wufjkufTPXrHqMRCtYaX3`
- **Private Key File:** `staking_address.json`
- **Purpose:** Stability pool address for fee accumulation when stake mode is enabled

## Usage
This address is used as the `stake_contract_address` in the fee distributor contract when `is_stake_enabled` is set to `true`.

## Security Note
- Keep the private key secure and never commit it to version control
- This is for testing purposes only
- In production, this would be a proper staking contract address

## Seed Phrase (for recovery)
```
ticket thrive grocery sphere youth luggage found oxygen promote tiger hospital focus
```

## Testing
Use this address in tests to verify fee distribution to the stability pool functionality.

taha@HP-Probook:~/Documents/Projects/Aeroscraper/aerospacer-solana$ cd /home/taha/Documents/Projects/Aeroscraper/aerospacer-solana
solana-keygen new --no-bip39-passphrase --outfile user1-keypair.json
solana-keygen new --no-bip39-passphrase --outfile user2-keypair.json
Generating a new keypair
Wrote new keypair to user1-keypair.json
=========================================================================
pubkey: 7d4AG2fCbYW9gdmdUe9afUX69UknDmh9MXVXapRt7y56
=========================================================================
Save this seed phrase to recover your new keypair:
fury pioneer truck isolate snow daring word accident ahead buyer rug wise
=========================================================================
Generating a new keypair
Wrote new keypair to user2-keypair.json
============================================================================
pubkey: HdvpoRLcVAtNQgEeH62ypLycCuncGvWCn4DVA8Xgq8FY
============================================================================
Save this seed phrase to recover your new keypair:
opinion business torch pool system shock want clinic fat manage bicycle rose
============================================================================
taha@HP-Probook:~/Documents/Projects/Aeroscraper/aerospacer-solana$ 
