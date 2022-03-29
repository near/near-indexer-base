use crate::models;
use crate::db_adapters::create_query_with_placeholders;

pub(crate) async fn store_chunks(
    pool: &sqlx::Pool<sqlx::MySql>,
    shards: &[near_indexer_primitives::IndexerShard],
    block_hash: &near_indexer_primitives::CryptoHash,
) -> anyhow::Result<()> {
    if shards.is_empty() {
        return Ok(())
    }

    let mut args = sqlx::mysql::MySqlArguments::default();
    let mut chunks_count = 0;

    shards // : Vec<models::chunks::Chunk>
        .iter()
        .filter_map(|shard| shard.chunk.as_ref())
        .for_each(|chunk| {
            models::chunks::Chunk::add_to_args(chunk, block_hash, &mut args);
            chunks_count += 1;
        });

    let query = create_query_with_placeholders("INSERT IGNORE INTO chunks VALUES", chunks_count, models::chunks::Chunk::fields_count())?;
    sqlx::query_with(&query, args).execute(pool).await?;

    Ok(())
}
