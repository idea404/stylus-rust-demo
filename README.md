# Arbitrum Stylus Smart Contracts in Rust

A collection of smart contract examples of Rust on Arbitrum Stylus. 

## Why Stylus + Rust?

- **10x-100x faster execution** compared to traditional EVM contracts
- **Native Rust ecosystem access** - use any `no_std` crate from crates.io
- **Familiar tooling** - cargo, clippy, rustfmt, and your favorite IDE
- **Solidity interoperability** - seamlessly interact with existing contracts

## Architecture

This workspace contains three contracts:

### 🧮 [`counter-contract`](./contracts/1-counter-contract/)
**Basic state management and function patterns**

```rust
#[public]
impl Counter {
    pub fn increment(&mut self) {
        let number = self.number.get();
        self.set_number(number + U256::from(1));
    }
    
    #[payable]
    pub fn add_from_msg_value(&mut self) {
        let number = self.number.get();
        self.set_number(number + self.vm().msg_value());
    }
}
```

Demonstrates:
- `sol_storage!` macro for Solidity-compatible storage
- State mutations with `&mut self`
- Payable functions and EVM context access
- Comprehensive unit testing with `TestVM`

### 🏪 [`vending-machine`](./contracts/2-vending-machine/)
**Advanced state management with time-based logic**

```rust
pub fn vend(&mut self) -> Result<(), Vec<u8>> {
    let caller = self.vm().msg_sender();
    let current_time = self.vm().block_timestamp();
    
    // Cooldown enforcement
    if current_time < last_time + VEND_COOLDOWN_SECONDS {
        return Err("Cooldown: Please wait".into());
    }
    // ... circular buffer management
}
```

Demonstrates:
- Complex storage patterns with fixed-size arrays
- Circular buffer implementation for gas optimization
- Time-based business logic
- Error handling with custom messages

### 🔍 [`onchain-data-validator`](./contracts/3-onchain-data-validator/)
**Ecosystem integration with external crates**

```rust
use rust_decimal::Decimal;

pub fn submit_data(&mut self, value: String) -> Result<(), Vec<u8>> {
    let decimal = Decimal::from_str(&value)
        .map_err(|_| "Invalid decimal format".to_string().into_bytes())?;
    
    // Leverage rust_decimal's precision for validation
    if decimal > Decimal::new(1_000_000_000, 0) {
        return Err("Decimal value too large".into());
    }
    // ...
}
```

Demonstrates:
- Integration with external crates (`rust_decimal`)
- String processing and validation
- Advanced error handling patterns
- Complex data type management

## Quick Start

### Prerequisites

```bash
# Install Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Stylus CLI
cargo install cargo-stylus

# Add WebAssembly target
rustup target add wasm32-unknown-unknown
```

### Development Workflow

```bash
# Build all contracts (optimized for size)
cargo build --target wasm32-unknown-unknown --release

# Run contract tests
cargo test

# Check contract for Stylus deployment, e.g. counter-contract
cargo stylus check --wasm-file ./target/wasm32-unknown-unknown/release/counter_contract.wasm

# Export ABI for frontend integration
cd contracts/1-counter-contract
cargo stylus export-abi
```

### Deployment

```bash
# Deploy to Stylus testnet
cargo stylus deploy --private-key=<your-key> --wasm-file ./target/wasm32-unknown-unknown/release/<contract-name>

# Verify deployment
cargo stylus verify --deployment-tx=<tx-hash>
```

## Contributing

This repository follows standard Rust conventions:

```bash
# Format code
cargo fmt

# Lint code
cargo clippy
```
