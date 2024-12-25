use embedded_recruitment_task::{
    message::{client_message, server_message, AddRequest, EchoMessage},
    server::Server,
};
use std::{
    sync::Arc,
};
use rand::Rng;

mod client;

fn setup_server_task(server: Arc<Server>) {
    async_std::task::spawn(async move {
        server.run().await.expect("Server encountered an error");
    });
}

fn get_random_port() -> u16 {
    let mut rng = rand::thread_rng();
    rng.gen_range(10000..65535)
}

async fn create_server() -> Arc<Server> {
    let port = get_random_port();
    let addr = format!("localhost:{}",port);
    Arc::new(Server::new(&addr).await.expect("Failed to start server"))
}

#[async_std::test] // Make this test async
async fn test_client_connection() {
    // Set up the server in a separate task
    let server = create_server().await;
    setup_server_task(server.clone());

    // Create and connect the client
    let port = server.get_port();
    let mut client = client::Client::new("localhost", port.into(), 1000);
    assert!(client.connect().is_ok(), "Failed to connect to the server");

    // Disconnect the client
    assert!(
        client.disconnect().is_ok(),
        "Failed to disconnect from the server"
    );

    // Stop the server
    server.stop();
}

#[async_std::test] // Make this test async
async fn test_client_echo_message() {
    // Set up the server in a separate task
    let server = create_server().await;
    setup_server_task(server.clone());

    // Create and connect the client
    let port = server.get_port();
    let mut client = client::Client::new("localhost", port.into(), 1000);
    assert!(client.connect().is_ok(), "Failed to connect to the server");

    // Prepare the message
    let mut echo_message = EchoMessage::default();
    echo_message.content = "Hello, World!".to_string();
    let message = client_message::Message::EchoMessage(echo_message.clone());

    // Send the message to the server
    assert!(client.send(message).is_ok(), "Failed to send message");

    // Receive the echoed message
    let response = client.receive();
    assert!(
        response.is_ok(),
        "Failed to receive response for EchoMessage"
    );

    match response.unwrap().message {
        Some(server_message::Message::EchoMessage(echo)) => {
            assert_eq!(
                echo.content, echo_message.content,
                "Echoed message content does not match"
            );
        }
        _ => panic!("Expected EchoMessage, but received a different message"),
    }

    // Disconnect the client
    assert!(
        client.disconnect().is_ok(),
        "Failed to disconnect from the server"
    );

    // Stop the server
    server.stop();
}

#[async_std::test] // Make this test async
async fn test_multiple_echo_messages() {
    // Set up the server in a separate task
    let server = create_server().await;
    setup_server_task(server.clone());

    // Create and connect the client
    let port = server.get_port();
    let mut client = client::Client::new("localhost", port.into(), 1000);
    assert!(client.connect().is_ok(), "Failed to connect to the server");

    // Prepare multiple messages
    let messages = vec![
        "Hello, World!".to_string(),
        "How are you?".to_string(),
        "Goodbye!".to_string(),
    ];

    // Send and receive multiple messages
    for message_content in messages {
        let mut echo_message = EchoMessage::default();
        echo_message.content = message_content.clone();
        let message = client_message::Message::EchoMessage(echo_message);

        // Send the message to the server
        assert!(client.send(message).is_ok(), "Failed to send message");

        // Receive the echoed message
        let response = client.receive();
        assert!(
            response.is_ok(),
            "Failed to receive response for EchoMessage"
        );

        match response.unwrap().message {
            Some(server_message::Message::EchoMessage(echo)) => {
                assert_eq!(
                    echo.content, message_content,
                    "Echoed message content does not match"
                );
            }
            _ => panic!("Expected EchoMessage, but received a different message"),
        }
    }

    // Disconnect the client
    assert!(
        client.disconnect().is_ok(),
        "Failed to disconnect from the server"
    );

    // Stop the server
    server.stop();
}

#[async_std::test] // Make this test async
async fn test_multiple_clients() {
    // Set up the server in a separate task
    let server = create_server().await;
    setup_server_task(server.clone());

    // Create and connect multiple clients
    let port = server.get_port();
    let mut clients = vec![
        client::Client::new("localhost", port.into(), 1000),
        client::Client::new("localhost", port.into(), 1000),
        client::Client::new("localhost", port.into(), 1000),
    ];

    for client in clients.iter_mut() {
        assert!(client.connect().is_ok(), "Failed to connect to the server");
    }

    // Prepare multiple messages
    let messages = vec![
        "Hello, World!".to_string(),
        "How are you?".to_string(),
        "Goodbye!".to_string(),
    ];

    // Send and receive multiple messages for each client
    for message_content in messages {
        let mut echo_message = EchoMessage::default();
        echo_message.content = message_content.clone();
        let message = client_message::Message::EchoMessage(echo_message.clone());

        for client in clients.iter_mut() {
            // Send the message to the server
            assert!(
                client.send(message.clone()).is_ok(),
                "Failed to send message"
            );

            // Receive the echoed message
            let response = client.receive();
            assert!(
                response.is_ok(),
                "Failed to receive response for EchoMessage"
            );

            match response.unwrap().message {
                Some(server_message::Message::EchoMessage(echo)) => {
                    assert_eq!(
                        echo.content, message_content,
                        "Echoed message content does not match"
                    );
                }
                _ => panic!("Expected EchoMessage, but received a different message"),
            }
        }
    }

    // Disconnect the clients
    for client in clients.iter_mut() {
        assert!(
            client.disconnect().is_ok(),
            "Failed to disconnect from the server"
        );
    }

    // Stop the server
    server.stop();
}

#[async_std::test] // Make this test async
async fn test_client_add_request() {
    // Set up the server in a separate task
    let server = create_server().await;
    setup_server_task(server.clone());

    // Create and connect the client
    let port = server.get_port();
    let mut client = client::Client::new("localhost", port.into(), 1000);
    assert!(client.connect().is_ok(), "Failed to connect to the server");

    // Prepare the message
    let mut add_request = AddRequest::default();
    add_request.a = 10;
    add_request.b = 20;
    let message = client_message::Message::AddRequest(add_request.clone());

    // Send the message to the server
    assert!(client.send(message).is_ok(), "Failed to send message");

    // Receive the response
    let response = client.receive();
    assert!(
        response.is_ok(),
        "Failed to receive response for AddRequest"
    );

    match response.unwrap().message {
        Some(server_message::Message::AddResponse(add_response)) => {
            assert_eq!(
                add_response.result,
                add_request.a + add_request.b,
                "AddResponse result does not match"
            );
        }
        _ => panic!("Expected AddResponse, but received a different message"),
    }

    // Disconnect the client
    assert!(
        client.disconnect().is_ok(),
        "Failed to disconnect from the server"
    );

    // Stop the server
    server.stop();
}
