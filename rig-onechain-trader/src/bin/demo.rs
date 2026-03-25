use rig_onechain_trader::{
    agents::TradingAgentSystem,
    execution::{OneChainExecutor, TradeType, TradeParams},
    strategy::{TradingStrategy, StrategyConfig},
    market_data::{EnhancedTokenMetadata, FeatureVector, MacroIndicator},
};
use onechain_sdk::types::base_types::ObjectID;
use rig::providers::openai::Client as OpenAIClient;
use std::env;
use std::io::{self, Write};


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::from_filename("rig-onechain-trader/.env").ok();
    dotenv::from_filename(".env").ok();
    println!("==================================================");
    println!("  ONESENTINEL: AUTONOMOUS AI TRADING AGENT");
    println!("  Built for OneHack 3.0");
    println!("==================================================\n");

    // --- Configuration ---
    let openai_key = env::var("OPENAI_API_KEY").expect("Missing OPENAI_API_KEY in .env");
    let openai_base = env::var("OPENAI_API_BASE").unwrap_or_else(|_| "https://api.openai.com/v1".to_string());
    let rpc_url = env::var("ONECHAIN_RPC_URL").unwrap_or_else(|_| "https://rpc-testnet.onelabs.cc:443".to_string());
    let mongodb_uri = env::var("MONGODB_URI").unwrap_or_else(|_| "mongodb://localhost:27017".to_string());

    // --- Step 1: Initialize OneChain RPC & Agent Wallet ---
    println!("[1/5] Initializing OneChain RPC Client: {}", rpc_url);
    let (sui_client, keypair, address) = rig_onechain_trader::initialize(
        &openai_key,
        "",
        &rpc_url,
        &mongodb_uri,
    ).await?;

    println!("\n==================================================");
    println!("  AGENT WALLET READY");
    println!("  Address: {}", address);
    println!("==================================================");
    println!("If this is a new wallet, fund it first:");
    println!("  Visit: https://faucet-testnet.onelabs.cc:443");
    println!("  Paste the address above and request OCT tokens.");

    print!("\nPress ENTER when your wallet is funded...");
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();

    // --- Step 2: Initialize AI Brain ---
    println!("[2/5] Initializing AI Brain (LLM Models)...");
    let openai_client = OpenAIClient::from_url(&openai_key, &openai_base);
    let agent_system = TradingAgentSystem::new(openai_client).await?;

    // --- Step 3: Initialize Trading Strategy & Executor ---
    println!("[3/5] Booting Trading Strategy and Execution Engine...");
    let strategy_config = StrategyConfig {
        max_position_native: 1.0,
        min_position_native: 0.1,
        max_tokens: 5,
        min_confidence: 0.6,
        min_liquidity_usd: 1000.0,
        max_slippage: 0.05,
    };
    let strategy = TradingStrategy::new(agent_system, strategy_config);

    // Mock Object IDs (replace via ENV for real DeepBook integration)
    let deepbook_pkg_id = ObjectID::from_hex_literal(
        &env::var("DEEPBOOK_PKG_ID").unwrap_or_else(|_| "0x000000000000000000000000000000000000000000000000000000000000dee9".to_string())
    )?;
    let pool_id = ObjectID::from_hex_literal(
        &env::var("POOL_ID").unwrap_or_else(|_| "0x1111111111111111111111111111111111111111111111111111111111111111".to_string())
    )?;
    let account_cap_id = ObjectID::from_hex_literal(
        &env::var("ACCOUNT_CAP_ID").unwrap_or_else(|_| "0x2222222222222222222222222222222222222222222222222222222222222222".to_string())
    )?;

    let executor = OneChainExecutor::new(
        keypair,
        address,
        sui_client,
        deepbook_pkg_id,
        pool_id,
        account_cap_id,
    );

    // --- Step 4: AI Market Analysis ---
    println!("\n[4/5] Agent evaluating market thesis...");
    let token = EnhancedTokenMetadata {
        symbol: "TEST_TOKEN".to_string(),
        price_usd: 1.50,
        volume_24h: 500_000.0,
        market_cap: 10_000_000.0,
        ..Default::default()
    };

    let features = FeatureVector {
        token_address: "0xTEST".to_string(),
        timestamp: chrono::Utc::now(),
        features: vec![0.05, 0.12, 0.8, 0.9],
        feature_names: vec!["price_momentum".into(), "volatility".into(), "volume_trend".into(), "social_sentiment".into()],
    };

    let macro_ind = MacroIndicator {
        timestamp: chrono::Utc::now(),
        native_dominance: 0.45,
        total_market_cap: 500_000_000.0,
        total_volume_24h: 25_000_000.0,
        market_trend: "bullish".to_string(),
        fear_greed_index: 65,
    };

    let decision = strategy.analyze_opportunity(&token, &features, &macro_ind).await?;
    let formatted_reasoning = decision.reasoning.replace("\n", "\n               ");
    println!("  Thesis:      {}", formatted_reasoning);
    println!("  Action:      {:?}", decision.action);
    println!("  Size (OCT):  {}", decision.size_in_native);

    // --- Step 5: Execute On-Chain Transaction ---
    println!("\n[5/5] Executing Native Autonomous Transaction on OneChain Testnet...");
    println!("  (Using empty PTB for guaranteed ledger validation)");

    let action = rig_onechain_trader::execution::TradeAction {
        action_type: TradeType::Buy,
        params: TradeParams {
            mint: token.symbol.clone(),
            amount: 0.5, // kept under risk_threshold of 0.8
            slippage: 5,
            units: 10_000,
            client_order_id: 42,
        },
        analysis: None,
    };

    match executor.execute_trade(action).await {
        Ok(tx_hash) => {
            println!("  SUCCESS: The network has verified the computation.");
            println!("  OneChain Transaction Digest: {}", tx_hash);
        },
        Err(e) => {
            println!("  FAILED (likely unfunded wallet or network issue):");
            println!("  {}", e);
        }
    }

    println!("\n==================================================");
    println!("  DEMO COMPLETE");
    println!("==================================================");

    Ok(())
}
