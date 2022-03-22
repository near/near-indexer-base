// TODO cleanup imports in all the files in the end
mod db_adapters;
mod models;
mod utils;

use cached::SizedCache;
use dotenv::dotenv;
use futures::{try_join, StreamExt};
use near_lake_framework::LakeConfig;
use std::env;
use futures::future::try_join_all;
use tokio::sync::Mutex;
use tracing_subscriber::EnvFilter;

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub enum ReceiptOrDataId {
    ReceiptId(near_indexer_primitives::CryptoHash),
    DataId(near_indexer_primitives::CryptoHash),
}
// Creating type aliases to make HashMap types for cache more explicit
pub type ParentTransactionHashString = String;
// Introducing a simple cache for Receipts to find their parent Transactions without
// touching the database
// The key is ReceiptID
// The value is TransactionHash (the very parent of the Receipt)
pub type ReceiptsCache =
    std::sync::Arc<Mutex<SizedCache<ReceiptOrDataId, ParentTransactionHashString>>>;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    let pool = sqlx::MySqlPool::connect(&env::var("DATABASE_URL")?).await?;
    // TODO Error: while executing migrations: error returned from database: 1128 (HY000): Function 'near_indexer.GET_LOCK' is not defined
    // sqlx::migrate!().run(&pool).await?;

    init_tracing();

    let config = LakeConfig {
        // //  for testnet, lake starts streaming from 42839521
        //     s3_bucket_name: "near-lake-data-testnet".to_string(),
        //     s3_region_name: "eu-central-1".to_string(),
        //     start_block_height: 42376888 //42376923, // want to start from the first to fill in the cache correctly // 42376888
        s3_bucket_name: "near-lake-data-mainnet".to_string(),
        s3_region_name: "eu-central-1".to_string(),
        start_block_height: 9823031, //9820214, // 9820210 9823031
    };
    let stream = near_lake_framework::streamer(config);

    // We want to prevent unnecessary SELECT queries to the database to find
    // the Transaction hash for the Receipt.
    // Later we need to find the Receipt which is a parent to underlying Receipts.
    // Receipt ID will of the child will be stored as key and parent Transaction hash/Receipt ID
    // will be stored as a value
    let receipts_cache: ReceiptsCache =
        std::sync::Arc::new(Mutex::new(SizedCache::with_size(100_000)));

    // TODO now we ignore the errors here, we never fail. Change it
    let mut handlers = tokio_stream::wrappers::ReceiverStream::new(stream)
        .map(|streamer_message| {
            handle_streamer_message(
                streamer_message,
                &pool,
                std::sync::Arc::clone(&receipts_cache),
            )
        })
        .buffer_unordered(1usize);

    while let Some(_handle_message) = handlers.next().await {}

    Ok(())
}

async fn handle_streamer_message(
    streamer_message: near_lake_framework::near_indexer_primitives::StreamerMessage,
    pool: &sqlx::Pool<sqlx::MySql>,
    receipts_cache: ReceiptsCache,
) -> anyhow::Result<()> {
    // TODO: fault-tolerance
    // we need to have the ability to write the same block again
    // we need to fail if something goes wrong
    eprintln!(
        "{} / shards {}",
        streamer_message.block.header.height,
        streamer_message.shards.len()
    );
    // eprintln!(
    //     "ReceiptsCache #{} \n {:#?}",
    //     streamer_message.block.header.height, &receipts_cache
    // );
    let blocks_future = db_adapters::blocks::store_block(pool, &streamer_message.block);

    let chunks_future = db_adapters::chunks::store_chunks(
        pool,
        &streamer_message.shards,
        &streamer_message.block.header.hash,
    );

    let transactions_future = db_adapters::transactions::store_transactions(
        pool,
        &streamer_message.shards,
        &streamer_message.block.header.hash,
        streamer_message.block.header.timestamp,
        std::sync::Arc::clone(&receipts_cache),
    );

    let receipts_future = db_adapters::receipts::store_receipts(
        pool,
        &streamer_message.shards,
        &streamer_message.block.header.hash,
        streamer_message.block.header.timestamp,
        std::sync::Arc::clone(&receipts_cache),
    );

    let execution_outcomes_future = db_adapters::execution_outcomes::store_execution_outcomes(
        pool,
        &streamer_message.shards,
        streamer_message.block.header.timestamp,
        std::sync::Arc::clone(&receipts_cache),
    );

    let account_changes_future = async {
        let futures = streamer_message.shards.iter().map(|shard| {
            db_adapters::account_changes::store_account_changes(
                pool,
                &shard.state_changes,
                &streamer_message.block.header.hash,
                streamer_message.block.header.timestamp,
            )
        });

        try_join_all(futures).await.map(|_| ())
    };

    try_join!(blocks_future, chunks_future, transactions_future)?;
    try_join!(receipts_future)?; // this guy can contain local receipts, so we have to do that after transactions_future finished the work
    try_join!(execution_outcomes_future, account_changes_future)?; // this guy thinks that receipts_future finished, and clears the cache

    eprintln!("finished");
    Ok(())
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
