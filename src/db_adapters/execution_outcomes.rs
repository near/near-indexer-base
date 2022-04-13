use cached::Cached;
use futures::future::try_join_all;

use crate::models;

pub(crate) async fn store_execution_outcomes(
    pool: &sqlx::Pool<sqlx::MySql>,
    shards: &[near_indexer_primitives::IndexerShard],
    block_hash: &near_indexer_primitives::CryptoHash,
    block_timestamp: u64,
    receipts_cache: crate::ReceiptsCache,
) -> anyhow::Result<()> {
    let futures = shards.iter().map(|shard| {
        store_execution_outcomes_for_chunk(
            pool,
            &shard.receipt_execution_outcomes,
            shard.shard_id,
            block_hash,
            block_timestamp,
            std::sync::Arc::clone(&receipts_cache),
        )
    });

    try_join_all(futures).await.map(|_| ())
}

/// Saves ExecutionOutcome to database and then saves ExecutionOutcomesReceipts
pub async fn store_execution_outcomes_for_chunk(
    pool: &sqlx::Pool<sqlx::MySql>,
    execution_outcomes: &[near_indexer_primitives::IndexerExecutionOutcomeWithReceipt],
    shard_id: near_indexer_primitives::types::ShardId,
    block_hash: &near_indexer_primitives::CryptoHash,
    block_timestamp: u64,
    receipts_cache: crate::ReceiptsCache,
) -> anyhow::Result<()> {
    let mut outcome_models: Vec<models::ExecutionOutcome> = vec![];
    let mut outcome_receipt_models: Vec<models::ExecutionOutcomeReceipt> = vec![];
    let mut receipts_cache_lock = receipts_cache.lock().await;
    let mut index_through_chunk = -1;
    for (index_in_chunk, outcome) in execution_outcomes.iter().enumerate() {
        // Trying to take the parent Transaction hash for the Receipt from ReceiptsCache
        // remove it from cache once found as it is not expected to observe the Receipt for
        // second time
        let parent_transaction_hash = receipts_cache_lock.cache_remove(
            &crate::ReceiptOrDataId::ReceiptId(outcome.execution_outcome.id),
        );

        let model = models::ExecutionOutcome::from_execution_outcome(
            &outcome.execution_outcome,
            index_in_chunk as i32,
            block_timestamp,
            shard_id,
        );
        outcome_models.push(model);

        outcome_receipt_models.extend(outcome.execution_outcome.outcome.receipt_ids.iter().map(
            |receipt_id| {
                // if we have `parent_transaction_hash` from cache, then we put all "produced" Receipt IDs
                // as key and `parent_transaction_hash` as value, so the Receipts from one of the next blocks
                // could find their parents in cache
                if let Some(transaction_hash) = &parent_transaction_hash {
                    receipts_cache_lock.cache_set(
                        crate::ReceiptOrDataId::ReceiptId(*receipt_id),
                        transaction_hash.clone(),
                    );
                }
                index_through_chunk += 1;
                models::ExecutionOutcomeReceipt {
                    block_hash: block_hash.to_string(),
                    block_timestamp: block_timestamp.into(),
                    executed_receipt_id: outcome.execution_outcome.id.to_string(),
                    produced_receipt_id: receipt_id.to_string(),
                    chunk_index_in_block: shard_id as i32,
                    index_in_chunk: index_through_chunk,
                }
            },
        ));
    }

    drop(receipts_cache_lock);

    for execution_outcomes_part in
        outcome_models.chunks(crate::db_adapters::CHUNK_SIZE_FOR_BATCH_INSERT)
    {
        let mut args = sqlx::mysql::MySqlArguments::default();
        let mut execution_outcomes_count = 0;

        execution_outcomes_part
            .iter()
            .for_each(|execution_outcome| {
                execution_outcome.add_to_args(&mut args);
                execution_outcomes_count += 1;
            });

        let query = models::ExecutionOutcome::get_query(execution_outcomes_count)?;
        sqlx::query_with(&query, args).execute(pool).await?;
    }

    for outcome_receipts_part in
        outcome_receipt_models.chunks(crate::db_adapters::CHUNK_SIZE_FOR_BATCH_INSERT)
    {
        let mut args = sqlx::mysql::MySqlArguments::default();
        let mut outcome_receipts_count = 0;

        outcome_receipts_part.iter().for_each(|outcome_receipt| {
            outcome_receipt.add_to_args(&mut args);
            outcome_receipts_count += 1;
        });

        let query = models::ExecutionOutcomeReceipt::get_query(outcome_receipts_count)?;
        sqlx::query_with(&query, args).execute(pool).await?;
    }

    Ok(())
}
