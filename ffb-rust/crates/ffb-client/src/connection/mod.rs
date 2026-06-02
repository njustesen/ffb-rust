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

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::net::TcpListener;
    use tokio_tungstenite::accept_async;
    use tokio_tungstenite::tungstenite::Message;
    use futures_util::{SinkExt, StreamExt};
    use ffb_protocol::client_commands::{ClientCommand, ClientEndTurn};
    use ffb_protocol::server_commands::{ServerCommand, ServerPong};

    #[tokio::test]
    async fn connect_establishes_websocket() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let _ws = accept_async(stream).await.unwrap();
            // Server just accepts and drops the connection.
        });
        let url = format!("ws://{addr}");
        let result = ServerConnection::connect(&url).await;
        assert!(result.is_ok(), "connect() must succeed against a local server");
    }

    #[tokio::test]
    async fn send_transmits_client_command_as_json() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let (tx, rx) = tokio::sync::oneshot::channel::<String>();
        tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let ws = accept_async(stream).await.unwrap();
            let (_, mut source) = ws.split();
            if let Some(Ok(Message::Text(text))) = source.next().await {
                let _ = tx.send(text);
            }
        });
        let url = format!("ws://{addr}");
        let (mut conn, _rx) = ServerConnection::connect(&url).await.unwrap();
        conn.send(&ClientCommand::ClientEndTurn(ClientEndTurn)).await.unwrap();
        let received = tokio::time::timeout(
            std::time::Duration::from_secs(2),
            rx,
        ).await.expect("timed out waiting for message").unwrap();
        assert!(received.contains("clientEndTurn"),
            "send() must transmit command as JSON, got: {received}");
    }

    #[tokio::test]
    async fn incoming_server_command_dispatched_to_receiver() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let pong_json = serde_json::to_string(
            &ServerCommand::ServerPong(ServerPong { timestamp: 42 })
        ).unwrap();
        tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let ws = accept_async(stream).await.unwrap();
            let (mut sink, _) = ws.split();
            let _ = sink.send(Message::Text(pong_json)).await;
        });
        let url = format!("ws://{addr}");
        let (_conn, mut rx) = ServerConnection::connect(&url).await.unwrap();
        let cmd = tokio::time::timeout(
            std::time::Duration::from_secs(2),
            rx.recv(),
        ).await.expect("timed out waiting for command").expect("channel closed");
        assert!(matches!(cmd, ServerCommand::ServerPong(ServerPong { timestamp: 42 })),
            "incoming server message must be dispatched to the receiver channel");
    }

    #[tokio::test]
    async fn close_sends_close_frame() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let (tx, rx) = tokio::sync::oneshot::channel::<bool>();
        tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let ws = accept_async(stream).await.unwrap();
            let (_, mut source) = ws.split();
            while let Some(msg) = source.next().await {
                if matches!(msg, Ok(Message::Close(_))) {
                    let _ = tx.send(true);
                    break;
                }
            }
        });
        let url = format!("ws://{addr}");
        let (mut conn, _rx) = ServerConnection::connect(&url).await.unwrap();
        conn.close().await.unwrap();
        let got_close = tokio::time::timeout(
            std::time::Duration::from_secs(2),
            rx,
        ).await.expect("timed out waiting for close").unwrap();
        assert!(got_close, "close() must send a Close frame to the server");
    }
}
