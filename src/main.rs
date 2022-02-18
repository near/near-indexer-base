use futures::StreamExt;
use tracing_subscriber::EnvFilter;
use sqlx::mysql::MySqlPoolOptions;

use near_lake_framework::LakeConfig;
use sqlx::MySqlPool;

#[tokio::main]
async fn main() -> Result<(), tokio::io::Error> {
    let pool = MySqlPool::connect("mysql://admin:'password-in-quotes-because-it-should-be-escaped'@svc-49a0b120-c01e-4247-acc8-88be43f66613-ddl.aws-frankfurt-1.svc.singlestore.com:3306/near_indexer").await;

    let a = pool.unwrap();
    // init_tracing();
    //
    // let config = LakeConfig {
    //     s3_bucket_name: "near-lake-testnet".to_string(),
    //     s3_region_name: "eu-central-1".to_string(),
    //     start_block_height: 83030086, // want to start from the freshest
    // };
    // let stream = near_lake_framework::streamer(config);

    // let mut handlers = tokio_stream::wrappers::ReceiverStream::new(stream)
    //     .map(|streamer_message| handle_streamer_message(streamer_message))
    //     .buffer_unordered(1usize);
    //
    // while let Some(_handle_message) = handlers.next().await {}

    Ok(())
}

// async fn handle_streamer_message(
//     streamer_message: near_lake_framework::near_indexer_primitives::StreamerMessage,
// ) {
//     // eprintln!("{:#?}", &streamer_message);
//     eprintln!(
//         "{} / shards {}",
//         streamer_message.block.header.height,
//         streamer_message.shards.len()
//     );
// }

// fn init_tracing() {
//     let mut env_filter = EnvFilter::new(
//         "near_lake_framework=info",
//     );
//
//     if let Ok(rust_log) = std::env::var("RUST_LOG") {
//         if !rust_log.is_empty() {
//             for directive in rust_log.split(',').filter_map(|s| match s.parse() {
//                 Ok(directive) => Some(directive),
//                 Err(err) => {
//                     eprintln!("Ignoring directive `{}`: {}", s, err);
//                     None
//                 }
//             }) {
//                 env_filter = env_filter.add_directive(directive);
//             }
//         }
//     }
//
//     tracing_subscriber::fmt::Subscriber::builder()
//         .with_env_filter(env_filter)
//         .with_writer(std::io::stderr)
//         .init();
// }
