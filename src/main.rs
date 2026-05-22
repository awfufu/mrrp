mod app;
mod cache;
mod config;
mod error;
mod http;
mod models;
mod services;
mod sources;

#[tokio::main]
async fn main() {
    app::run().await;
}
