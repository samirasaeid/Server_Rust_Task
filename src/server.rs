use crate::message::{AddRequest, AddResponse, EchoMessage, ClientMessage, ServerMessage};
use crate::message::client_message::Message as ClientMessageType;
use crate::message::server_message::Message as ServerMessageType;
use prost::Message as ProstMessage;
use log::{error, info, warn};


use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use async_std::{
    io::{self, ReadExt, WriteExt},
    net::{TcpListener, TcpStream},
    sync::Mutex as AsyncMutex,
    task,

};

use tokio:: {
    sync::Notify,
};


struct Client {
    stream: Arc<AsyncMutex<TcpStream>>,
    buffer: Vec<u8>,
}

impl Client {
    pub fn new(stream: Arc<AsyncMutex<TcpStream>>) -> Self {
        Client {
            stream,
            buffer: vec![0; 512],
        }
    }

    pub async fn handle(&mut self) -> io::Result<()> {
        let mut stream = self.stream.lock().await; // lock the stream
        loop {
            let bytes_read = match stream.read(&mut self.buffer).await {
                Ok(0) => {
                    info!("Client disconnected.");
                    return Ok(());
                }
                Ok(n) => n,
                Err(e) => {
                    if e.kind() == io::ErrorKind::WouldBlock {
                        return Ok(()); // No data to read yet
                    } else {
                        return Err(e); // Other errors
                    }
                }
            };

            if let Ok(client_message) = ClientMessage::decode(&self.buffer[..bytes_read]) {
                match client_message.message{
                    Some(ClientMessageType::AddRequest(add_request)) => {
                        println!("Recieved AddRequest: a = {}, b = {}", add_request.a, add_request.b);
                        let result = add_request.a + add_request.b;

                        let add_response = AddResponse{result};

                        let server_message = ServerMessage {
                            message: Some(ServerMessageType::AddResponse(add_response)),
                        };

                        let payload = server_message.encode_to_vec();
                        stream.write(&payload).await?;
                        stream.flush().await?;
                    }
                    Some(ClientMessageType::EchoMessage(echo_message)) => {

                        info!("Recieved EchoMessage: {:?}", echo_message);
                        println!("Recieved EchoMessage: {:?}", echo_message);

                        let server_message = ServerMessage { 
                            message: Some(ServerMessageType::EchoMessage(echo_message)),
                        };

                        let payload = server_message.encode_to_vec();
                        stream.write(&payload).await?;
                        stream.flush().await?;
                    }
                    None => {
                        error!("Unknown message type.");
                        return Err(io::Error::new(io::ErrorKind::InvalidData, "Unknown message type"));
                    }
                }
            
            } else {
                error!("Failed to decode message");
                let error_msg = EchoMessage {
                    content: "Error".to_string(),
                };
                let payload = error_msg.encode_to_vec();
                stream.write_all(&payload).await?;
                stream.flush().await?;
            }
        }
    }
}

pub struct Server {
    listener: TcpListener,
    is_running: Arc<AtomicBool>,
    port: u16,
    shutdown_notify: Arc<Notify>, 
}

impl Server {
    /// Creates a new server instance
    pub async fn new(addr: &str) -> io::Result<Self> {
        let listener = TcpListener::bind(addr).await?;
        let port = listener.local_addr()?.port();
        Ok(Server {
            listener,
            port,
            is_running : Arc::new(AtomicBool::new(false)),
            shutdown_notify : Arc::new(Notify::new()),
        })
    }

    /// Runs the server, listening for incoming connections and handling them
    pub async fn run(&self) -> io::Result<()> {
        self.is_running.store(true, Ordering::SeqCst); // Set the server as running
        info!("Server is running on {}", self.listener.local_addr()?);
        println!("Server is running on {}", self.listener.local_addr()?);

        while self.is_running.load(Ordering::SeqCst) {
            match self.listener.accept().await {
                Ok((tcp_stream, addr)) => {
                    // Successfully accepted a connection
                    println!("Accepted connection from: {:?}", addr);
                    let is_running = self.is_running.clone();
                    let tcp_stream = Arc::new(AsyncMutex::new(tcp_stream));
                    task::spawn(async move {
                        println!("New client connected: {}", addr);
                        if let Err(e) = Server::handle_client(tcp_stream, is_running).await {
                            error!("Error handling client: {}", e);
                        } else {
                            info!("Client disconnected");
                        }
                    });
                }
                Err(e) => {
                    // Handle the error
                    eprintln!("Failed to accept connection: {}", e);
                    return Err(e);
                }
            }
        }
        self.shutdown_notify.notified().await;
        info!("Server shutdown completely");
        Ok(())
    }

    async fn handle_client(
        stream: Arc<AsyncMutex<TcpStream>>,
        is_running: Arc<AtomicBool>,
    ) -> io::Result<()> {
        let mut client = Client::new(stream.clone());
        while is_running.load(Ordering::SeqCst) {
            if let Err(e) = client.handle().await {
                error!("Error handling client: {}", e);
                return Err(e);
            }
        }
        info!("Client disconnected.");
        Ok(())
    }

    /// Stops the server by setting the "is_running" to false
    pub fn stop(&self) {
        if self.is_running.load(Ordering::SeqCst) {
            self.is_running.store(false, Ordering::SeqCst);
            self.shutdown_notify.notify_waiters();
            info!("Shutdown signal sent.");
        } else {
            warn!("Server was already stopped or not running.");
        }
    }

    //used to get the server port
    pub fn get_port(&self) -> u16 {
        self.port
    }
}
