mod app;
mod config;
mod db;
mod log;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    config::check();
    log::setup()?;
    let pool = db::setup().await?;
    app::run(app::setup(pool).await?).await
}
