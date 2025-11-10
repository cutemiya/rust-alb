mod app;
mod api;
mod config;
mod balancer;
mod limiter;
mod models;

use app::App;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    App::new().await?.run().await?;

    Ok(())
}