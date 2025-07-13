# CryptoSun Token-2022 Staking Contract

A Solana-based staking contract for the CryptoSun (CSN) Token-2022 ecosystem, designed to incentivize renewable energy infrastructure participation.

## Features

- **Token-2022 Compatibility**: Built for the latest Solana token standard
- **Secure Vault System**: Uses Program Derived Addresses (PDAs) for secure token storage
- **Dynamic Rewards**: Calculates rewards based on solar panel performance metrics
- **Staking & Unstaking**: Complete lifecycle management for user tokens
- **Reward Claiming**: Automated reward distribution based on performance factors

## Reward Formula

Based on the CryptoSun whitepaper, rewards are calculated using:

```
Reward = BaseRate × Stake × (EnergyFactor + UptimeFactor + MaintenanceFactor)
```

Where:
- **BaseRate**: 0.0001 CSN/day per staked CSN
- **EnergyFactor**: kWh/10 (capped at 1.0)
- **UptimeFactor**: uptime percentage / 100
- **MaintenanceFactor**: 1.0 (compliant) or 0.5 (non-compliant)

## Contract Instructions

### 1. `create_vault`
Creates the staking vault PDA token account.

### 2. `initialize_stake`
Stakes user tokens into the vault and records stake information.

### 3. `claim_rewards`
Calculates and distributes rewards based on performance metrics.

### 4. `unstake`
Withdraws staked tokens back to the user's wallet.

## Setup & Installation

### Prerequisites
- Node.js (v16 or higher)
- Rust (latest stable)
- Solana CLI (Agave client recommended)
- Anchor Framework

### Installation
```bash
# Clone the repository
git clone <your-repo-url>
cd staking

# Install dependencies
yarn install

# Build the program
anchor build

# Run tests
anchor test
```

## Testing

The test suite demonstrates the complete staking flow:

1. **Create Vault**: Initialize the staking vault
2. **Stake Tokens**: Lock user tokens in the vault
3. **Claim Rewards**: Calculate and distribute performance-based rewards
4. **Unstake**: Withdraw tokens back to user

Run tests with detailed balance tracking:
```bash
anchor test
```

## Configuration

### Token Details
- **Mint Address**: `45qA6AB2EZa3wUfBGwifw31Qt3iajAwnduLrMMjdcakm`
- **Token Program**: Token-2022 (`TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb`)
- **Decimals**: 9

### Network
- **Devnet**: Configured for Solana Devnet testing
- **Mainnet**: Ready for mainnet deployment

## Architecture

### Key Components
- **Vault PDA**: Secure token storage controlled by the program
- **Stake State**: Per-user account tracking stake amount and timestamps
- **Reward Calculation**: On-chain computation of performance-based rewards
- **Token Transfers**: Secure movement of tokens between user and vault

### Security Features
- PDA-based vault ownership
- Ed25519 signature verification
- Timestamp-based reward calculations
- Owner validation for all operations

## Future Enhancements

- Oracle integration for real-time solar panel data
- Governance controls for parameter updates
- Penalty/slashing mechanisms for non-compliance
- Cross-chain bridge integration
- Advanced reward distribution algorithms

## License

MIT License

Copyright (c) 2024 CryptoSun

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.

## Contributing

[Your Contributing Guidelines Here] 