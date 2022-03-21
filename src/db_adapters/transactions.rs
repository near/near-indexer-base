use cached::Cached;
use futures::future::try_join_all;
use itertools::Itertools;

use crate::{batch_insert, models};

/// Saves Transactions to database
pub(crate) async fn store_transactions(
    pool: &sqlx::Pool<sqlx::MySql>,
    shards: &[near_indexer_primitives::IndexerShard],
    block_hash: &near_indexer_primitives::CryptoHash,
    block_timestamp: u64,
    receipts_cache: crate::ReceiptsCache,
) -> anyhow::Result<()> {
    let tx_futures = shards
        .iter()
        .filter_map(|shard| shard.chunk.as_ref())
        .map(|chunk| {
            store_chunk_transactions(
                pool,
                chunk.transactions.iter().enumerate().collect::<Vec<(
                    usize,
                    &near_indexer_primitives::IndexerTransactionWithOutcome,
                )>>(),
                &chunk.header.chunk_hash,
                block_hash,
                block_timestamp,
                "",
                receipts_cache.clone(),
            )
        });

    try_join_all(tx_futures).await?;
    Ok(())
}

async fn store_chunk_transactions(
    pool: &sqlx::Pool<sqlx::MySql>,
    transactions: Vec<(
        usize,
        &near_indexer_primitives::IndexerTransactionWithOutcome,
    )>,
    chunk_hash: &near_indexer_primitives::CryptoHash,
    block_hash: &near_indexer_primitives::CryptoHash,
    block_timestamp: u64,
    // hack for supporting duplicated transaction hashes. Empty for most of transactions
    // TODO it's a rudiment of previous solution. Create the solution again
    transaction_hash_suffix: &str,
    receipts_cache: crate::ReceiptsCache,
) -> anyhow::Result<()> {
    let mut receipts_cache_lock = receipts_cache.lock().await;

    let transaction_models: Vec<models::Transaction> = transactions
        .iter()
        .map(|(index, tx)| {
            let transaction_hash = tx.transaction.hash.to_string() + transaction_hash_suffix;
            let converted_into_receipt_id = tx
                .outcome
                .execution_outcome
                .outcome
                .receipt_ids
                .first()
                .expect("`receipt_ids` must contain one Receipt Id");

            // Save this Transaction hash to ReceiptsCache
            // we use the Receipt ID to which this transaction was converted
            // and the Transaction hash as a value.
            // Later, while Receipt will be looking for a parent Transaction hash
            // it will be able to find it in the ReceiptsCache
            receipts_cache_lock.cache_set(
                crate::ReceiptOrDataId::ReceiptId(*converted_into_receipt_id),
                transaction_hash.clone(),
            );

            models::Transaction::from_indexer_transaction(
                tx,
                &transaction_hash,
                &converted_into_receipt_id.to_string(),
                block_hash,
                chunk_hash,
                block_timestamp,
                *index as i32,
            )
        })
        .collect();

    // releasing the lock
    drop(receipts_cache_lock);

    batch_insert!(
        &pool.clone(),
        "INSERT INTO transactions VALUES {}",
        transaction_models
    );
    Ok(())
}
