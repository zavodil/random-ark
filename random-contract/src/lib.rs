mod types;

use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::{env, ext_contract, log, near_bindgen, AccountId, NearToken, Promise, PromiseError};

use types::{CoinSide, RandomResponse};

/// Minimum deposit to cover OutLayer execution cost
const MIN_DEPOSIT: u128 = 10_000_000_000_000_000_000_000; // 0.01 NEAR

/// Fixed gas for callback
const CALLBACK_GAS: u64 = 5_000_000_000_000; // 5 TGas

/// OutLayer contract ID (hardcoded)
/// For testnet: "outlayer.testnet"
/// For mainnet: "outlayer.near"
const OUTLAYER_CONTRACT_ID: &str = "outlayer.near";

/// External contract interface for OutLayer
#[ext_contract(ext_outlayer)]
#[allow(dead_code)]
trait OutLayer {
    fn request_execution(
        &mut self,
        code_source: near_sdk::serde_json::Value,
        resource_limits: near_sdk::serde_json::Value,
        input_data: String,
        secrets_ref: Option<near_sdk::serde_json::Value>,
        response_format: String,
        payer_account_id: Option<AccountId>,
    );
}

/// External contract interface for self callbacks
#[ext_contract(ext_self)]
#[allow(dead_code)]
trait ExtSelf {
    fn on_random_result(
        &mut self,
        player: AccountId,
        choice: CoinSide,
        #[callback_result] result: Result<Option<RandomResponse>, PromiseError>,
    ) -> String;
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, Default)]
#[borsh(crate = "near_sdk::borsh")]
pub struct CoinFlipContract {}

#[near_bindgen]
impl CoinFlipContract {
    /// Initialize the contract
    #[init]
    pub fn new() -> Self {
        Self {}
    }

    /// Flip a coin and guess the result
    ///
    /// # Arguments
    /// * `choice` - Your prediction: Heads or Tails
    ///
    /// # Payment
    /// Attach 0.01 NEAR to pay for OffchainVM execution
    ///
    /// # Returns
    /// Promise that will resolve with win/lose message
    #[payable]
    pub fn flip_coin(&mut self, choice: CoinSide) -> Promise {
        let player = env::predecessor_account_id();
        let attached = env::attached_deposit().as_yoctonear();

        assert!(
            attached >= MIN_DEPOSIT,
            "Minimum deposit is 0.01 NEAR to pay for OutLayer execution"
        );

        log!("Player {} chose {:?}. Requesting random number from OutLayer", player, choice);

        // Hardcoded parameters
        let code_source = near_sdk::serde_json::json!({
            "repo": "https://github.com/zavodil/random-ark",
            "commit": "main",
            "build_target": "wasm32-wasip1"
        });

        let resource_limits = near_sdk::serde_json::json!({
            "max_instructions": 10000000000u64,
            "max_memory_mb": 128u32,
            "max_execution_seconds": 60u64
        });

        // Call OutLayer using ext_contract
        // Use with_unused_gas_weight to allocate all remaining gas to request_execution
        // and reserve fixed CALLBACK_GAS for callback
        // Pass player as payer_account_id so refund goes to player, not this contract
        ext_outlayer::ext(OUTLAYER_CONTRACT_ID.parse().unwrap())
            .with_attached_deposit(NearToken::from_yoctonear(attached))
            .with_unused_gas_weight(1) // All unused gas goes to request_execution
            .request_execution(
                code_source,
                resource_limits,
                "{\"min\":0,\"max\":1}".to_string(),
                None,
                "Json".to_string(),
                Some(player.clone()), // Refund to player, not this contract
            )
            .then(
                ext_self::ext(env::current_account_id())
                    .with_static_gas(near_sdk::Gas::from_gas(CALLBACK_GAS))
                    .on_random_result(player, choice),
            )
    }

    /// Callback to handle random number result
    ///
    /// Expected input:
    /// - Ok(Some(RandomResponse{random_number})) - Success with random number
    /// - Ok(None) - Execution failed (contract error, compilation failed, etc.)
    /// - Err(_) - Promise system error (should never happen)
    #[private]
    pub fn on_random_result(
        &mut self,
        player: AccountId,
        choice: CoinSide,
        #[callback_result] result: Result<Option<RandomResponse>, PromiseError>,
    ) -> String {
        match result {
            // Success case: We received Some(RandomResponse)
            Ok(Some(random_response)) => {
                log!("✅ Received random_response from OutLayer: {:?}", random_response);

                let random_number = random_response.random_number;
                log!("🎲 Random number: {}", random_number);

                // Convert to CoinSide (0 = Heads, 1 = Tails)
                let result_side = if random_number == 0 {
                    CoinSide::Heads
                } else {
                    CoinSide::Tails
                };

                // Check if player won
                if choice == result_side {
                    log!(
                        "🎉 Player {} WON! Choice: {:?}, Result: {:?}",
                        player,
                        choice,
                        result_side
                    );

                    format!(
                        "🎉 Congratulations! You won! Result: {:?}, Your choice: {:?}",
                        result_side,
                        choice
                    )
                } else {
                    log!(
                        "😢 Player {} LOST. Choice: {:?}, Result: {:?}",
                        player,
                        choice,
                        result_side
                    );

                    format!(
                        "😢 Sorry, you lost. Result: {:?}, Your choice: {:?}. Better luck next time!",
                        result_side,
                        choice
                    )
                }
            }

            // Failure case: OutLayer returned None (execution failed, no panic)
            Ok(None) => {
                log!("❌ OutLayer execution failed - received None");
                env::panic_str("OutLayer execution failed")
            }

            // Promise error: This should never happen in normal operation
            Err(promise_error) => {
                log!("❌ Promise system error: {:?}", promise_error);
                env::panic_str(&format!("Promise system error: {:?}", promise_error))
            }
        }
    }
}
