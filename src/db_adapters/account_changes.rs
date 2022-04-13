use crate::models;

use futures::future::try_join_all;
use itertools::Itertools;

pub(crate) async fn store_account_changes(
    pool: &sqlx::Pool<sqlx::MySql>,
    shards: &[near_indexer_primitives::IndexerShard],
    block_hash: &near_indexer_primitives::CryptoHash,
    block_timestamp: u64,
) -> anyhow::Result<()> {
    let futures = shards.iter().map(|shard| {
        store_account_changes_for_chunk(
            pool,
            &shard.state_changes,
            block_hash,
            block_timestamp,
            shard.shard_id,
        )
    });

    try_join_all(futures).await.map(|_| ())
}

async fn store_account_changes_for_chunk(
    pool: &sqlx::Pool<sqlx::MySql>,
    state_changes: &near_indexer_primitives::views::StateChangesView,
    block_hash: &near_indexer_primitives::CryptoHash,
    block_timestamp: u64,
    shard_id: near_indexer_primitives::types::ShardId,
) -> anyhow::Result<()> {
    if state_changes.is_empty() {
        return Ok(());
    }

    for account_changes_part in &state_changes
        .iter()
        .filter_map(|state_change| {
            models::account_changes::AccountChange::from_state_change_with_cause(
                state_change,
                block_hash,
                block_timestamp,
                shard_id as i32,
                0, // we will fill it later
            )
        })
        .enumerate()
        .chunks(crate::db_adapters::CHUNK_SIZE_FOR_BATCH_INSERT)
    {
        let mut args = sqlx::mysql::MySqlArguments::default();
        let mut changes_count = 0;

        account_changes_part.for_each(|(i, mut account_change)| {
            account_change.index_in_chunk = i as i32;
            account_change.add_to_args(&mut args);
            changes_count += 1;
        });

        let query = models::account_changes::AccountChange::get_query(changes_count)?;
        sqlx::query_with(&query, args).execute(pool).await?;
    }

    Ok(())
}
