mod models;
mod db_adapters;

use bigdecimal::BigDecimal;
use dotenv::dotenv;
use futures::StreamExt;
use near_indexer_primitives;
use near_lake_framework::LakeConfig;
use num_traits::cast::FromPrimitive;
use serde::{Deserialize, Serialize};
use sqlx::mysql::{MySqlConnectOptions, MySqlPoolOptions};
use sqlx::{FromRow, MySql, MySqlPool, Pool, Row};
use std::convert::TryFrom;
use std::env;
use std::str::FromStr;
use tracing_subscriber::EnvFilter;
use crate::models::Block;


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    let pool = MySqlPool::connect(&env::var("DATABASE_URL")?).await?;
    // TODO Error: while executing migrations: error returned from database: 1128 (HY000): Function 'near_indexer.GET_LOCK' is not defined
    // sqlx::migrate!().run(&pool).await?;

    init_tracing();

    let config = LakeConfig {
        s3_bucket_name: "near-lake-data-testnet".to_string(),
        s3_region_name: "eu-central-1".to_string(),
        start_block_height: 83030086, // want to start from the freshest
    };
    let stream = near_lake_framework::streamer(config);

    let mut handlers = tokio_stream::wrappers::ReceiverStream::new(stream)
        .map(|streamer_message| handle_streamer_message(streamer_message, &pool))
        .buffer_unordered(1usize);

    while let Some(_handle_message) = handlers.next().await {}

    Ok(())
}

async fn handle_streamer_message(
    streamer_message: near_lake_framework::near_indexer_primitives::StreamerMessage,
    pool: &Pool<MySql>,
) {
    let block_model = Block::from(&streamer_message.block);
    eprintln!(
        "{} / shards {}",
        streamer_message.block.header.height,
        streamer_message.shards.len()
    );
    // TODO find a better way to insert the objects to the DB
    let new_user = sqlx::query!(
        r#"
       INSERT INTO blocks
       VALUES (?, ?, ?, ?, ?, ?, ?)
       "#,
        block_model.block_height,
        block_model.block_hash,
        block_model.prev_block_hash,
        block_model.block_timestamp,
        block_model.total_supply,
        block_model.gas_price,
        block_model.author_account_id
    );
    let a = new_user.fetch_all(&pool.clone()).await;
}

fn init_tracing() {
    let mut env_filter = EnvFilter::new("near_lake_framework=info");

    if let Ok(rust_log) = std::env::var("RUST_LOG") {
        if !rust_log.is_empty() {
            for directive in rust_log.split(',').filter_map(|s| match s.parse() {
                Ok(directive) => Some(directive),
                Err(err) => {
                    eprintln!("Ignoring directive `{}`: {}", s, err);
                    None
                }
            }) {
                env_filter = env_filter.add_directive(directive);
            }
        }
    }

    tracing_subscriber::fmt::Subscriber::builder()
        .with_env_filter(env_filter)
        .with_writer(std::io::stderr)
        .init();
}
