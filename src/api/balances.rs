use crate::client::HexClient;
use crate::error::HexSdkError;
use crate::types::UserBalance;

impl HexClient {
    /// Get USDC balance and locked amount for the authenticated user (requires L2 auth).
    pub fn get_balance(&self) -> Result<UserBalance, HexSdkError> {
        let pubkey = self.require_pubkey()?;
        let path = format!("/api/v1/balances?user={}", pubkey);
        self.get_auth(&path)
    }
}
