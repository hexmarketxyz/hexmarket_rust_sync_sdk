use crate::client::HexClient;
use crate::error::HexSdkError;
use crate::types::{MergedOrderBook, OrderBook};

impl HexClient {
    /// Get the direct orderbook for an outcome.
    pub fn get_orderbook(&self, outcome_id: &str) -> Result<OrderBook, HexSdkError> {
        self.get(&self.url(&format!("/api/v1/orderbook/{}", outcome_id)))
    }

    /// Get the merged orderbook (direct + cross-outcome synthetic liquidity).
    pub fn get_merged_orderbook(
        &self,
        outcome_id: &str,
    ) -> Result<MergedOrderBook, HexSdkError> {
        self.get(&self.url(&format!("/api/v1/orderbook/{}/merged", outcome_id)))
    }
}
