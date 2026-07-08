use std::net::SocketAddr;
use ffb_server::fantasy_football_server::FantasyFootballServer;

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let addr: SocketAddr = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or_else(|| "0.0.0.0:22222".parse().unwrap());

    FantasyFootballServer::new().run(addr).await;
}
