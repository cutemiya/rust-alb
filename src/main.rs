mod app;
mod api;
mod config;

use app::App;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    App::new()?.run().await?;

    Ok(())
}