# Test WASM - Random Number Generator

Simple WASM module for testing NEAR Offshore platform.

## Description

Generates a random number in the specified range using `getrandom` library.

## Input Format

```json
{
  "min": 0,
  "max": 100
}
```

## Output Format

```json
{
  "random_number": 42
}
```

## Building

```bash
# Add WASM target
rustup target add wasm32-unknown-unknown

# Build
cargo build --release --target wasm32-unknown-unknown

# Output will be at:
# target/wasm32-unknown-unknown/release/test_wasm.wasm
```

## Usage with NEAR Offshore

1. Push this code to a GitHub repository
2. Call `request_execution` on the OffchainVM contract with:
   - `repo`: Your GitHub repo URL
   - `commit`: Git commit hash
   - `build_target`: `"wasm32-unknown-unknown"`
   - Input data with min/max range

3. Worker will:
   - Compile the WASM
   - Execute it with your input
   - Return the random number result to your callback

## Testing Locally

```bash
cargo test
```
