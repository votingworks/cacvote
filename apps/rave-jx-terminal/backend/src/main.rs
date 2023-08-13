mod app;
mod cac;
mod config;
mod db;
mod log;
mod sync;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    config::check();
    log::setup()?;
    let pool = db::setup().await?;
    sync::sync_periodically(&pool).await;
    app::run(app::setup(pool).await?).await
}
