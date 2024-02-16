use anyhow::{ Result};
use log::*;
use std::env;

mod bot;
mod db;
mod types;
mod auth;
mod http;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let token = &args[1];
    let token_clone = token.clone();
    let mut db = db::Database::open()?;
    db.migrate()?;
    drop(db);

    let bot = bot::MyBot::new(token).await.expect("Error creating bot");
    
    let (handle, _) = bot.spawn();

    http::start_http(token_clone.clone()).await?;

    handle.await.expect("Bot task failed");

    Ok(())
}
