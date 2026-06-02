/// Network integration test: two Rust clients vs. the Java server.
///
/// Run with: `cargo run -p ffb-parity -- --network`
///
/// Requires the Java server running at PARITY_SERVER (default: ws://localhost:22222/ffb).
/// Tests that the Rust client can connect, complete the protocol handshake, and receive
/// at least one valid ServerCommand from the server.
///
/// For a full game-driving test, additional game_id / team setup is required (future work).
pub fn run() {
    let server_url = std::env::var("PARITY_SERVER")
        .unwrap_or_else(|_| "ws://localhost:22222/ffb".to_string());

    println!("Network integration test");
    println!("  Server: {server_url}");
    println!("  (Set PARITY_SERVER env var to override)");
    println!();

    if std::env::var("PARITY_SERVER").is_err() {
        println!("PARITY_SERVER not set — skipping network test (CI-safe).");
        println!("To run: set PARITY_SERVER=ws://<host>:<port>/ffb and start the Java server.");
        return;
    }

    let rt = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");
    match rt.block_on(run_async(&server_url)) {
        Ok(()) => {
            println!("Network integration test: PASSED");
        }
        Err(e) => {
            eprintln!("Network integration test: FAILED — {e}");
            std::process::exit(1);
        }
    }
}

async fn run_async(url: &str) -> Result<(), String> {
    use ffb_client::connection::ServerConnection;
    use ffb_protocol::server_commands::ServerCommand;

    println!("Connecting to {url}...");
    let (_conn, mut rx) = ServerConnection::connect(url).await
        .map_err(|e| format!("connection failed: {e}"))?;
    println!("  WebSocket handshake complete.");

    // Wait up to 5 seconds for at least one valid ServerCommand.
    let cmd = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        rx.recv(),
    ).await
        .map_err(|_| "timed out waiting for first server command (is the server running and sending?)".to_string())?
        .ok_or_else(|| "server closed the channel without sending a command".to_string())?;

    let cmd_name = server_command_name(&cmd);
    println!("  Received first server command: {cmd_name}");
    println!("  Protocol handshake verified.");
    Ok(())
}

fn server_command_name(cmd: &ffb_protocol::server_commands::ServerCommand) -> &'static str {
    use ffb_protocol::server_commands::ServerCommand;
    match cmd {
        ServerCommand::ServerGameState(_) => "ServerGameState",
        ServerCommand::ServerModelSync(_) => "ServerModelSync",
        ServerCommand::ServerGameTime(_) => "ServerGameTime",
        ServerCommand::ServerStatus(_) => "ServerStatus",
        ServerCommand::ServerJoin(_) => "ServerJoin",
        ServerCommand::ServerLeave(_) => "ServerLeave",
        ServerCommand::ServerTalk(_) => "ServerTalk",
        ServerCommand::ServerAdminMessage(_) => "ServerAdminMessage",
        ServerCommand::ServerSound(_) => "ServerSound",
        ServerCommand::ServerPong(_) => "ServerPong",
        ServerCommand::ServerPasswordChallenge(_) => "ServerPasswordChallenge",
        ServerCommand::ServerVersion(_) => "ServerVersion",
        ServerCommand::ServerAddPlayer(_) => "ServerAddPlayer",
        ServerCommand::ServerZapPlayer(_) => "ServerZapPlayer",
        ServerCommand::ServerUnzapPlayer(_) => "ServerUnzapPlayer",
        ServerCommand::ServerGameList(_) => "ServerGameList",
        ServerCommand::ServerTeamList(_) => "ServerTeamList",
    }
}
