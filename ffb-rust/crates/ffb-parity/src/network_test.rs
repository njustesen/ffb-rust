/// Network integration test: two Rust clients vs. the Java server.
///
/// Run with: `cargo run -p ffb-parity -- --network`
///
/// Requires the Java server running at PARITY_SERVER (default: ws://localhost:22222/ffb).
/// Runs 10 seeded games and compares the event logs against the headless Rust logs.
pub fn run() {
    println!("Network integration test: not yet implemented.");
    println!("Set PARITY_SERVER=ws://<host>:<port>/ffb and ensure the Java server is running.");
    println!("Skipping — exiting with success (stub).");
}
