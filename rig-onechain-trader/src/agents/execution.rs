use rig_core::{
    agent::Agent,
    message_bus::{Message, MessageBus},
    storage::VectorStorage,
};
use crate::{personality::StoicPersonality};
use std::sync::Arc;

pub struct ExecutionAgent {
    bus: MessageBus,
    storage: Arc<dyn VectorStorage>,
    personality: Arc<StoicPersonality>,
}

impl ExecutionAgent {
    pub fn new(
        bus: MessageBus,
        storage: Arc<dyn VectorStorage>,
        personality: Arc<StoicPersonality>,
    ) -> Self {
        Self { bus, storage, personality }
    }
}

#[async_trait]
impl Agent for ExecutionAgent {
    async fn run(&self) -> anyhow::Result<()> {
        let mut receiver = self.bus.subscribe("trade_decisions");
        
        while let Ok(msg) = receiver.recv().await {
            if let Message::TradeDecision(decision) = msg {
                let sig: String = "dummy_signature".to_string(); // self.personality.execute_trade(&decision).await?;
                
                // Store execution record
                let execution = crate::execution::TradeAction {
                    action_type: crate::execution::TradeType::Buy,
                    params: crate::execution::TradeParams {
                        mint: decision.mint.clone(),
                        amount: decision.amount,
                        slippage: 10,
                        units: 1,
                        client_order_id: 0,
                    },
                    analysis: None,
                };
                
                self.storage
                    .insert("trade_history", execution.clone())
                    .await?;

                self.bus.publish(Message::TradeExecuted(execution)).await;
            }
        }
        Ok(())
    }
} 