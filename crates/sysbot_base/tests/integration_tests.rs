//! Integration tests for real Nintendo Switch connection
//!
//! These tests require a Nintendo Switch with sys-botbase installed
//! and connected to the same network.
//!
//! To run these tests:
//! 1. Set the SWITCH_IP environment variable to your Switch's IP address
//! 2. Run: cargo test --test integration_tests -- --ignored
//!
//! Example:
//!   SWITCH_IP=192.168.1.100 cargo test --test integration_tests -- --ignored

use std::env;
use std::time::Duration;
use sysbot_base::{ConnectionConfig, OffsetType, SwitchButton, SwitchConnection, WifiConnection};

#[allow(unused_imports)]
use std::fs;

/// Get the Switch IP from environment variable
fn get_switch_ip() -> String {
    env::var("SWITCH_IP").unwrap_or_else(|_| {
        // Try to read from .env file
        if let Ok(contents) = std::fs::read_to_string(".env") {
            for line in contents.lines() {
                if let Some(ip) = line.strip_prefix("SWITCH_IP=") {
                    return ip.trim().to_string();
                }
            }
        }
        panic!(
            "SWITCH_IP environment variable not set!\n\
            Please set it to your Nintendo Switch IP address:\n\
            \n\
            Option 1: Environment variable\n\
            SWITCH_IP=192.168.1.100 cargo test --test integration_tests -- --ignored\n\
            \n\
            Option 2: Create a .env file\n\
            echo \"SWITCH_IP=192.168.1.100\" > .env\n\
            cargo test --test integration_tests -- --ignored"
        )
    })
}

/// Get the Switch port (default: 6000)
fn get_switch_port() -> u16 {
    env::var("SWITCH_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(6000)
}

/// Create a connection to the Switch
fn create_connection() -> WifiConnection {
    let ip = get_switch_ip();
    let port = get_switch_port();

    let config = ConnectionConfig::wifi(&ip, port)
        .expect("Invalid IP address")
        .with_timeout(10000) // 10 seconds
        .with_name("IntegrationTest");

    WifiConnection::new(config)
}

// ============================================================================
// Connection Tests
// ============================================================================

#[tokio::test]
#[ignore = "Requires real Switch connection - run with --ignored flag"]
async fn test_connect_to_switch() {
    let mut conn = create_connection();

    let result = conn.connect().await;
    assert!(result.is_ok(), "Failed to connect: {:?}", result.err());
    assert!(conn.is_connected(), "Connection should be marked as connected");

    // Clean up
    let _ = conn.disconnect().await;
}

#[tokio::test]
#[ignore = "Requires real Switch connection - run with --ignored flag"]
async fn test_connect_and_disconnect() {
    let mut conn = create_connection();

    // Connect
    conn.connect().await.expect("Failed to connect");
    assert!(conn.is_connected());

    // Disconnect
    conn.disconnect().await.expect("Failed to disconnect");
    assert!(!conn.is_connected());
}

#[tokio::test]
#[ignore = "Requires real Switch connection - run with --ignored flag"]
async fn test_reconnect() {
    let mut conn = create_connection();

    // First connection
    conn.connect().await.expect("Failed to connect");
    conn.disconnect().await.expect("Failed to disconnect");

    // Reconnect
    conn.connect().await.expect("Failed to reconnect");
    assert!(conn.is_connected());

    // Clean up
    let _ = conn.disconnect().await;
}

// ============================================================================
// System Info Tests
// ============================================================================

#[tokio::test]
#[ignore = "Requires real Switch connection - run with --ignored flag"]
async fn test_get_sys_botbase_version() {
    let mut conn = create_connection();
    conn.connect().await.expect("Failed to connect");

    let result = conn.get_version().await;
    assert!(
        result.is_ok(),
        "Failed to get version: {:?}",
        result.err()
    );

    let version = result.unwrap();
    println!("sys-botbase version: {}", version);
    assert!(!version.is_empty(), "Version should not be empty");

    let _ = conn.disconnect().await;
}

#[tokio::test]
#[ignore = "Requires real Switch connection - run with --ignored flag"]
async fn test_get_title_id() {
    let mut conn = create_connection();
    conn.connect().await.expect("Failed to connect");

    let result = conn.get_title_id().await;
    assert!(
        result.is_ok(),
        "Failed to get title ID: {:?}",
        result.err()
    );

    let title_id = result.unwrap();
    println!("Current title ID: 0x{:016X}", title_id);

    // Title ID should be non-zero if a game is running
    // It may be 0 if on home screen
    println!(
        "Note: Title ID is {} - {}",
        title_id,
        if title_id == 0 {
            "No game running (home screen)"
        } else {
            "Game is running"
        }
    );

    let _ = conn.disconnect().await;
}

#[tokio::test]
#[ignore = "Requires real Switch connection - run with --ignored flag"]
async fn test_get_heap_base() {
    let mut conn = create_connection();
    conn.connect().await.expect("Failed to connect");

    let result = conn.get_heap_base().await;
    assert!(
        result.is_ok(),
        "Failed to get heap base: {:?}",
        result.err()
    );

    let heap_base = result.unwrap();
    println!("Heap base address: 0x{:016X}", heap_base);
    assert!(heap_base > 0, "Heap base should be non-zero");

    let _ = conn.disconnect().await;
}

#[tokio::test]
#[ignore = "Requires real Switch connection - run with --ignored flag"]
async fn test_get_main_nso_base() {
    let mut conn = create_connection();
    conn.connect().await.expect("Failed to connect");

    let result = conn.get_main_nso_base().await;
    assert!(
        result.is_ok(),
        "Failed to get main NSO base: {:?}",
        result.err()
    );

    let main_base = result.unwrap();
    println!("Main NSO base address: 0x{:016X}", main_base);
    assert!(main_base > 0, "Main NSO base should be non-zero");

    let _ = conn.disconnect().await;
}

// ============================================================================
// Button Tests
// ============================================================================

#[tokio::test]
#[ignore = "Requires real Switch connection - run with --ignored flag"]
async fn test_click_a_button() {
    let mut conn = create_connection();
    conn.connect().await.expect("Failed to connect");

    let result = conn.click(SwitchButton::A).await;
    assert!(result.is_ok(), "Failed to click A: {:?}", result.err());
    println!("Successfully clicked A button");

    let _ = conn.disconnect().await;
}

#[tokio::test]
#[ignore = "Requires real Switch connection - run with --ignored flag"]
async fn test_click_b_button() {
    let mut conn = create_connection();
    conn.connect().await.expect("Failed to connect");

    let result = conn.click(SwitchButton::B).await;
    assert!(result.is_ok(), "Failed to click B: {:?}", result.err());
    println!("Successfully clicked B button");

    let _ = conn.disconnect().await;
}

#[tokio::test]
#[ignore = "Requires real Switch connection - run with --ignored flag"]
async fn test_click_multiple_buttons() {
    let mut conn = create_connection();
    conn.connect().await.expect("Failed to connect");

    let buttons = [
        SwitchButton::A,
        SwitchButton::B,
        SwitchButton::X,
        SwitchButton::Y,
    ];

    for button in buttons {
        let result = conn.click(button).await;
        assert!(
            result.is_ok(),
            "Failed to click {:?}: {:?}",
            button,
            result.err()
        );
        println!("Successfully clicked {:?}", button);
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    let _ = conn.disconnect().await;
}

// ============================================================================
// Memory Tests
// ============================================================================

#[tokio::test]
#[ignore = "Requires real Switch connection - run with --ignored flag"]
async fn test_read_heap_memory() {
    let mut conn = create_connection();
    conn.connect().await.expect("Failed to connect");

    // Read 16 bytes from heap offset 0
    let result = conn.read_bytes(0, 16, OffsetType::Heap).await;
    assert!(
        result.is_ok(),
        "Failed to read heap memory: {:?}",
        result.err()
    );

    let bytes = result.unwrap();
    assert_eq!(bytes.len(), 16, "Should read exactly 16 bytes");
    println!("Read {} bytes from heap: {:02X?}", bytes.len(), bytes);

    let _ = conn.disconnect().await;
}

#[tokio::test]
#[ignore = "Requires real Switch connection - run with --ignored flag"]
async fn test_read_main_memory() {
    let mut conn = create_connection();
    conn.connect().await.expect("Failed to connect");

    // Read 16 bytes from main offset 0
    let result = conn.read_bytes(0, 16, OffsetType::Main).await;
    assert!(
        result.is_ok(),
        "Failed to read main memory: {:?}",
        result.err()
    );

    let bytes = result.unwrap();
    assert_eq!(bytes.len(), 16, "Should read exactly 16 bytes");
    println!("Read {} bytes from main: {:02X?}", bytes.len(), bytes);

    let _ = conn.disconnect().await;
}

// ============================================================================
// Full Connection Flow Test
// ============================================================================

#[tokio::test]
#[ignore = "Requires real Switch connection - run with --ignored flag"]
async fn test_full_connection_flow() {
    println!("\n=== Full Connection Flow Test ===\n");

    let mut conn = create_connection();

    // Step 1: Connect
    println!("Step 1: Connecting to Switch...");
    conn.connect().await.expect("Failed to connect");
    assert!(conn.is_connected());
    println!("  Connected successfully!");

    // Step 2: Get sys-botbase version
    println!("\nStep 2: Getting sys-botbase version...");
    let version = conn.get_version().await.expect("Failed to get version");
    println!("  sys-botbase version: {}", version);

    // Step 3: Get title ID
    println!("\nStep 3: Getting title ID...");
    let title_id = conn.get_title_id().await.expect("Failed to get title ID");
    println!("  Title ID: 0x{:016X}", title_id);

    // Step 4: Get heap base
    println!("\nStep 4: Getting heap base...");
    let heap_base = conn.get_heap_base().await.expect("Failed to get heap base");
    println!("  Heap base: 0x{:016X}", heap_base);

    // Step 5: Read some memory
    println!("\nStep 5: Reading memory...");
    let bytes = conn
        .read_bytes(0, 8, OffsetType::Heap)
        .await
        .expect("Failed to read memory");
    println!("  First 8 bytes of heap: {:02X?}", bytes);

    // Step 6: Click A button
    println!("\nStep 6: Clicking A button...");
    conn.click(SwitchButton::A)
        .await
        .expect("Failed to click A");
    println!("  A button clicked!");

    // Step 7: Disconnect
    println!("\nStep 7: Disconnecting...");
    conn.disconnect().await.expect("Failed to disconnect");
    assert!(!conn.is_connected());
    println!("  Disconnected successfully!");

    println!("\n=== All steps completed successfully! ===\n");
}

// ============================================================================
// Connection Error Tests
// ============================================================================

#[tokio::test]
async fn test_connection_to_invalid_ip_fails() {
    // This test doesn't require --ignored because it tests failure case
    let config = ConnectionConfig::wifi("192.0.2.1", 6000) // TEST-NET-1, should fail
        .expect("Valid IP format")
        .with_timeout(2000); // 2 seconds

    let mut conn = WifiConnection::new(config);
    let result = conn.connect().await;

    assert!(result.is_err(), "Connection to invalid IP should fail");
    println!("Connection correctly failed: {:?}", result.err());
}

#[tokio::test]
async fn test_connection_to_wrong_port_fails() {
    // This test doesn't require --ignored because it tests failure case
    let config = ConnectionConfig::wifi("127.0.0.1", 65534) // Local, wrong port
        .expect("Valid IP format")
        .with_timeout(2000); // 2 seconds

    let mut conn = WifiConnection::new(config);
    let result = conn.connect().await;

    assert!(result.is_err(), "Connection to wrong port should fail");
    println!("Connection correctly failed: {:?}", result.err());
}
