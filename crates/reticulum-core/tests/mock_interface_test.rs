//! Integration test for MockInterface

use reticulum_core::{Identity, MockInterface, NetworkInterface, Packet, PacketType};

#[tokio::test]
async fn test_mock_interface_bidirectional() {
    // Create a pair of interfaces
    let (client_interface, server_interface) = MockInterface::create_pair();

    // Both should be ready
    assert!(client_interface.is_ready().await);
    assert!(server_interface.is_ready().await);

    // Create a test packet
    let destination = [42u8; 32];
    let test_data = b"Hello from client!".to_vec();
    let packet = Packet::data(destination, test_data.clone());

    // Client sends to server
    client_interface.send(&packet).await.unwrap();

    // Server receives
    let received = server_interface.receive().await.unwrap();
    assert_eq!(received.destination, destination);
    assert_eq!(received.data.as_ref(), test_data.as_slice());

    // Server responds
    let response_data = b"Hello from server!".to_vec();
    let response_packet = Packet::data(destination, response_data.clone());
    server_interface.send(&response_packet).await.unwrap();

    // Client receives response
    let client_received = client_interface.receive().await.unwrap();
    assert_eq!(client_received.data.as_ref(), response_data.as_slice());
}

#[tokio::test]
async fn test_mock_interface_with_identity() {
    let (client_interface, server_interface) = MockInterface::create_pair();

    // Create identities
    let client_identity = Identity::generate();
    let server_identity = Identity::generate();

    // Client sends signed packet
    let destination = server_identity.destination_hash();
    let data = b"Authenticated message".to_vec();
    let mut packet = Packet::data(destination, data.clone());

    // Sign the packet
    let signable_data = packet.signable_data();
    let signature = client_identity.sign(&signable_data);
    packet = packet.with_signature(signature);

    // Send
    client_interface.send(&packet).await.unwrap();

    // Server receives and verifies
    let received = server_interface.receive().await.unwrap();
    assert_eq!(received.destination, destination);

    // Verify signature
    let received_signable = received.signable_data();
    let received_signature = received.signature.as_ref().unwrap();
    Identity::verify_external(&client_identity.public_key(), &received_signable, received_signature)
        .unwrap();
}
