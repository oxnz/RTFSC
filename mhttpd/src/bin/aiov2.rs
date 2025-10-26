use mhttpd::server;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::ERROR)
        .init();
    let mut server = server::v2::asyncio::Server::default();
    let addr = "127.0.0.1:8000";
    server.serve(addr).await.unwrap();
}
