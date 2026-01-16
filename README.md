# Raydium CPI - Anti-Sniping Bot Protection for Raydium Launchpad

A Solana program designed to provide atomic transaction protection against sniping bot attacks on the Raydium Launchpad. This Cross-Program Invocation (CPI) wrapper ensures secure token launches by enforcing atomic execution of token creation and initial purchases.

## 🌐 Live Deployment

**✅ Program Deployed on Devnet**
- **Program ID**: `EXZHGtUuunmGujBhM6WxhCbZEqyN9bsAnsvXTyg1rqF`
- **Network**: Solana Devnet
- **Cluster URL**: `https://api.devnet.solana.com`
- **Deployment Signature**: `5QYAMww6AMzvzgMtyS622qmUp3eSbxd3WrDD4G8fPEK9xuru5QtePPQ4qrJiBiXGcdXukmNyYkCZsg5DmfU6dFFfa`

The program is actively deployed and ready for testing on Solana Devnet. You can interact with it using the Program ID above.

## 🚀 Overview

The Raydium CPI program is an innovative solution that prevents malicious sniping bots from front-running token launches on Raydium's launchpad. By implementing atomic instructions that must be executed together within a single transaction, the program ensures fair token distribution and protects legitimate users from sandwich attacks and front-running.

## 🔧 Key Features

### Atomic Instructions
- **Atomic Token Creation & Purchase**: Combines token launch and initial buy in a single atomic transaction
- **Anti-Sniping Protection**: Prevents bots from inserting transactions between token creation and legitimate purchases
- **Fair Launch Mechanism**: Ensures all participants have equal opportunity during token launches

### Security Features
- **Transaction Validation**: Validates that buy instructions follow create instructions within the same transaction
- **Instruction Sysvar Checking**: Uses Solana's instruction sysvar to verify transaction composition
- **PDA-based Authority**: Secure program-derived addresses for vault operations

## 📋 Program Instructions

### 1. `atomic`
The core anti-sniping instruction that creates a token pool and validates that a buy instruction follows in the same transaction.

**Key Anti-Sniping Logic:**
```rust
// Validates that a buy instruction follows in the same transaction
let current_idx = sysvar::instructions::load_current_index_checked(&ctx.accounts.instructions_sysvar)?;
let buy_token_disc = instruction_discriminator("buy");

// Search for buy instruction in remaining transaction instructions
let mut found_buy_token = false;
let mut instruction_index = current_idx + 1;

loop {
    match sysvar::instructions::load_instruction_at_checked(
        instruction_index as usize,
        &ctx.accounts.instructions_sysvar,
    ) {
        Ok(instruction) => {
            if instruction.program_id == crate::id()
                && instruction.data.len() >= 8
                && instruction.data[0..8] == buy_token_disc
            {
                found_buy_token = true;
                break;
            }
            instruction_index += 1;
        }
        Err(_) => break,
    }
}

require!(found_buy_token, ErrorCode::BuyInstructionMissing);
```

### 2. `create`
Standard token creation instruction for non-atomic launches.

### 3. `buy`
Token purchase instruction that works with both atomic and standard flows.

## 🛡️ How Atomic Instructions Prevent Sniping Bot Attacks

### The Problem: Sniping Bots
Traditional token launches are vulnerable to:
- **Front-running**: Bots detect pending token creation transactions and insert their own buy orders with higher gas fees
- **Sandwich attacks**: Bots place orders before and after legitimate transactions to extract value
- **MEV (Maximal Extractable Value)**: Bots reorder transactions to maximize their profits at users' expense

### The Solution: Atomic Execution
Our atomic instruction approach solves these issues by:

1. **Single Transaction Requirement**: Token creation and initial purchase must occur in the same transaction
2. **Instruction Validation**: The program verifies that buy instructions follow create instructions within the transaction
3. **No Intermediate State**: Prevents bots from inserting transactions between creation and purchase
4. **Deterministic Execution**: Ensures predictable order of operations

### Technical Implementation

The atomic protection works through:

1. **Instruction Sysvar Access**: Uses `Sysvar1nstructions1111111111111111111111111` to read the current transaction's instruction sequence
2. **Discriminator Matching**: Searches for buy instruction discriminators in the remaining transaction instructions
3. **Validation Logic**: Fails the transaction if no buy instruction is found, preventing partial execution
4. **CPI Integration**: Seamlessly integrates with Raydium's launchpad program through cross-program invocations

## 🏗️ Architecture

### Program Structure
```
raydium-cpi/
├── programs/raydium-cpi/src/
│   ├── lib.rs              # Main program entry point
│   ├── constants.rs        # Seed constants for PDAs
│   ├── error.rs           # Custom error definitions
│   └── instructions/
│       ├── atomic.rs      # Anti-sniping atomic instruction
│       ├── create.rs      # Standard token creation
│       ├── buy.rs         # Token purchase instruction
│       └── mod.rs         # Module exports
├── idls/                  # Interface Definition Language files
├── tests/                 # TypeScript tests
└── target/                # Compiled program artifacts
```

### Dependencies
- **Anchor Framework**: `0.31.1` - Solana program development framework
- **anchor-spl**: Token program integrations
- **Raydium Launchpad**: External program for token launch functionality

## 🚀 Quick Start

### Prerequisites
- [Rust](https://rustlang.org/) `1.75.0+`
- [Solana CLI](https://docs.solana.com/cli/install-solana-cli-tools) `1.18.0+`
- [Anchor Framework](https://www.anchor-lang.com/docs/installation) `0.31.1+`
- [Node.js](https://nodejs.org/) `18.0.0+`
- [Yarn](https://yarnpkg.com/) package manager
- [Surfpool](https://github.com/drift-labs/surfpool) - Local Solana test validator with enhanced features

### Installation

1. **Clone the repository**
```bash
git clone https://github.com/prince981620/raydium-cpi.git
cd raydium-cpi
```

2. **Install Surfpool (for local testing)**
```bash
npm install -g @drift-labs/surfpool
```

3. **Install dependencies**
```bash
yarn install
```

4. **Build the program**
```bash
anchor build
```

5. **Start Surfpool and run tests**
```bash
# Start Surfpool in a separate terminal
surfpool

# Run tests with local validator
anchor test
```

### Deployment

1. **Configure your Solana CLI**
```bash
solana config set --url <CLUSTER_URL>
solana config set --keypair <PATH_TO_KEYPAIR>
```

2. **Deploy the program**
```bash
anchor deploy
```

3. **Verify deployment**
```bash
solana program show <PROGRAM_ID>
```

## 📝 Usage Examples

### Atomic Token Launch (Anti-Sniping)

```typescript
import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { RaydiumCpi } from "../target/types/raydium_cpi";

// Create atomic transaction with token creation and purchase
const tx = new anchor.web3.Transaction();

// Add atomic instruction (creates token and validates buy follows)
tx.add(
  await program.methods
    .atomic({
      name: "MyToken",
      symbol: "MTK",
      uri: "https://example.com/metadata.json"
    })
    .accounts({
      creator: creator.publicKey,
      instructionsSysvar: anchor.web3.SYSVAR_INSTRUCTIONS_PUBKEY,
      // ... other accounts
    })
    .instruction()
);

// Add buy instruction (must be in same transaction)
tx.add(
  await program.methods
    .buy(
      new anchor.BN(1000000), // amount_in
      new anchor.BN(950000),  // minimum_amount_out (slippage protection)
      new anchor.BN(100)      // share_fee_rate
    )
    .accounts({
      user: buyer.publicKey,
      // ... other accounts
    })
    .instruction()
);

// Execute atomically - bots cannot insert transactions between create and buy
const signature = await anchor.web3.sendAndConfirmTransaction(
  connection,
  tx,
  [creator, buyer]
);
```

### Standard Token Launch

```typescript
// Standard creation (vulnerable to sniping)
await program.methods
  .create({
    name: "MyToken",
    symbol: "MTK",
    uri: "https://example.com/metadata.json"
  })
  .accounts({
    user: creator.publicKey,
    // ... other accounts
  })
  .rpc();

// Separate buy transaction (bots can front-run this)
await program.methods
  .buy(amountIn, minimumAmountOut, shareFeeRate)
  .accounts({
    user: buyer.publicKey,
    // ... other accounts
  })
  .rpc();
```

## 🔍 Testing

The project includes comprehensive tests covering:

- Atomic instruction validation
- Anti-sniping protection mechanisms
- Integration with Raydium launchpad
- Error handling and edge cases

### Using Surfpool (Recommended)

Surfpool provides an enhanced local Solana test validator with better performance and additional features:

1. **Start Surfpool**
```bash
surfpool start --watch
```

2. **Run tests** (in a separate terminal)
```bash
anchor test
```

### Alternative Testing Methods

For standard Anchor testing:
```bash
anchor test
```

For localnet testing:
```bash
anchor test --provider.cluster localnet
```

### Test Features
- **Surfpool Integration**: Optimized local validator for faster test execution
- **Atomic Transaction Testing**: Validates anti-sniping mechanisms
- **CPI Integration Testing**: Tests cross-program invocations with Raydium
- **Error Scenario Testing**: Ensures proper error handling

## 🛠️ Configuration

### Program Configuration (`Anchor.toml`)

```toml
[programs.localnet]
raydium_cpi = "EXZHGtUuunmGujBhM6WxhCbZEqyN9bsAnsvXTyg1rqF"

[provider]
cluster = "localnet"
wallet = "~/.config/solana/id.json"
```

### Environment Variables
- `ANCHOR_PROVIDER_URL`: Solana cluster endpoint
- `ANCHOR_WALLET`: Path to keypair file

## 🔐 Security Considerations

### Program Security
- **PDA Derivation**: All program-derived addresses use secure seed combinations
- **Account Validation**: Strict validation of all incoming accounts
- **CPI Security**: Secure cross-program invocations with Raydium launchpad

### Anti-Sniping Mechanisms
- **Atomic Execution**: Prevents transaction insertion attacks
- **Instruction Validation**: Ensures proper transaction composition
- **Slippage Protection**: Built-in minimum output amount validation

### Best Practices
- Always use atomic instructions for fair launches
- Implement proper slippage tolerance
- Validate all account relationships
- Monitor for unusual transaction patterns

## 🤝 Contributing

We welcome contributions! Please follow these steps:

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/your-feature`
3. Commit your changes: `git commit -am 'Add some feature'`
4. Push to the branch: `git push origin feature/your-feature`
5. Submit a pull request

### Development Guidelines
- Follow Rust best practices and conventions
- Add tests for new functionality
- Update documentation for API changes
- Ensure all tests pass before submitting

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🔗 Links

- [Raydium Protocol](https://raydium.io/)
- [Solana Documentation](https://docs.solana.com/)
- [Anchor Framework](https://www.anchor-lang.com/)
- [SPL Token Program](https://spl.solana.com/token)

## ⚠️ Disclaimer

This software is provided "as is" without warranty of any kind. Users should conduct their own security audits before deploying to mainnet. The developers are not responsible for any losses incurred through the use of this software.

## 📊 Program Statistics

- **Program ID**: `EXZHGtUuunmGujBhM6WxhCbZEqyN9bsAnsvXTyg1rqF`
- **Raydium Launchpad ID**: `LanMV9sAd7wArD4vJFi2qDdfnVhFxYSUg6eADduJ3uj`
- **Network**: Solana Mainnet/Devnet/Localnet compatible
- **Language**: Rust (Anchor Framework)

---

Built with ❤️ for the Solana ecosystem to ensure fair and secure token launches.