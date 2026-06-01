use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures_util::{SinkExt, StreamExt};
use serde_json;

use ffb_protocol::commands::{parse_server_command, serialize_client_command, ProtocolError};
use ffb_protocol::client_commands::ClientCommand;
use ffb_protocol::server_commands::ServerCommand;

#[derive(Debug, thiserror::Error)]
pub enum ConnectionError {
    #[error("WebSocket error: {0}")]
    Ws(#[from] tokio_tungstenite::tungstenite::Error),
    #[error("protocol error: {0}")]
    Protocol(#[from] ProtocolError),
    #[error("connection closed")]
    Closed,
}

/// Async WebSocket connection to the FFB Java server.
pub struct ServerConnection {
    sender: futures_util::stream::SplitSink<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>
        >,
        Message,
    >,
}

impl ServerConnection {
    /// Connect to `url` (e.g. `ws://localhost:22222/ffb`).
    ///
    /// Returns the connection and a receiver channel for incoming server commands.
    pub async fn connect(
        url: &str,
    ) -> Result<(Self, tokio::sync::mpsc::Receiver<ServerCommand>), ConnectionError> {
        let (ws, _) = connect_async(url).await?;
        let (sender, mut receiver) = ws.split();

        let (tx, rx) = tokio::sync::mpsc::channel::<ServerCommand>(256);

        tokio::spawn(async move {
            while let Some(msg) = receiver.next().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        match parse_server_command(&text) {
                            Ok(cmd) => {
                                if tx.send(cmd).await.is_err() {
                                    break;
                                }
                            }
                            Err(e) => {
                                log::warn!("Failed to parse server command: {e}");
                            }
                        }
                    }
                    Ok(Message::Binary(data)) => {
                        if let Ok(text) = std::str::from_utf8(&data) {
                            match parse_server_command(text) {
                                Ok(cmd) => {
                                    if tx.send(cmd).await.is_err() {
                                        break;
                                    }
                                }
                                Err(e) => {
                                    log::warn!("Failed to parse binary server command: {e}");
                                }
                            }
                        }
                    }
                    Ok(Message::Close(_)) => break,
                    Ok(Message::Ping(data)) => {
                        log::trace!("Ping received ({} bytes)", data.len());
                    }
                    Ok(_) => {}
                    Err(e) => {
                        log::error!("WebSocket receive error: {e}");
                        break;
                    }
                }
            }
        });

        Ok((ServerConnection { sender }, rx))
    }

    /// Send a client command to the server.
    pub async fn send(&mut self, cmd: &ClientCommand) -> Result<(), ConnectionError> {
        let json = serialize_client_command(cmd)?;
        self.sender.send(Message::Text(json)).await?;
        Ok(())
    }

    /// Send a keep-alive ping.
    pub async fn ping(&mut self, timestamp: i64) -> Result<(), ConnectionError> {
        use ffb_protocol::client_commands::{ClientCommand, ClientPing};
        self.send(&ClientCommand::ClientPing(ClientPing { timestamp })).await
    }

    /// Close the connection.
    pub async fn close(&mut self) -> Result<(), ConnectionError> {
        self.sender.send(Message::Close(None)).await?;
        Ok(())
    }
}
