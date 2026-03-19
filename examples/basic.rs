//! Basic usage: browse events, read orderbook, check balance.

use hexmarket_sdk_sync::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_url =
        std::env::var("HEX_API_URL").unwrap_or_else(|_| "https://api.hexmarket.xyz".into());

    let client = HexClient::new(HexClientConfig { api_url });

    // --- Public endpoints (no auth) ---

    // List active events
    let events = client.list_events(&ListEventsParams {
        status: Some("active".into()),
        limit: Some(5),
        ..Default::default()
    })?;

    println!("=== Active Events ===");
    for item in &events {
        println!(
            "  {} — {} outcomes",
            item.event.title,
            item.outcomes.len()
        );
    }

    // Read the orderbook for the first outcome
    if let Some(first) = events.first().and_then(|e| e.outcomes.first()) {
        let book = client.get_orderbook(&first.id.to_string())?;
        println!("\n=== Orderbook for {} ===", first.label);
        println!("  Best bid: {:?}", book.bids.first());
        println!("  Best ask: {:?}", book.asks.first());
    }

    // --- Authenticated endpoints ---

    let api_key = std::env::var("HEX_API_KEY").ok();
    let secret = std::env::var("HEX_SECRET").ok();
    let passphrase = std::env::var("HEX_PASSPHRASE").ok();
    let pubkey = std::env::var("HEX_PUBKEY").ok();

    if let (Some(api_key), Some(secret), Some(passphrase), Some(pubkey)) =
        (api_key, secret, passphrase, pubkey)
    {
        client.set_credentials(
            &pubkey,
            ApiCredentials {
                api_key,
                secret,
                passphrase,
            },
        );

        let balance = client.get_balance()?;
        println!("\n=== Balance ===");
        println!(
            "  USDC: {} (locked: {})",
            balance.usdc_balance, balance.locked_usdc
        );

        let positions = client.get_positions()?;
        println!("\n=== Positions ({}) ===", positions.len());
        for pos in &positions {
            println!(
                "  outcome={} qty={} avg_price={:?}",
                pos.outcome_id, pos.quantity, pos.avg_price
            );
        }
    } else {
        println!("\nSkipping auth endpoints (set HEX_API_KEY, HEX_SECRET, HEX_PASSPHRASE, HEX_PUBKEY)");
    }

    Ok(())
}
