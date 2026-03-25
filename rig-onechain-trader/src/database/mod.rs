//! Database Module for Rig OneChain Trader
//!
//! This module handles all MongoDB interactions for the trading bot. It manages:
//!
//! # Collections
//! - `token_states`: Historical market data for tokens
//!   - Indexed by: token address, timestamp
//!   - Contains: price, volume, market cap, changes
//!
//! - `positions`: Active trading positions
//!   - Indexed by: token address
//!   - Contains: entry price, quantity, partial sells
//!
//! # Configuration
//! Database connection is configured through:
//! - `DATABASE_URL`: MongoDB connection string
//! - Database name: onechain_trader

pub mod positions;
pub mod sync;

use mongodb::{Client as MongoClient, Database};
use mongodb::options::{IndexOptions, ClientOptions};
use mongodb::bson::{doc, Document};
use mongodb::IndexModel; // Added back IndexModel as it's used
use anyhow::Result;
use tracing::{info, debug};
use chrono::Utc;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenAnalysis {
    pub token_address: String,
    pub sentiment_score: f64,
    pub technical_score: f64,
    pub risk_score: f64,
    pub symbol: String,
    pub description: String,
    pub recent_events: Vec<String>,
    pub market_sentiment: String,
}

const POSITIONS_COLLECTION: &str = "positions";
const TOKEN_STATES_COLLECTION: &str = "token_states";
const TRADE_HISTORY_COLLECTION: &str = "trade_history";
const SOCIAL_INTERACTIONS_COLLECTION: &str = "social_interactions";
const TOKEN_ANALYSIS_COLLECTION: &str = "token_analysis";

#[derive(Debug)]
#[derive(Clone)]
pub struct DatabaseClient {
    pub db: Arc<Database>,
}

impl DatabaseClient {
    pub async fn new(connection_string: &str, database_name: &str) -> Result<Self> {
        debug!("Initializing MongoDB client with database: {}", database_name);
        
        let mut client_options = ClientOptions::parse(connection_string).await?;
        client_options.app_name = Some("rig-onechain-trader".to_string());
        
        let client = MongoClient::with_options(client_options)?;
        let db = client.database(database_name);
        
        info!("MongoDB client initialized successfully");
        Ok(Self { db: Arc::new(db) })
    }

    pub async fn initialize_collections(&self) -> Result<()> {
        info!("Initializing MongoDB collections and indexes...");

        // Positions collection indexes
        self.create_positions_indexes().await?;
        info!("Positions collection indexes created");

        // Token states collection indexes
        self.create_token_states_indexes().await?;
        info!("Token states collection indexes created");

        // Trade history collection indexes
        self.create_trade_history_indexes().await?;
        info!("Trade history collection indexes created");

        // Social interactions collection indexes
        self.create_social_indexes().await?;
        info!("Social interactions collection indexes created");

        // Token analysis collection indexes
        self.create_token_analysis_indexes().await?;
        info!("Token analysis collection indexes created");

        Ok(())
    }

    async fn create_positions_indexes(&self) -> Result<()> {
        let collection = self.db.collection::<Document>(POSITIONS_COLLECTION);
        
        // Unique index on token address
        let address_index = IndexModel::builder()
            .keys(doc! { "token.address": 1 })
            .options(IndexOptions::builder().unique(true).build())
            .build();

        // Index on entry timestamp
        let timestamp_index = IndexModel::builder()
            .keys(doc! { "entry_timestamp": -1 })
            .build();

        collection.create_index(address_index).await?;
        collection.create_index(timestamp_index).await?;

        debug!("Created indexes for positions collection");
        Ok(())
    }

    async fn create_token_states_indexes(&self) -> Result<()> {
        let collection = self.db.collection::<Document>(TOKEN_STATES_COLLECTION);
        
        // Compound index on address and timestamp
        let address_time_index = IndexModel::builder()
            .keys(doc! { "address": 1, "timestamp": -1 })
            .build();

        // Index on market cap for quick sorting
        let market_cap_index = IndexModel::builder()
            .keys(doc! { "market_cap": -1 })
            .build();

        collection.create_index(address_time_index).await?;
        collection.create_index(market_cap_index).await?;

        debug!("Created indexes for token states collection");
        Ok(())
    }

    async fn create_trade_history_indexes(&self) -> Result<()> {
        let collection = self.db.collection::<Document>(TRADE_HISTORY_COLLECTION);
        
        // Index on timestamp
        let timestamp_index = IndexModel::builder()
            .keys(doc! { "timestamp": -1 })
            .build();

        // Compound index on token address and timestamp
        let token_time_index = IndexModel::builder()
            .keys(doc! { "token_address": 1, "timestamp": -1 })
            .build();

        collection.create_index(timestamp_index).await?;
        collection.create_index(token_time_index).await?;

        debug!("Created indexes for trade history collection");
        Ok(())
    }

    async fn create_social_indexes(&self) -> Result<()> {
        let collection = self.db.collection::<Document>(SOCIAL_INTERACTIONS_COLLECTION);
        
        // Index on timestamp
        let timestamp_index = IndexModel::builder()
            .keys(doc! { "timestamp": -1 })
            .build();

        // Index on interaction type
        let type_index = IndexModel::builder()
            .keys(doc! { "interaction_type": 1 })
            .build();

        // Compound index on user and timestamp
        let user_time_index = IndexModel::builder()
            .keys(doc! { "user_id": 1, "timestamp": -1 })
            .build();

        collection.create_index(timestamp_index).await?;
        collection.create_index(type_index).await?;
        collection.create_index(user_time_index).await?;

        debug!("Created indexes for social interactions collection");
        Ok(())
    }

    async fn create_token_analysis_indexes(&self) -> Result<()> {
        let collection = self.db.collection::<Document>(TOKEN_ANALYSIS_COLLECTION);
        
        // Index on token address
        let address_index = IndexModel::builder()
            .keys(doc! { "token_address": 1 })
            .options(IndexOptions::builder().unique(true).build())
            .build();

        // Index on timestamp
        let timestamp_index = IndexModel::builder()
            .keys(doc! { "timestamp": -1 })
            .build();

        // Index on embedding vector
        let vector_index = IndexModel::builder()
            .keys(doc! { "embedding": 1 })
            .build();

        collection.create_index(address_index).await?;
        collection.create_index(timestamp_index).await?;
        collection.create_index(vector_index).await?;

        debug!("Created indexes for token analysis collection");
        Ok(())
    }

    pub fn positions(&self) -> positions::PositionsCollection {
        positions::PositionsCollection::new(&self.db)
    }

    pub async fn insert_one<T>(&self, collection_name: &str, document: &T) -> Result<()> 
    where 
        T: serde::Serialize + Send + Sync
    {
        debug!("Inserting document into collection: {}", collection_name);
        
        self.db.collection(collection_name)
            .insert_one(mongodb::bson::to_document(document)?)
            .await?;
            
        debug!("Document inserted successfully");
        Ok(())
    }

    pub async fn find_one<T>(&self, collection_name: &str, filter: Document) -> Result<Option<T>>
    where
        T: for<'de> serde::Deserialize<'de> + Send + Sync
    {
        debug!("Finding document in collection: {} with filter: {:?}", collection_name, filter);
        
        let result = self.db.collection(collection_name)
            .find_one(filter)
            .await?;
            
        if result.is_some() {
            debug!("Document found");
        } else {
            debug!("No document found");
        }
        
        Ok(result)
    }

    pub async fn save_token_analysis(&self, analysis: &TokenAnalysis, embedding: Vec<f32>) -> Result<()> {
        let collection = self.db.collection::<Document>(TOKEN_ANALYSIS_COLLECTION);
        
        let doc = doc! {
            "token_address": &analysis.token_address,
            "symbol": &analysis.symbol,
            "description": &analysis.description,
            "recent_events": &analysis.recent_events,
            "market_sentiment": &analysis.market_sentiment,
            "embedding": embedding,
            "timestamp": Utc::now().timestamp(),
        };

        collection.insert_one(doc).await?;
        debug!("Saved token analysis for {}", analysis.symbol);
        Ok(())
    }

    pub async fn get_token_analysis(&self, token_address: &str) -> Result<Option<(TokenAnalysis, Vec<f32>)>> {
        let collection = self.db.collection::<Document>(TOKEN_ANALYSIS_COLLECTION);
        let filter = doc! { "token_address": token_address };
        let _result = collection.find_one(filter).await?;
        Ok(None)
    }
}