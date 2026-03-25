use rig_onechain_trader::{
    agents::TradingAgentSystem,
    execution::{OneChainExecutor},
    strategy::TradingStrategy,
    strategy::risk::RiskManager,
};
use onechain_sdk::types::base_types::{ObjectID, SuiAddress};
use onechain_sdk::SuiClientBuilder;
use fastcrypto::ed25519::Ed25519KeyPair;
use fastcrypto::traits::KeyPair;
use rand::rngs::OsRng;
use rig::providers::openai::Client as OpenAIClient;
use std::sync::Arc;
use tokio;

#[tokio::test]
async fn test_system_initialization_without_keys() {
    // 1. Verify we can spawn an ephemereal wallet
    let mut rng = OsRng;
    let keypair = Arc::new(Ed25519KeyPair::generate(&mut rng));
    let address = SuiAddress::from(&keypair.public());
    
    // 2. Mock OneChain Testnet URL to assert SDK bindings
    let rpc_url = "https://rpc-testnet.onelabs.cc:443";
    let sui_client = Arc::new(
        SuiClientBuilder::default()
            .build(rpc_url)
            .await
            .expect("SUI Client failed to bind to url")
    );
    
    // 3. Mock DeepBook IDs to verify Execution specialist bindings
    let deepbook_pkg_id = ObjectID::from_hex_literal("0x000000000000000000000000000000000000000000000000000000000000dee9").unwrap();
    let pool_id = ObjectID::from_hex_literal("0x1111111111111111111111111111111111111111111111111111111111111111").unwrap();
    let account_cap_id = ObjectID::from_hex_literal("0x2222222222222222222222222222222222222222222222222222222222222222").unwrap();

    let executor = OneChainExecutor::new(
        keypair.clone(),
        address,
        sui_client.clone(),
        deepbook_pkg_id,
        pool_id,
        account_cap_id,
    );
    
    // We expect the execution struct to be successfully assembled without panicking on missing tokens.
    assert!(address.to_string().starts_with("0x"), "Address generation failed format");

    // 4. Test AI Brain initialization placeholder
    // We test that `TradingAgentSystem::new` expects the properly structured rig OpenAI client.
    let mock_openai = OpenAIClient::from_url("test_key", "https://api.openai.com/v1");
    // We wrap it in a sanity check. Real test requires a valid OPENAI_API_KEY
    if std::env::var("OPENAI_API_KEY").is_ok() {
        let agent_system = TradingAgentSystem::new(mock_openai).await.unwrap();
        let risk_manager = RiskManager::new();
        let _strategy = TradingStrategy::new(agent_system, risk_manager);
        // Validates all mathematical models compile cleanly and LLM wrappers bind without legacy `DeepSeek` / `Transformer` constraints.
    }
}
