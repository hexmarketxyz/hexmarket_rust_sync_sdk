use crate::client::HexClient;
use crate::error::HexSdkError;
use crate::types::{Order, PlaceOrderParams, PlaceOrderResponse};

impl HexClient {
    /// Place a new order (requires L2 auth).
    pub fn place_order(&self, params: &PlaceOrderParams) -> Result<PlaceOrderResponse, HexSdkError> {
        let path = "/api/v1/orders";
        let body = serde_json::to_value(params)
            .map_err(|e| HexSdkError::Other(e.to_string()))?;
        self.post_auth(path, &body)
    }

    /// Cancel an order by ID (requires L2 auth).
    pub fn cancel_order(&self, order_id: &str) -> Result<serde_json::Value, HexSdkError> {
        let path = format!("/api/v1/orders/{}", order_id);
        self.delete_auth(&path)
    }

    /// Cancel all open orders, optionally filtered by market or event (requires L2 auth).
    pub fn cancel_all_orders(
        &self,
        market_id: Option<&str>,
        event_id: Option<&str>,
    ) -> Result<serde_json::Value, HexSdkError> {
        let mut path = "/api/v1/orders".to_string();
        let mut params = Vec::new();
        if let Some(mid) = market_id {
            params.push(format!("market_id={}", mid));
        }
        if let Some(eid) = event_id {
            params.push(format!("event_id={}", eid));
        }
        if !params.is_empty() {
            path.push('?');
            path.push_str(&params.join("&"));
        }
        self.delete_auth(&path)
    }

    /// Place multiple orders in a single batch (requires L2 auth).
    /// All orders must belong to the same market.
    pub fn batch_place_orders(
        &self,
        market_id: &str,
        orders: &[PlaceOrderParams],
    ) -> Result<serde_json::Value, HexSdkError> {
        let path = "/api/v1/orders/batch";
        let body = serde_json::json!({
            "market_id": market_id,
            "orders": orders,
        });
        self.post_auth(path, &body)
    }

    /// Cancel multiple orders in a single batch (requires L2 auth).
    /// All orders must belong to the same market.
    pub fn batch_cancel_orders(
        &self,
        market_id: &str,
        order_ids: &[&str],
    ) -> Result<serde_json::Value, HexSdkError> {
        let path = "/api/v1/orders/batch";
        let body = serde_json::json!({
            "market_id": market_id,
            "order_ids": order_ids,
        });
        self.delete_auth_with_body(path, &body)
    }

    /// List open orders for the authenticated user (requires L2 auth).
    pub fn get_open_orders(
        &self,
        outcome_id: Option<&str>,
    ) -> Result<Vec<Order>, HexSdkError> {
        let pubkey = self.require_pubkey()?;
        let mut path = format!("/api/v1/orders?user={}&status=open", pubkey);
        if let Some(oid) = outcome_id {
            path.push_str(&format!("&outcome_id={}", oid));
        }
        self.get_auth(&path)
    }

    /// List closed (filled/cancelled) orders for the authenticated user.
    pub fn get_closed_orders(
        &self,
        outcome_id: Option<&str>,
    ) -> Result<Vec<Order>, HexSdkError> {
        let pubkey = self.require_pubkey()?;
        let mut path = format!("/api/v1/orders?user={}&status=closed", pubkey);
        if let Some(oid) = outcome_id {
            path.push_str(&format!("&outcome_id={}", oid));
        }
        self.get_auth(&path)
    }
}
