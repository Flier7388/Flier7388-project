//! One-off smoke test: logs in, persists+restores the session, starts
//! sync, lists joined rooms, and prints the last few messages of the
//! first one.
//!
//! Deliberately not a `#[test]` — needs real network access and a real
//! account, must never run in CI. Credentials come from environment
//! variables only; never written to a file. Sending a message is
//! implemented in `matrix-client` and exercised here only when
//! `MATRIX_SEND_TEST_MESSAGE` is set to a room ID — this is a visible
//! action against a real account, opt-in on purpose, confirmed with the
//! account owner before first use (2026-11-06, room "Слуги Донбасса").
//!
//! Usage:
//! ```sh
//! MATRIX_HOMESERVER=https://your-homeserver.example \
//! MATRIX_TEST_USER=... \
//! MATRIX_TEST_PASSWORD=... \
//! cargo run --example sync_smoke_test -p matrix-client
//! ```

// Same reason as `matrix_client`'s own `#![recursion_limit]` — this
// example's `#[tokio::main]` body is its own crate root and needs the
// attribute independently.
#![recursion_limit = "256"]

use std::time::Duration;

use matrix_client::MatrixClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let homeserver = std::env::var("MATRIX_HOMESERVER")
        .expect("MATRIX_HOMESERVER must be set (not hardcoded, not written to a file)");
    let username = std::env::var("MATRIX_TEST_USER")
        .expect("MATRIX_TEST_USER must be set (not hardcoded, not written to a file)");
    let password = std::env::var("MATRIX_TEST_PASSWORD")
        .expect("MATRIX_TEST_PASSWORD must be set (not hardcoded, not written to a file)");

    let store_path = std::env::temp_dir().join("matrix-client-sync-smoke-test");
    std::fs::create_dir_all(&store_path)?;

    println!("Logging in as {username} against {homeserver}...");
    let client = MatrixClient::login(&homeserver, &username, &password, &store_path).await?;

    let session = client
        .session()
        .expect("just logged in, session must exist");
    let round_tripped: matrix_client::Session =
        serde_json::from_str(&serde_json::to_string(&session)?)?;
    println!(
        "Session round-tripped through serde_json for user {}.",
        round_tripped.user_id
    );

    println!("Starting sync...");
    client.start_sync().await;
    tokio::time::sleep(Duration::from_secs(5)).await;

    let rooms = client.rooms().await?;
    println!("Joined rooms ({}):", rooms.len());
    for room in &rooms {
        println!("  {} — {}", room.room_id, room.display_name);
    }

    if rooms.is_empty() {
        println!("No joined rooms to read a timeline from.");
    }

    let send_target = std::env::var("MATRIX_SEND_TEST_MESSAGE").ok();
    for room in &rooms {
        let timeline = client.timeline_for(&room.room_id).await?;

        if send_target.as_deref() == Some(room.room_id.as_str()) {
            println!("Sending a test message to {}...", room.display_name);
            timeline
                .send_text("matrix-client sync_smoke_test: send_text() + timeline read check")
                .await?;
            tokio::time::sleep(Duration::from_secs(3)).await;
        }

        let messages = timeline.messages().await;
        println!(
            "Last messages in {} ({} total):",
            room.display_name,
            messages.len()
        );
        for message in messages.iter().rev().take(5).rev() {
            println!("  {}: {}", message.sender, message.body);
        }
    }

    client.stop_sync().await;
    Ok(())
}
