use crate::client::HexClient;
use crate::error::HexSdkError;
use crate::types::VaultBalance;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct TransactionResponse {
    pub transaction: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct SubmitResponse {
    pub signature: String,
}

impl HexClient {
    /// Create a vault for the authenticated user (requires L2 auth).
    /// Returns a partially-signed transaction to co-sign and submit.
    pub fn create_vault(&self) -> Result<TransactionResponse, HexSdkError> {
        let path = "/api/v1/vault/create";
        self.post_auth(path, &serde_json::json!({}))
    }

    /// Build a deposit transaction (requires L2 auth).
    /// `amount` is in USDC base units (6 decimals, e.g. 10_000_000 = 10 USDC).
    pub fn deposit(&self, amount: u64) -> Result<TransactionResponse, HexSdkError> {
        let path = "/api/v1/vault/deposit";
        self.post_auth(path, &serde_json::json!({ "amount": amount }))
    }

    /// Build a withdrawal transaction (requires L2 auth).
    /// `amount` is in USDC base units (6 decimals).
    pub fn withdraw(&self, amount: u64) -> Result<TransactionResponse, HexSdkError> {
        let path = "/api/v1/vault/withdraw";
        self.post_auth(path, &serde_json::json!({ "amount": amount }))
    }

    /// Submit a fully-signed transaction to Solana (requires L2 auth).
    pub fn submit_transaction(
        &self,
        transaction_b64: &str,
    ) -> Result<SubmitResponse, HexSdkError> {
        let path = "/api/v1/vault/submit";
        self.post_auth(path, &serde_json::json!({ "transaction": transaction_b64 }))
    }

    /// Get on-chain vault USDC balance (requires L2 auth).
    pub fn get_vault_balance(&self) -> Result<VaultBalance, HexSdkError> {
        let pubkey = self.require_pubkey()?;
        let path = format!("/api/v1/vault/balance?user={}", pubkey);
        self.get_auth(&path)
    }
}
