//! Integration test for full client-server command execution

use reticulum_core::MockInterface;
use shell_client::{client::Client, config::ClientConfig};
use shell_server::{config::ServerConfig, server::Server};
use std::sync::Arc;
use tokio::time::{sleep, Duration};

#[tokio::test]
async fn test_full_command_execution_flow() {
    // Create mock interfaces
    let (client_interface, server_interface) = MockInterface::create_pair();

    // Create server config
    let server_config = ServerConfig::default();
    let server_dest_hex = server_config.identity.destination_hex();

    // Create server with interface
    let server = Server::with_interface(server_config, Arc::new(server_interface))
        .await
        .unwrap();

    // Start server in background
    tokio::spawn(async move {
        if let Err(e) = server.run().await {
            eprintln!("Server error: {}", e);
        }
    });

    // Give server time to start
    sleep(Duration::from_millis(100)).await;

    // Create client config
    let mut client_config = ClientConfig::default();
    client_config.server_destination = server_dest_hex.clone();

    // Parse server destination to bytes
    let server_dest_bytes = hex::decode(&server_dest_hex).unwrap();
    let mut server_dest = [0u8; 32];
    server_dest.copy_from_slice(&server_dest_bytes);

    // Create client with interface
    let client = Client::with_interface(client_config, Arc::new(client_interface), server_dest)
        .await
        .unwrap();

    // Connect to server
    client.connect().await.unwrap();

    // Execute whoami command
    let response = client.execute_command("whoami".to_string(), vec![]).await.unwrap();

    println!("Command: whoami");
    println!("Exit code: {}", response.exit_code);
    println!("Stdout: {}", String::from_utf8_lossy(&response.stdout));
    println!("Stderr: {}", String::from_utf8_lossy(&response.stderr));

    assert_eq!(response.exit_code, 0);
    assert!(!response.stdout.is_empty());
}

#[tokio::test]
async fn test_ps_command() {
    // Create mock interfaces
    let (client_interface, server_interface) = MockInterface::create_pair();

    // Create server config
    let server_config = ServerConfig::default();
    let server_dest_hex = server_config.identity.destination_hex();

    // Create server with interface
    let server = Server::with_interface(server_config, Arc::new(server_interface))
        .await
        .unwrap();

    // Start server in background
    tokio::spawn(async move {
        if let Err(e) = server.run().await {
            eprintln!("Server error: {}", e);
        }
    });

    // Give server time to start
    sleep(Duration::from_millis(100)).await;

    // Create client config
    let mut client_config = ClientConfig::default();
    client_config.server_destination = server_dest_hex.clone();

    // Parse server destination to bytes
    let server_dest_bytes = hex::decode(&server_dest_hex).unwrap();
    let mut server_dest = [0u8; 32];
    server_dest.copy_from_slice(&server_dest_bytes);

    // Create client with interface
    let client = Client::with_interface(client_config, Arc::new(client_interface), server_dest)
        .await
        .unwrap();

    // Connect to server
    client.connect().await.unwrap();

    // Execute ps -ef command
    let response = client
        .execute_command("ps".to_string(), vec!["-ef".to_string()])
        .await
        .unwrap();

    println!("Command: ps -ef");
    println!("Exit code: {}", response.exit_code);
    println!("Stdout length: {} bytes", response.stdout.len());
    println!("First 200 chars: {}", String::from_utf8_lossy(&response.stdout[..200.min(response.stdout.len())]));

    assert_eq!(response.exit_code, 0);
    assert!(!response.stdout.is_empty());
    // ps output should contain process listings
    let output = String::from_utf8_lossy(&response.stdout);
    assert!(output.contains("PID") || output.contains("UID"));
}

#[tokio::test]
async fn test_ss_command() {
    // Create mock interfaces
    let (client_interface, server_interface) = MockInterface::create_pair();

    // Create server config
    let server_config = ServerConfig::default();
    let server_dest_hex = server_config.identity.destination_hex();

    // Create server with interface
    let server = Server::with_interface(server_config, Arc::new(server_interface))
        .await
        .unwrap();

    // Start server in background
    tokio::spawn(async move {
        if let Err(e) = server.run().await {
            eprintln!("Server error: {}", e);
        }
    });

    // Give server time to start
    sleep(Duration::from_millis(100)).await;

    // Create client config
    let mut client_config = ClientConfig::default();
    client_config.server_destination = server_dest_hex.clone();

    // Parse server destination to bytes
    let server_dest_bytes = hex::decode(&server_dest_hex).unwrap();
    let mut server_dest = [0u8; 32];
    server_dest.copy_from_slice(&server_dest_bytes);

    // Create client with interface
    let client = Client::with_interface(client_config, Arc::new(client_interface), server_dest)
        .await
        .unwrap();

    // Connect to server
    client.connect().await.unwrap();

    // Execute ss -antp command
    let response = client
        .execute_command("ss".to_string(), vec!["-antp".to_string()])
        .await
        .unwrap();

    println!("Command: ss -antp");
    println!("Exit code: {}", response.exit_code);
    println!("Stdout length: {} bytes", response.stdout.len());
    println!("First 200 chars: {}", String::from_utf8_lossy(&response.stdout[..200.min(response.stdout.len())]));

    assert_eq!(response.exit_code, 0);
    // ss output should show socket information
    let output = String::from_utf8_lossy(&response.stdout);
    // Output should contain typical ss headers or socket states
    assert!(output.contains("State") || output.contains("LISTEN") || output.contains("ESTAB"));
}
