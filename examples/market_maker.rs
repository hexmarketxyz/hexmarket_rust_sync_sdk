//! Simple symmetric market maker — places bid/ask quotes around the mid price.
//!
//! Usage:
//!   export HEX_API_URL=https://api.hexmarket.xyz
//!   export HEX_API_KEY=...  HEX_SECRET=...  HEX_PASSPHRASE=...  HEX_PUBKEY=...
//!   export HEX_SECRET_KEY=<base58 ed25519 secret key>
//!   cargo run --example market_maker -- <outcome_id> [spread] [size]

use hexmarket_sdk_sync::*;
use rust_decimal::prelude::*;
use rust_decimal_macros::dec;
use std::{thread, time::Duration};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let outcome_id = args.get(1).expect("Usage: market_maker <outcome_id> [spread] [size]");
    let half_spread: Decimal = args
        .get(2)
        .and_then(|s| Decimal::from_str(s).ok())
        .unwrap_or(dec!(0.02));
    let size: u64 = args
        .get(3)
        .and_then(|s| s.parse().ok())
        .unwrap_or(10);

    let api_url =
        std::env::var("HEX_API_URL").unwrap_or_else(|_| "https://api.hexmarket.xyz".into());
    let client = HexClient::new(HexClientConfig { api_url });

    client.set_credentials(
        &std::env::var("HEX_PUBKEY").expect("HEX_PUBKEY required"),
        ApiCredentials {
            api_key: std::env::var("HEX_API_KEY").expect("HEX_API_KEY required"),
            secret: std::env::var("HEX_SECRET").expect("HEX_SECRET required"),
            passphrase: std::env::var("HEX_PASSPHRASE").expect("HEX_PASSPHRASE required"),
        },
    );

    let secret_key_b58 = std::env::var("HEX_SECRET_KEY").expect("HEX_SECRET_KEY required");
    let key_bytes = bs58::decode(&secret_key_b58).into_vec()?;
    let signing_key = ed25519_dalek::SigningKey::from_bytes(key_bytes[..32].try_into()?);

    println!("Market maker started — outcome={outcome_id}, spread={half_spread}, size={size}");

    loop {
        // Cancel existing orders
        let open = client.get_open_orders(Some(outcome_id))?;
        for order in &open {
            let _ = client.cancel_order(&order.id.to_string());
        }

        // Read orderbook for mid price
        let book = client.get_orderbook(outcome_id)?;
        let best_bid = book.bids.first().map(|l| l.price);
        let best_ask = book.asks.first().map(|l| l.price);

        let mid = match (best_bid, best_ask) {
            (Some(b), Some(a)) => (b + a) / dec!(2),
            (Some(b), None) => b + half_spread,
            (None, Some(a)) => a - half_spread,
            _ => dec!(0.50),
        };

        let bid_price = (mid - half_spread).max(dec!(0.01)).round_dp(2);
        let ask_price = (mid + half_spread).min(dec!(0.99)).round_dp(2);

        // Place bid
        let nonce = auth::generate_nonce();
        let msg = auth::build_order_message(outcome_id, "buy", &bid_price.to_string(), size, nonce);
        let sig = auth::ed25519_sign(&signing_key, &msg);
        match client.place_order(&PlaceOrderParams {
            outcome_id: outcome_id.clone(),
            side: Side::Buy,
            order_type: OrderType::Limit,
            time_in_force: TimeInForce::Gtc,
            price: bid_price,
            quantity: size,
            nonce,
            signature: sig,
        }) {
            Ok(r) => println!("BID {bid_price} x{size} → {}", r.order_id),
            Err(e) => eprintln!("bid error: {e}"),
        }

        // Place ask
        let nonce = auth::generate_nonce();
        let msg = auth::build_order_message(outcome_id, "sell", &ask_price.to_string(), size, nonce);
        let sig = auth::ed25519_sign(&signing_key, &msg);
        match client.place_order(&PlaceOrderParams {
            outcome_id: outcome_id.clone(),
            side: Side::Sell,
            order_type: OrderType::Limit,
            time_in_force: TimeInForce::Gtc,
            price: ask_price,
            quantity: size,
            nonce,
            signature: sig,
        }) {
            Ok(r) => println!("ASK {ask_price} x{size} → {}", r.order_id),
            Err(e) => eprintln!("ask error: {e}"),
        }

        thread::sleep(Duration::from_secs(5));
    }
}
