# Get Random - WASI Example

> **[Full documentation](https://outlayer.fastnear.com/docs/examples#random-ark)** on the OutLayer dashboard.

Simple WASM binary for testing NEAR OutLayer platform with WASI support.

## Description

Generates a random number in the specified range using `rand` crate with WASI random source.
Reads input from stdin and writes output to stdout.

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
# Add WASI target
rustup target add wasm32-wasip1

# Build
cargo build --release --target wasm32-wasip1

# Output: target/wasm32-wasip1/release/random-ark.wasm (~111KB)
```

## Local Testing

```bash
# Test with wasmtime
echo '{"min":1,"max":100}' | wasmtime target/wasm32-wasip1/release/random-ark.wasm

# Expected output: {"random_number":42}  (some number between 1-100)
```

## Usage with NEAR OutLayer

1. Push this code to a GitHub repository (e.g., https://github.com/zavodil/random-ark)

2. Call `request_execution` on the OffchainVM contract:
```bash
near call outlayer.testnet request_execution '{
  "code_source": {
    "repo": "https://github.com/zavodil/random-ark",
    "commit": "main",
    "build_target": "wasm32-wasip1"
  },
  "resource_limits": {
    "max_instructions": 10000000,
    "max_memory_mb": 128,
    "max_execution_seconds": 60
  },
  "input_data": "{\"min\":1,\"max\":100}"
}' --accountId your-account.testnet --deposit 0.1
```

3. Worker will:
   - Compile the WASM in sandboxed Docker container
   - Execute with wasmi (WASI P1) or wasmtime (WASI P2)
   - Return the random number as readable text (not bytes!)
   - Show result in NEAR explorer

## Unit Tests

```bash
cargo test
```
