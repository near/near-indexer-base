use crate::models;
use itertools::Itertools;

pub(crate) async fn store_chunks(
    pool: &sqlx::Pool<sqlx::MySql>,
    shards: &[near_indexer_primitives::IndexerShard],
    block_hash: &near_indexer_primitives::CryptoHash,
    block_timestamp: u64,
) -> anyhow::Result<()> {
    crate::models::chunked_insert(
        pool,
        &shards
            .iter()
            .filter_map(|shard| {
                shard.chunk.as_ref().and_then(|chunk| {
                    Some(models::chunks::Chunk::from_chunk_view(
                        chunk,
                        block_hash,
                        block_timestamp,
                    ))
                })
            })
            .collect::<Vec<models::chunks::Chunk>>(),
        10,
    )
    .await?;

    Ok(())
}
