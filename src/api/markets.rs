use crate::client::HexClient;
use crate::error::HexSdkError;
use crate::types::Outcome;

/// Query parameters for listing markets/outcomes.
#[derive(Debug, Default)]
pub struct ListMarketsParams {
    pub status: Option<String>,
    pub category: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

impl HexClient {
    /// List outcomes (paginated).
    pub fn list_markets(&self, params: &ListMarketsParams) -> Result<Vec<Outcome>, HexSdkError> {
        let mut url = self.url("/api/v1/markets");
        let mut sep = '?';
        if let Some(ref s) = params.status {
            url.push_str(&format!("{}status={}", sep, s));
            sep = '&';
        }
        if let Some(ref c) = params.category {
            url.push_str(&format!("{}category={}", sep, c));
            sep = '&';
        }
        if let Some(l) = params.limit {
            url.push_str(&format!("{}limit={}", sep, l));
            sep = '&';
        }
        if let Some(o) = params.offset {
            url.push_str(&format!("{}offset={}", sep, o));
            let _ = sep;
        }

        self.get(&url)
    }

    /// Get a single outcome by ID.
    pub fn get_market(&self, outcome_id: &str) -> Result<Outcome, HexSdkError> {
        self.get(&self.url(&format!("/api/v1/markets/{}", outcome_id)))
    }
}
