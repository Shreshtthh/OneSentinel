use mongodb::{
    bson::doc,
    Collection, Database,
};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use crate::strategy::{PortfolioPosition, PartialSell};

#[derive(Debug, Serialize, Deserialize)]
pub struct PositionDocument {
    pub token_address: String,
    pub symbol: String,
    pub name: String,
    pub quantity: f64,
    pub cost_basis_native: f64,
    pub entry_timestamp: i64,
    pub partial_sells: Vec<PartialSell>,
    pub current_price_native: f64,
    pub current_price_usd: f64,
    pub unrealized_pnl_native: f64,
    pub realized_pnl_native: f64,
    pub last_updated: i64,
}

pub struct PositionsCollection {
    collection: Collection<PositionDocument>,
}

impl PositionsCollection {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection("positions"),
        }
    }

    pub async fn upsert_position(&self, position: &PortfolioPosition, current_prices: (f64, f64)) -> Result<String> {
        let (current_price_native, current_price_usd) = current_prices;
        
        let unrealized_pnl = (current_price_native - position.cost_basis_native) * position.quantity;
        let realized_pnl: f64 = position.partial_sells.iter()
            .map(|sell| (sell.price_native - position.cost_basis_native) * sell.quantity)
            .sum();

        let doc = PositionDocument {
            token_address: position.token.address.clone(),
            symbol: position.token.symbol.clone(),
            name: position.token.name.clone(),
            quantity: position.quantity,
            cost_basis_native: position.cost_basis_native,
            entry_timestamp: position.entry_timestamp,
            partial_sells: position.partial_sells.clone(),
            current_price_native,
            current_price_usd,
            unrealized_pnl_native: unrealized_pnl,
            realized_pnl_native: realized_pnl,
            last_updated: chrono::Utc::now().timestamp(),
        };

        let filter = doc! {
            "token_address": &position.token.address
        };

        let update = doc! {
            "$set": mongodb::bson::to_document(&doc)?
        };

        let result = self.collection
            .update_one(filter, update)
            .upsert(true)
            .await?;

        Ok(result.upserted_id
            .map(|id| id.to_string())
            .unwrap_or_else(|| "updated".to_string()))
    }

    pub async fn get_all_positions(&self) -> Result<Vec<PositionDocument>> {
        let mut positions = Vec::new();
        let mut cursor = self.collection.find(doc! {}).await?;

        while cursor.advance().await? {
            positions.push(cursor.deserialize_current()?);
        }

        Ok(positions)
    }

    pub async fn get_position(&self, token_address: &str) -> Result<Option<PositionDocument>> {
        let filter = doc! {
            "token_address": token_address
        };

        Ok(self.collection.find_one(filter).await?)
    }

    pub async fn delete_position(&self, token_address: &str) -> Result<bool> {
        let filter = doc! {
            "token_address": token_address
        };

        let result = self.collection.delete_one(filter).await?;
        Ok(result.deleted_count > 0)
    }

    pub async fn get_portfolio_stats(&self) -> Result<PortfolioStats> {
        let positions = self.get_all_positions().await?;
        
        let mut stats = PortfolioStats {
            total_value_native: 0.0,
            total_value_usd: 0.0,
            total_realized_pnl_native: 0.0,
            total_unrealized_pnl_native: 0.0,
            position_count: positions.len(),
            profitable_positions: 0,
        };

        for pos in positions {
            stats.total_value_native += pos.quantity * pos.current_price_native;
            stats.total_value_usd += pos.quantity * pos.current_price_usd;
            stats.total_realized_pnl_native += pos.realized_pnl_native;
            stats.total_unrealized_pnl_native += pos.unrealized_pnl_native;
            
            if pos.unrealized_pnl_native > 0.0 {
                stats.profitable_positions += 1;
            }
        }

        Ok(stats)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PortfolioStats {
    pub total_value_native: f64,
    pub total_value_usd: f64,
    pub total_realized_pnl_native: f64,
    pub total_unrealized_pnl_native: f64,
    pub position_count: usize,
    pub profitable_positions: usize,
} 