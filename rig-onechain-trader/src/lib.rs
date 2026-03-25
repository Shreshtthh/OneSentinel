//! # OneSentinel: Autonomous AI Trading Agent for OneChain
//!
//! This bot uses LLM-powered analysis to make trading decisions on OneChain tokens.
//! It combines market data from multiple sources, technical analysis, and stoic
//! principles to execute trades via native SUI Programmable Transaction Blocks (PTBs).
//!
//! # Key Features
//! - Real-time market data aggregation from BirdEye API
//! - LLM-powered trading analysis with stoic personality
//! - Automated trade execution via OneChain PTBs (DeepBook-ready for Mainnet)
//! - Headless programmatic wallet using fastcrypto Ed25519
//! - Position tracking and portfolio management
//! - MongoDB persistence for market data and positions
//!
//! # Architecture
//! The bot consists of several key components:
//! - `agents`: Multi-agent LLM system (Market Analyst, Risk Manager, Execution Specialist)
//! - `market_data`: Interfaces with BirdEye API for token data
//! - `strategy`: Implements trading logic and LLM analysis
//! - `execution`: Constructs and signs OneChain PTBs
//! - `personality`: Handles stoic tweet generation
//! - `database`: Handles MongoDB persistence

use rig::providers::openai::Client as OpenAIClient;
// Using standard anyhow::Result since rig::Result might not be exported
use anyhow::Result;
use std::sync::Arc;
use onechain_sdk::SuiClientBuilder;
use onechain_sdk::SuiClient;
use fastcrypto::ed25519::Ed25519KeyPair;
use onechain_sdk::types::base_types::SuiAddress;
use mongodb::Client as MongoDBClient;
use tracing::{info, debug, Level};
use tracing_subscriber::{FmtSubscriber, EnvFilter};
use std::time::Duration;

pub mod agents;
pub mod market_data;
pub mod strategy;
pub mod database;
pub mod dex;
pub mod personality;
pub mod twitter;
pub mod analysis;
pub mod execution;


// Constants for rate limiting
const TRADE_COOLDOWN: Duration = Duration::from_secs(60); // 1 minute between trades
const TWEET_COOLDOWN: Duration = Duration::from_secs(300); // 5 minutes between tweets
const RESPONSE_COOLDOWN: Duration = Duration::from_secs(60); // 1 minute between responses

pub async fn initialize(
    openai_api_key: &str,
    _twitter_bearer_token: &str,
    onechain_rpc_url: &str,
    mongodb_uri: &str,
) -> Result<(Arc<SuiClient>, Arc<Ed25519KeyPair>, SuiAddress)> {
    // Initialize logging
    let _subscriber = FmtSubscriber::builder()
        .with_env_filter(EnvFilter::from_default_env()
            .add_directive(Level::INFO.into())
            .add_directive("rig_onechain_trader=debug".parse()?))
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_file(true)
        .with_line_number(true)
        .with_target(true)
        .with_level(true)
        .with_ansi(true)
        .init();

    info!("Initializing OneChain trading bot...");

    // Initialize MongoDB connection
    let _db_client = MongoDBClient::with_uri_str(mongodb_uri)
        .await
        .map_err(|e| {
            debug!("MongoDB connection error: {:?}", e);
            e
        })?;

    info!("MongoDB connection established");

    // Initialize OpenAI client
    let _openai_client = OpenAIClient::new(openai_api_key);
    debug!("OpenAI client initialized");

    // Twitter initialization removed for standard refactor compilation
    debug!("Twitter client skipped");

    // Initialize OneChain (Sui) client
    let sui_client = SuiClientBuilder::default()
        .build(onechain_rpc_url)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to construct SuiClient: {:?}", e))?;
    debug!("OneChain RPC client initialized. Natively connected.");

    // Wallet: load from mnemonic env var or generate new with mnemonic output
    // Uses SLIP-0010 derivation path m/44'/784'/0'/0'/0' (Sui standard)
    let derive_keypair_from_seed = |seed: &[u8]| -> Result<Ed25519KeyPair> {
        let derivation_path: [u32; 5] = [
            44 | 0x8000_0000,   // purpose (hardened)
            784 | 0x8000_0000,  // coin_type for Sui (hardened)
            0 | 0x8000_0000,    // account (hardened)
            0 | 0x8000_0000,    // change (hardened)
            0 | 0x8000_0000,    // address_index (hardened)
        ];
        let derived = slip10_ed25519::derive_ed25519_private_key(seed, &derivation_path);
        use fastcrypto::traits::ToFromBytes;
        let sk = fastcrypto::ed25519::Ed25519PrivateKey::from_bytes(&derived)
            .map_err(|e| anyhow::anyhow!("Key derivation failed: {}", e))?;
        Ok(Ed25519KeyPair::from(sk))
    };

    let keypair = if let Ok(mnemonic_phrase) = std::env::var("AGENT_MNEMONIC") {
        use bip39::{Mnemonic, Language, Seed};
        let mnemonic = Mnemonic::from_phrase(mnemonic_phrase.trim(), Language::English)
            .map_err(|e| anyhow::anyhow!("Invalid mnemonic: {:?}", e))?;
        let seed = Seed::new(&mnemonic, "");
        let kp = derive_keypair_from_seed(seed.as_bytes())?;
        info!("Loaded agent wallet from AGENT_MNEMONIC");
        kp
    } else {
        use bip39::{Mnemonic, MnemonicType, Language, Seed};
        let mnemonic = Mnemonic::new(MnemonicType::Words12, Language::English);
        let seed = Seed::new(&mnemonic, "");
        let kp = derive_keypair_from_seed(seed.as_bytes())?;
        let phrase = mnemonic.phrase().to_string();
        info!("=== NEW AGENT WALLET CREATED ===");
        info!("Mnemonic (SAVE THIS): {}", phrase);
        info!("To reuse this wallet, add to .env:");
        info!("AGENT_MNEMONIC={}", phrase);
        info!("================================");
        kp
    };
    use fastcrypto::traits::KeyPair as _;
    let pk = onechain_sdk::types::crypto::PublicKey::Ed25519(keypair.public().into());
    let address = SuiAddress::from(&pk);
    info!("Agent SuiAddress: {}", address);

    info!("Initialization complete");
    Ok((
        Arc::new(sui_client),
        Arc::new(keypair),
        address
    ))
}

pub fn get_rate_limits() -> (Duration, Duration, Duration) {
    (TRADE_COOLDOWN, TWEET_COOLDOWN, RESPONSE_COOLDOWN)
}

#[cfg(test)]
mod tests {
    use super::*;
    use dotenv::dotenv;
    use std::env;

    #[tokio::test]
    async fn test_initialization() {
        dotenv().ok();

        let openai_api_key = env::var("OPENAI_API_KEY").unwrap_or_else(|_| "dummy_key".to_string());
        let twitter_bearer_token = env::var("TWITTER_BEARER_TOKEN").unwrap_or_else(|_| "dummy_token".to_string());
        // Since test doesn't actually hit OneChain natively unless setup, we might skip depending on actual endpoint in unit tests
        // or ensure env var is set if testing integration
        let onechain_rpc_url = env::var("ONECHAIN_RPC_URL").unwrap_or_else(|_| "https://rpc-testnet.onelabs.cc:443".to_string());
        let mongodb_uri = env::var("MONGODB_URI").unwrap_or_else(|_| "mongodb://localhost:27017".to_string());

        let result = initialize(
            &openai_api_key,
            &twitter_bearer_token,
            &onechain_rpc_url,
            &mongodb_uri,
        ).await;

        assert!(result.is_ok(), "Initialization failed: {:?}", result.err());
    }
}

// ... rest of the file ... 