use std::fs::File;
use itertools::Itertools;
use avro_rs::Writer;
use crate::{batch_insert, models, run_query};
use tempfile::NamedTempFile;
use std::io::{self, Write};
use mysql_async::prelude::Query;
use tempfile::Builder;

pub(crate) async fn store_chunks(
    pool: &mysql_async::Pool, //&sqlx::Pool<sqlx::MySql>,
    shards: &[near_indexer_primitives::IndexerShard],
    block_hash: &near_indexer_primitives::CryptoHash,
) -> anyhow::Result<()> {
    let schema = models::chunks::Chunk::schema();
    let mut writer = Writer::new(&schema, Vec::new());
    if shards.is_empty() {
        return Ok(())
    }


    let chunk_models: Vec<models::chunks::Chunk> = shards
        .iter()
        .filter_map(|shard| shard.chunk.as_ref())
        .map(|chunk| {
            let a = models::chunks::Chunk::from_chunk_view(chunk, block_hash);
            writer.append_ser(&a).unwrap();
            a
        }).collect();

    let input = writer.into_inner()?;
    let mut file = File::create("/Users/telezhnaya/dev/near-lake-flows-into-sql/foo.avro")?;

    // let mut file = Builder::new()
    //     .prefix("my-temporary-note")
    //     .suffix(".avro")
    //     .rand_bytes(5)
    //     .tempfile()?;
    file.write(&input)?;


    // in-memory pipe and streams

    let query = format!("
                         LOAD DATA LOCAL INFILE \"{}\"
                         INTO TABLE chunks
                         FORMAT AVRO
                             ( {} )
		                 ERRORS HANDLE \"chunks\"
                         ",  "/Users/telezhnaya/dev/near-lake-flows-into-sql/foo.avro"//file.path().to_str().unwrap()
                        ,models::chunks::Chunk::format()).;


    run_query!(&pool.clone(), query);
    // let a = sqlx::query(&query).execute(pool).await;
    println!("YAAY");
    // batch_insert!(&pool.clone(), "INSERT INTO chunks VALUES {}", chunk_models);


    // https://github.com/launchbadge/sqlx/blob/593364f801bba53842265f32bcd34b74fa5c7593/sqlx-core/src/mysql/connection/executor.rs#L129
    //
    Ok(())
}
