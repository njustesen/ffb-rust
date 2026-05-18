#[tokio::main]
async fn main() {
    let app = ffb_server::create_app();
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    println!("FFB server listening on http://localhost:8080");
    axum::serve(listener, app).await.unwrap();
}
