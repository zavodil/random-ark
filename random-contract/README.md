# Coin Flip Contract

A simple example NEAR contract that uses OutLayer for random number generation.

## How It Works

1. User calls `flip_coin(Heads or Tails)` + attaches 0.01 NEAR
2. Contract calls OutLayer `request_execution` to generate a random number (0 or 1)
3. Worker compiles and executes WASM from https://github.com/out-layer/random-example
4. Contract receives result via callback: `{"random_number": 0}` or `{"random_number": 1}`
5. Compares user's choice with result and returns message


## Demo

- Mainnet: coin-toss.near (uses outlayer.near)
- Testnet: coin-toss.testnet (uses outlayer.testnet)
- UI to play: https://app.outlayer.ai/playground

## Build

```bash
cargo near build non-reproducible-wasm
```

WASM: `target/wasm32-unknown-unknown/release/random_contract.wasm`

## Deploy

```bash
near contract deploy coin-toss.testnet \
  use-file target/wasm32-unknown-unknown/release/random_contract.wasm \
  with-init-call new \
  json-args '{}' \
  prepaid-gas '100.0 Tgas' \
  attached-deposit '0 NEAR' \
  network-config testnet \
  sign-with-keychain \
  send
```

## Usage

### Flip Heads

```bash
near contract call-function as-transaction coin-toss.testnet flip_coin \
  json-args '{"choice": "Heads"}' \
  prepaid-gas '300.0 Tgas' \
  attached-deposit '0.01 NEAR' \
  sign-as alice.testnet \
  network-config testnet \
  sign-with-keychain \
  send
```

### Flip Tails

```bash
near contract call-function as-transaction coin-toss.testnet flip_coin \
  json-args '{"choice": "Tails"}' \
  prepaid-gas '300.0 Tgas' \
  attached-deposit '0.01 NEAR' \
  sign-as zavodil.testnet \
  network-config testnet \
  sign-with-keychain \
  send
```

## Example Logs

### Successful flip (won):

```
Log [coinflip.testnet]: 🎲 Player alice.testnet flips coin: Heads
Log [coinflip.testnet]: 📤 Requesting random number from OutLayer
Log [outlayer.testnet]: EVENT_JSON:{"event":"execution_requested",...}
Log [coinflip.testnet]: ✅ Received random result: {"random_number":0}
Log [coinflip.testnet]: 🎲 Random number: 0
Log [coinflip.testnet]: 🎉 Player alice.testnet WON! Choice: Heads, Result: Heads

Result: "🎉 Congratulations! You won! Result: Heads, Your choice: Heads"
```

### Lost:

```
Log [coinflip.testnet]: 🎲 Player alice.testnet flips coin: Heads
Log [coinflip.testnet]: 📤 Requesting random number from OutLayer
Log [outlayer.testnet]: EVENT_JSON:{"event":"execution_requested",...}
Log [coinflip.testnet]: ✅ Received random result: {"random_number":1}
Log [coinflip.testnet]: 🎲 Random number: 1
Log [coinflip.testnet]: 😢 Player alice.testnet LOST. Choice: Heads, Result: Tails

Result: "😢 Sorry, you lost. Result: Tails, Your choice: Heads. Better luck next time!"
```

## Architecture

```
Player
  ↓ flip_coin(Heads) + 0.01 NEAR
CoinFlipContract (coin-toss.testnet or coin-toss.near)
  ↓ request_execution() + 0.01 NEAR (hardcoded params)
OutLayer Contract (outlayer.testnet or outlayer.near)
  ├─ promise_yield_create (pause)
  └─ Emit event
     ↓
Worker
  ├─ Compilation: github.com/out-layer/random-example (hardcoded)
  ├─ Execution: {"min":0, "max":1} (hardcoded)
  └─ Result: {"random_number": 0 or 1}
     ↓
OutLayer
  └─ promise_yield_resume
     ↓
CoinFlipContract
  └─ on_random_result() callback
     ├─ Parse {"random_number": X}
     ├─ 0 = Heads, 1 = Tails
     └─ Compare with player's choice
        ├─ Match: 🎉 "You won!"
        └─ No match: 😢 "You lost!"
```

## Contract Code (simplified)

```rust
const MIN_DEPOSIT: u128 = 10_000_000_000_000_000_000_000; // 0.01 NEAR

#[payable]
pub fn flip_coin(&mut self, choice: CoinSide) -> Promise {
    // Hardcoded parameters
    let args = json!({
        "code_source": {
            "repo": "https://github.com/out-layer/random-example",
            "commit": "main",
            "build_target": "wasm32-wasip1"
        },
        "resource_limits": {
            "max_instructions": 10000000000,
            "max_memory_mb": 128,
            "max_execution_seconds": 60
        },
        "input_data": "{\"min\":0,\"max\":1}",
        "response_format": "Json",
        "payer_account_id": Some(player.clone())
    });

    // Call OutLayer (outlayer.testnet for testnet, outlayer.near for mainnet)
    ext_outlayer::ext("outlayer.testnet".parse().unwrap())
        .with_attached_deposit(NearToken::from_yoctonear(attached))
        .with_unused_gas_weight(1)
        .request_execution(code_source, resource_limits, input_data, None, "Json".to_string(), Some(player.clone()))
        .then(
            ext_self::ext(env::current_account_id())
                .with_static_gas(Gas::from_gas(5_000_000_000_000))
                .on_random_result(player, choice)
        )
}

#[private]
pub fn on_random_result(
    choice: CoinSide,
    #[callback_result] result: Result<Option<RandomResponse>, PromiseError>,
) -> String {
    match result {
        Ok(Some(response)) => {
            let result_side = if response.random_number == 0 { Heads } else { Tails };

            if choice == result_side {
                "🎉 Congratulations! You won!"
            } else {
                "😢 Sorry, you lost. Better luck next time!"
            }
        }
        Ok(None) => "Error: OutLayer execution failed",
        Err(_) => "Error: Promise failed"
    }
}
```

## Key Features

✅ **All parameters hardcoded** (repo, commit, resource_limits, OutLayer contract ID)
✅ **One constant**: `MIN_DEPOSIT = 0.01 NEAR`
✅ **One method**: `flip_coin(choice)`
✅ **Cross-contract call** with yield/resume
✅ **Callback handling** of result
✅ **Error handling** (Ok(None), Err)
✅ **Refund to player** via `payer_account_id` parameter
