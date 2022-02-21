use futures::StreamExt;
use sqlx::mysql::{MySqlConnectOptions, MySqlPoolOptions};
use tracing_subscriber::EnvFilter;

use near_lake_framework::LakeConfig;
use sqlx::{FromRow, MySql, MySqlPool, Pool, Row};

use dotenv::dotenv;
use std::env;

#[derive(Debug, FromRow)]
struct Aaa {
    a: i64,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    let opts = MySqlConnectOptions::new()
        .host(&env::var("SINGLESTORE_HOST")?)
        .username(&env::var("SINGLESTORE_USER")?)
        .password(&env::var("SINGLESTORE_PASSWORD")?)
        .database(&env::var("SINGLESTORE_DATABASE")?);
    let pool = MySqlPoolOptions::new().connect_with(opts).await?;

    // TODO Since it's not possible to create DATABASE_URL properly (we can't escape the characters from password),
    // I can't use sqlx::query_as! macro, right?
    // https://docs.rs/sqlx/0.3.0/sqlx/macro.query.html#requirements
    let select_query = sqlx::query_as::<_, Aaa>("SELECT * FROM aaa");
    let a = select_query.fetch_all(&pool).await?;

    init_tracing();

    let config = LakeConfig {
        s3_bucket_name: "near-lake-testnet".to_string(),
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
    eprintln!(
        "{} / shards {}",
        streamer_message.block.header.height,
        streamer_message.shards.len()
    );
    // TODO find a better way to insert the objects to the DB
    let query = format!(
        r#"
       INSERT INTO aaa
       VALUES ({})
       "#,
        &streamer_message.block.header.height
    );
    let new_user = sqlx::query_as::<_, Aaa>(&query);
    let a = new_user.fetch_all(&pool.clone()).await;
    let b = 0;
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
