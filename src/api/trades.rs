use crate::client::HexClient;
use crate::error::HexSdkError;
use crate::types::Trade;

/// Query parameters for listing trades.
#[derive(Debug, Default)]
pub struct ListTradesParams {
    pub outcome_id: Option<String>,
    pub user: Option<String>,
    pub limit: Option<i64>,
}

impl HexClient {
    /// List trades (no auth required).
    pub fn list_trades(&self, params: &ListTradesParams) -> Result<Vec<Trade>, HexSdkError> {
        let mut url = self.url("/api/v1/trades");
        let mut sep = '?';
        if let Some(ref oid) = params.outcome_id {
            url.push_str(&format!("{}outcome_id={}", sep, oid));
            sep = '&';
        }
        if let Some(ref u) = params.user {
            url.push_str(&format!("{}user={}", sep, u));
            sep = '&';
        }
        if let Some(l) = params.limit {
            url.push_str(&format!("{}limit={}", sep, l));
            let _ = sep;
        }

        self.get(&url)
    }
}
