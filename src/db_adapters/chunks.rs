use itertools::Itertools;

use crate::{batch_insert, models};

pub(crate) async fn store_chunks(
    pool: &sqlx::Pool<sqlx::MySql>,
    shards: &[near_indexer_primitives::IndexerShard],
    block_hash: &near_indexer_primitives::CryptoHash,
) -> anyhow::Result<()> {
    let chunk_models: Vec<models::chunks::Chunk> = shards
        .iter()
        .filter_map(|shard| shard.chunk.as_ref())
        .map(|chunk| models::chunks::Chunk::from_chunk_view(chunk, block_hash))
        .collect();

    batch_insert!(&pool.clone(), "INSERT INTO chunks VALUES {}", chunk_models);
    Ok(())
}
