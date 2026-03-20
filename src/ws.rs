//! WebSocket clients for real-time market data and user events.
//!
//! Two WebSocket endpoints are available:
//!
//! - **`/ws/market`** — public market data (order books, trades, prices).
//!   Subscribe by outcome (asset) IDs, no authentication required.
//! - **`/ws/user`** — private user events (order fills, cancellations).
//!   Requires L2 API key authentication.
//!
//! These clients use a background thread to manage the WebSocket connection
//! and provide a blocking `recv()` API via crossbeam channels.
//!
//! # Example — Market WebSocket
//!
//! ```no_run
//! use hexmarket_sdk_sync::ws::HexMarketWs;
//!
//! let (ws, rx) = HexMarketWs::connect("wss://api.hexmarket.xyz/ws/market")?;
//! ws.subscribe(vec!["outcome-id-1".into()])?;
//!
//! while let Ok(event) = rx.recv() {
//!     println!("event_type={}, asset_id={}", event.event_type, event.asset_id);
//! }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! # Example — User WebSocket
//!
//! ```no_run
//! use hexmarket_sdk_sync::ws::HexUserWs;
//! use hexmarket_sdk_sync::ApiCredentials;
//!
//! let creds = ApiCredentials {
//!     api_key: "your-api-key".into(),
//!     secret: "your-secret".into(),
//!     passphrase: "your-passphrase".into(),
//! };
//!
//! let (_ws, rx) = HexUserWs::connect("wss://api.hexmarket.xyz/ws/user", creds, vec![])?;
//!
//! while let Ok(event) = rx.recv() {
//!     println!("{:?}", event);
//! }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use serde::{Deserialize, Serialize};
use std::sync::mpsc;
use tungstenite::{connect, Message};

use crate::auth::ApiCredentials;

// ---------------------------------------------------------------------------
// Market WebSocket
// ---------------------------------------------------------------------------

/// A market data event received from `/ws/market`.
#[derive(Debug, Clone, Deserialize)]
pub struct MarketEvent {
    pub event_type: String,
    pub asset_id: String,
    #[serde(flatten)]
    pub data: serde_json::Value,
}

/// Handle to a connected market WebSocket.
pub struct HexMarketWs {
    cmd_tx: mpsc::Sender<String>,
}

impl HexMarketWs {
    /// Connect to the market WebSocket endpoint.
    /// Returns a handle and a receiver for incoming [`MarketEvent`]s.
    ///
    /// Spawns a background thread for the WebSocket I/O loop.
    pub fn connect(
        url: &str,
    ) -> Result<(Self, mpsc::Receiver<MarketEvent>), Box<dyn std::error::Error>> {
        let (mut socket, _) = connect(url)?;

        let (event_tx, event_rx) = mpsc::channel::<MarketEvent>();
        let (cmd_tx, cmd_rx) = mpsc::channel::<String>();

        std::thread::spawn(move || {
            // Process outgoing commands (non-blocking check)
            loop {
                // Send any pending commands
                while let Ok(cmd) = cmd_rx.try_recv() {
                    if socket.send(Message::Text(cmd.into())).is_err() {
                        return;
                    }
                }

                // Try to read a message with a short timeout
                match socket.read() {
                    Ok(Message::Text(text)) => {
                        let text = text.to_string();
                        if text == "PONG" {
                            continue;
                        }
                        if let Ok(event) = serde_json::from_str::<MarketEvent>(&text) {
                            if event_tx.send(event).is_err() {
                                return;
                            }
                        }
                    }
                    Ok(Message::Close(_)) | Err(_) => return,
                    _ => continue,
                }
            }
        });

        Ok((Self { cmd_tx }, event_rx))
    }

    /// Subscribe to market events for the given outcome (asset) IDs.
    pub fn subscribe(&self, asset_ids: Vec<String>) -> Result<(), mpsc::SendError<String>> {
        let msg = serde_json::json!({
            "assets_ids": asset_ids,
            "type": "market"
        });
        self.cmd_tx.send(msg.to_string())
    }

    /// Dynamically subscribe to additional asset IDs.
    pub fn subscribe_more(&self, asset_ids: Vec<String>) -> Result<(), mpsc::SendError<String>> {
        let msg = serde_json::json!({
            "operation": "subscribe",
            "assets_ids": asset_ids
        });
        self.cmd_tx.send(msg.to_string())
    }

    /// Unsubscribe from asset IDs.
    pub fn unsubscribe(&self, asset_ids: Vec<String>) -> Result<(), mpsc::SendError<String>> {
        let msg = serde_json::json!({
            "operation": "unsubscribe",
            "assets_ids": asset_ids
        });
        self.cmd_tx.send(msg.to_string())
    }
}

// ---------------------------------------------------------------------------
// User WebSocket
// ---------------------------------------------------------------------------

/// An event received from `/ws/user`.
#[derive(Debug, Clone, Deserialize)]
pub struct UserEvent {
    #[serde(flatten)]
    pub data: serde_json::Value,
}

/// Handle to a connected user WebSocket.
pub struct HexUserWs {
    cmd_tx: mpsc::Sender<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct UserAuthMessage<'a> {
    auth: UserAuthPayload<'a>,
    #[serde(rename = "type")]
    msg_type: &'static str,
    markets: &'a [String],
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct UserAuthPayload<'a> {
    api_key: &'a str,
    secret: &'a str,
    passphrase: &'a str,
}

impl HexUserWs {
    /// Connect to the user WebSocket endpoint with L2 API key credentials.
    /// Returns a handle and a receiver for incoming [`UserEvent`]s.
    ///
    /// Spawns a background thread for the WebSocket I/O loop.
    pub fn connect(
        url: &str,
        credentials: ApiCredentials,
        markets: Vec<String>,
    ) -> Result<(Self, mpsc::Receiver<UserEvent>), Box<dyn std::error::Error>> {
        let (mut socket, _) = connect(url)?;

        // Send auth message immediately
        let auth_msg = serde_json::to_string(&UserAuthMessage {
            auth: UserAuthPayload {
                api_key: &credentials.api_key,
                secret: &credentials.secret,
                passphrase: &credentials.passphrase,
            },
            msg_type: "user",
            markets: &markets,
        })?;
        socket.send(Message::Text(auth_msg.into()))?;

        let (event_tx, event_rx) = mpsc::channel::<UserEvent>();
        let (cmd_tx, cmd_rx) = mpsc::channel::<String>();

        std::thread::spawn(move || {
            loop {
                while let Ok(cmd) = cmd_rx.try_recv() {
                    if socket.send(Message::Text(cmd.into())).is_err() {
                        return;
                    }
                }

                match socket.read() {
                    Ok(Message::Text(text)) => {
                        let text = text.to_string();
                        if text == "PONG" {
                            continue;
                        }
                        if let Ok(event) = serde_json::from_str::<UserEvent>(&text) {
                            if event_tx.send(event).is_err() {
                                return;
                            }
                        }
                    }
                    Ok(Message::Close(_)) | Err(_) => return,
                    _ => continue,
                }
            }
        });

        Ok((Self { cmd_tx }, event_rx))
    }

    /// Dynamically subscribe to additional markets.
    pub fn subscribe_markets(&self, markets: Vec<String>) -> Result<(), mpsc::SendError<String>> {
        let msg = serde_json::json!({
            "operation": "subscribe",
            "markets": markets
        });
        self.cmd_tx.send(msg.to_string())
    }

    /// Dynamically unsubscribe from markets.
    pub fn unsubscribe_markets(&self, markets: Vec<String>) -> Result<(), mpsc::SendError<String>> {
        let msg = serde_json::json!({
            "operation": "unsubscribe",
            "markets": markets
        });
        self.cmd_tx.send(msg.to_string())
    }
}
