use crate::{models, ParentTransactionHashString, ReceiptOrDataId};
use cached::Cached;
use futures::future::try_join_all;
use futures::try_join;
use itertools::{Either, Itertools};
use num_traits::FromPrimitive;
use sqlx::{Arguments, Row};
use std::collections::HashMap;
use std::str::FromStr;
use tracing::warn;

/// Saves receipts to database
pub(crate) async fn store_receipts(
    pool: &sqlx::Pool<sqlx::MySql>,
    strict_mode: bool,
    shards: &[near_indexer_primitives::IndexerShard],
    block_hash: &near_indexer_primitives::CryptoHash,
    block_timestamp: u64,
    block_height: u64,
    receipts_cache: crate::ReceiptsCache,
) -> anyhow::Result<()> {
    let futures = shards
        .iter()
        .filter_map(|shard| shard.chunk.as_ref())
        .filter(|chunk| !chunk.receipts.is_empty())
        .map(|chunk| {
            store_chunk_receipts(
                pool,
                strict_mode,
                &chunk.receipts,
                block_hash,
                &chunk.header.chunk_hash,
                block_timestamp,
                block_height,
                receipts_cache.clone(),
            )
        });

    try_join_all(futures).await.map(|_| ())
}

async fn store_chunk_receipts(
    pool: &sqlx::Pool<sqlx::MySql>,
    strict_mode: bool,
    receipts: &[near_indexer_primitives::views::ReceiptView],
    block_hash: &near_indexer_primitives::CryptoHash,
    chunk_hash: &near_indexer_primitives::CryptoHash,
    block_timestamp: u64,
    block_height: u64,
    receipts_cache: crate::ReceiptsCache,
) -> anyhow::Result<()> {
    let tx_hashes_for_receipts: HashMap<ReceiptOrDataId, ParentTransactionHashString> =
        find_tx_hashes_for_receipts(
            pool,
            strict_mode,
            receipts.to_vec(),
            block_hash,
            block_height,
            chunk_hash,
            std::sync::Arc::clone(&receipts_cache),
        )
        .await?;

    // At the moment we can observe output data in the Receipt it's impossible to know
    // the Receipt Id of that Data Receipt. That's why we insert the pair DataId<>ParentTransactionHash
    // to ReceiptsCache
    let mut receipts_cache_lock = receipts_cache.lock().await;
    for receipt in receipts {
        if let near_indexer_primitives::views::ReceiptEnumView::Action {
            output_data_receivers,
            ..
        } = &receipt.receipt
        {
            if !output_data_receivers.is_empty() {
                if let Some(transaction_hash) = tx_hashes_for_receipts
                    .get(&crate::ReceiptOrDataId::ReceiptId(receipt.receipt_id))
                {
                    for data_receiver in output_data_receivers {
                        receipts_cache_lock.cache_set(
                            crate::ReceiptOrDataId::DataId(data_receiver.data_id),
                            transaction_hash.clone(),
                        );
                    }
                }
            }
        }
    }
    // releasing the lock
    drop(receipts_cache_lock);

    let (action_receipts, data_receipts): (
        Vec<(usize, &near_indexer_primitives::views::ReceiptView)>,
        Vec<models::DataReceipt>,
    ) = receipts
        .iter()
        .enumerate()
        .partition_map(|(index, receipt)| match receipt.receipt {
            near_indexer_primitives::views::ReceiptEnumView::Action { .. } => {
                Either::Left((index, receipt))
            }
            near_indexer_primitives::views::ReceiptEnumView::Data { data_id, .. } => {
                // todo it's not good to do it like that. get rid of expect
                let transaction_hash = tx_hashes_for_receipts
                    .get(&crate::ReceiptOrDataId::DataId(data_id))
                    .expect("");
                Either::Right(
                    models::DataReceipt::try_from_data_receipt_view(
                        receipt,
                        block_hash,
                        transaction_hash,
                        chunk_hash,
                        index as i32,
                        block_timestamp,
                    )
                    .expect("DataReceipt should be converted smoothly"),
                )
            }
        });

    let process_receipt_actions_future = store_receipt_actions(
        pool,
        action_receipts,
        &tx_hashes_for_receipts,
        block_hash,
        chunk_hash,
        block_timestamp,
    );

    let process_receipt_data_future = store_data_receipts(pool, &data_receipts);

    try_join!(process_receipt_actions_future, process_receipt_data_future)?;
    Ok(())
}

/// Looks for already created parent transaction hash for given receipts
async fn find_tx_hashes_for_receipts(
    pool: &sqlx::Pool<sqlx::MySql>,
    strict_mode: bool,
    mut receipts: Vec<near_indexer_primitives::views::ReceiptView>,
    // TODO we need to add sort of retry logic, these vars could be helpful
    block_hash: &near_indexer_primitives::CryptoHash,
    block_height: u64,
    chunk_hash: &near_indexer_primitives::CryptoHash,
    receipts_cache: crate::ReceiptsCache,
) -> anyhow::Result<HashMap<crate::ReceiptOrDataId, crate::ParentTransactionHashString>> {
    let mut tx_hashes_for_receipts: HashMap<
        crate::ReceiptOrDataId,
        crate::ParentTransactionHashString,
    > = HashMap::new();

    let mut receipts_cache_lock = receipts_cache.lock().await;
    // add receipt-transaction pairs from the cache to the response
    tx_hashes_for_receipts.extend(receipts.iter().filter_map(|receipt| {
        match receipt.receipt {
            near_indexer_primitives::views::ReceiptEnumView::Action { .. } => receipts_cache_lock
                .cache_get(&crate::ReceiptOrDataId::ReceiptId(receipt.receipt_id))
                .map(|parent_transaction_hash| {
                    (
                        crate::ReceiptOrDataId::ReceiptId(receipt.receipt_id),
                        parent_transaction_hash.clone(),
                    )
                }),
            near_indexer_primitives::views::ReceiptEnumView::Data { data_id, .. } => {
                // Pair DataId:ParentTransactionHash won't be used after this moment
                // We want to clean it up to prevent our cache from growing
                receipts_cache_lock
                    .cache_remove(&crate::ReceiptOrDataId::DataId(data_id))
                    .map(|parent_transaction_hash| {
                        (
                            crate::ReceiptOrDataId::DataId(data_id),
                            parent_transaction_hash,
                        )
                    })
            }
        }
    }));
    // releasing the lock
    drop(receipts_cache_lock);

    // discard the Receipts already in cache from the attempts to search
    receipts.retain(|r| match r.receipt {
        near_indexer_primitives::views::ReceiptEnumView::Data { data_id, .. } => {
            !tx_hashes_for_receipts.contains_key(&crate::ReceiptOrDataId::DataId(data_id))
        }
        near_indexer_primitives::views::ReceiptEnumView::Action { .. } => {
            !tx_hashes_for_receipts.contains_key(&crate::ReceiptOrDataId::ReceiptId(r.receipt_id))
        }
    });

    if receipts.is_empty() {
        return Ok(tx_hashes_for_receipts);
    }

    warn!(
        target: crate::INDEXER,
        "Looking for parent transaction hash in database for {} receipts {:#?}",
        &receipts.len(),
        &receipts,
    );

    let (action_receipt_ids, data_ids): (Vec<String>, Vec<String>) =
        receipts.iter().partition_map(|r| match r.receipt {
            near_indexer_primitives::views::ReceiptEnumView::Action { .. } => {
                Either::Left(r.receipt_id.to_string())
            }
            near_indexer_primitives::views::ReceiptEnumView::Data { data_id, .. } => {
                Either::Right(data_id.to_string())
            }
        });

    if !data_ids.is_empty() {
        let tx_hashes_for_data_receipts =
            find_transaction_hashes_for_data_receipts(pool, &data_ids, &receipts).await?;
        tx_hashes_for_receipts.extend(tx_hashes_for_data_receipts.clone());

        receipts.retain(|r| {
            !tx_hashes_for_data_receipts
                .contains_key(&crate::ReceiptOrDataId::ReceiptId(r.receipt_id))
        });
        if receipts.is_empty() {
            return Ok(tx_hashes_for_receipts);
        }
    }

    let tx_hashes_for_receipts_via_outcomes =
        find_transaction_hashes_for_receipts_via_outcomes(pool, &action_receipt_ids).await?;
    tx_hashes_for_receipts.extend(tx_hashes_for_receipts_via_outcomes.clone());

    receipts.retain(|r| {
        !tx_hashes_for_receipts_via_outcomes
            .contains_key(&crate::ReceiptOrDataId::ReceiptId(r.receipt_id))
    });
    if receipts.is_empty() {
        return Ok(tx_hashes_for_receipts);
    }

    let tx_hashes_for_receipt_via_transactions =
        find_transaction_hashes_for_receipt_via_transactions(pool, &action_receipt_ids).await?;
    tx_hashes_for_receipts.extend(tx_hashes_for_receipt_via_transactions.clone());

    receipts.retain(|r| {
        !tx_hashes_for_receipt_via_transactions
            .contains_key(&crate::ReceiptOrDataId::ReceiptId(r.receipt_id))
    });
    if !receipts.is_empty() {
        if strict_mode {
            panic!("all the transactions should be found by this place");
        }
        eprintln!(
            "The block {} has {} receipt(s) we still need to put to the DB later",
            block_height,
            receipts.len()
        );
        let mut args = sqlx::mysql::MySqlArguments::default();
        args.add(block_height);
        let query = "INSERT IGNORE INTO _blocks_to_rerun VALUES (?)";
        sqlx::query_with(query, args).execute(pool).await?;
    }

    Ok(tx_hashes_for_receipts)
}

async fn find_transaction_hashes_for_data_receipts(
    pool: &sqlx::Pool<sqlx::MySql>,
    data_ids: &[String],
    receipts: &[near_indexer_primitives::views::ReceiptView],
) -> anyhow::Result<HashMap<crate::ReceiptOrDataId, crate::ParentTransactionHashString>> {
    let query = "SELECT action_receipt_output_data.output_data_id, receipts.originated_from_transaction_hash
                        FROM action_receipt_output_data JOIN receipts ON action_receipt_output_data.output_from_receipt_id = receipts.receipt_id
                        WHERE action_receipt_output_data.output_data_id IN ".to_owned() + &crate::models::create_placeholder(data_ids.len())?;
    let mut args = sqlx::mysql::MySqlArguments::default();
    data_ids.iter().for_each(|data_id| {
        args.add(data_id);
    });

    let res = sqlx::query_with(&query, args).fetch_all(pool).await?;

    let tx_hashes_for_data_id_via_data_output_hashmap: HashMap<
        crate::ReceiptOrDataId,
        crate::ParentTransactionHashString,
    > = res
        .iter()
        .map(|q| (q.get(0), q.get(1)))
        .map(
            |(receipt_id_string, transaction_hash_string): (String, String)| {
                (
                    crate::ReceiptOrDataId::DataId(
                        near_indexer_primitives::CryptoHash::from_str(&receipt_id_string)
                            .expect("Failed to convert String to CryptoHash"),
                    ),
                    transaction_hash_string,
                )
            },
        )
        .collect();

    Ok(receipts
        .iter()
        .filter_map(|r| match r.receipt {
            near_indexer_primitives::views::ReceiptEnumView::Data { data_id, .. } => {
                tx_hashes_for_data_id_via_data_output_hashmap
                    .get(&crate::ReceiptOrDataId::DataId(data_id))
                    .map(|tx_hash| {
                        (
                            crate::ReceiptOrDataId::ReceiptId(r.receipt_id),
                            tx_hash.to_string(),
                        )
                    })
            }
            _ => None,
        })
        .collect())
}

async fn find_transaction_hashes_for_receipts_via_outcomes(
    pool: &sqlx::Pool<sqlx::MySql>,
    action_receipt_ids: &[String],
) -> anyhow::Result<HashMap<crate::ReceiptOrDataId, crate::ParentTransactionHashString>> {
    let query = "SELECT execution_outcome_receipts.produced_receipt_id, receipts.originated_from_transaction_hash
                        FROM execution_outcome_receipts JOIN receipts ON execution_outcome_receipts.executed_receipt_id = receipts.receipt_id
                        WHERE execution_outcome_receipts.produced_receipt_id IN ".to_owned() + &crate::models::create_placeholder(action_receipt_ids.len())?;
    let mut args = sqlx::mysql::MySqlArguments::default();
    action_receipt_ids.iter().for_each(|data_id| {
        args.add(data_id);
    });

    let res = sqlx::query_with(&query, args).fetch_all(pool).await?;

    Ok(res
        .iter()
        .map(|q| (q.get(0), q.get(1)))
        .map(
            |(receipt_id_string, transaction_hash_string): (String, String)| {
                (
                    crate::ReceiptOrDataId::ReceiptId(
                        near_indexer_primitives::CryptoHash::from_str(&receipt_id_string)
                            .expect("Failed to convert String to CryptoHash"),
                    ),
                    transaction_hash_string,
                )
            },
        )
        .collect())
}

async fn find_transaction_hashes_for_receipt_via_transactions(
    pool: &sqlx::Pool<sqlx::MySql>,
    action_receipt_ids: &[String],
) -> anyhow::Result<HashMap<crate::ReceiptOrDataId, crate::ParentTransactionHashString>> {
    let query = "SELECT converted_into_receipt_id, transaction_hash
                        FROM transactions
                        WHERE converted_into_receipt_id IN "
        .to_owned()
        + &crate::models::create_placeholder(action_receipt_ids.len())?;
    let mut args = sqlx::mysql::MySqlArguments::default();
    action_receipt_ids.iter().for_each(|data_id| {
        args.add(data_id);
    });

    let res = sqlx::query_with(&query, args).fetch_all(pool).await?;

    Ok(res
        .iter()
        .map(|q| (q.get(0), q.get(1)))
        .map(
            |(receipt_id_string, transaction_hash_string): (String, String)| {
                (
                    crate::ReceiptOrDataId::ReceiptId(
                        near_indexer_primitives::CryptoHash::from_str(&receipt_id_string)
                            .expect("Failed to convert String to CryptoHash"),
                    ),
                    transaction_hash_string,
                )
            },
        )
        .collect())
}

async fn store_receipt_actions(
    pool: &sqlx::Pool<sqlx::MySql>,
    receipts: Vec<(usize, &near_indexer_primitives::views::ReceiptView)>,
    tx_hashes_for_receipts: &HashMap<ReceiptOrDataId, ParentTransactionHashString>,
    block_hash: &near_indexer_primitives::CryptoHash,
    chunk_hash: &near_indexer_primitives::CryptoHash,
    block_timestamp: u64,
) -> anyhow::Result<()> {
    let receipt_actions: Vec<models::ActionReceipt> = receipts
        .iter()
        .filter_map(|(index, receipt)| {
            let transaction_hash = tx_hashes_for_receipts
                .get(&crate::ReceiptOrDataId::ReceiptId(receipt.receipt_id))
                .expect("");
            models::ActionReceipt::try_from_action_receipt_view(
                receipt,
                block_hash,
                transaction_hash,
                chunk_hash,
                *index as i32,
                block_timestamp,
            )
            .ok()
        })
        .collect();

    let receipt_action_actions: Vec<models::ActionReceiptAction> = receipts
        .iter()
        .filter_map(|(_, receipt)| {
            if let near_indexer_primitives::views::ReceiptEnumView::Action { actions, .. } =
                &receipt.receipt
            {
                Some(actions.iter().enumerate().map(move |(index, action)| {
                    models::ActionReceiptAction::from_action_view(
                        receipt.receipt_id.to_string(),
                        i32::from_usize(index).expect("We expect usize to not overflow i32 here"),
                        action,
                        receipt.predecessor_id.to_string(),
                        receipt.receiver_id.to_string(),
                        block_timestamp,
                    )
                }))
            } else {
                None
            }
        })
        .flatten()
        .collect();

    let receipt_action_input_data: Vec<models::ActionReceiptInputData> = receipts
        .iter()
        .filter_map(|(_, receipt)| {
            if let near_indexer_primitives::views::ReceiptEnumView::Action {
                input_data_ids, ..
            } = &receipt.receipt
            {
                Some(input_data_ids.iter().map(move |data_id| {
                    models::ActionReceiptInputData::from_data_id(
                        block_timestamp,
                        receipt.receipt_id.to_string(),
                        data_id.to_string(),
                    )
                }))
            } else {
                None
            }
        })
        .flatten()
        .collect();

    let receipt_action_output_data: Vec<models::ActionReceiptOutputData> = receipts
        .iter()
        .filter_map(|(_, receipt)| {
            if let near_indexer_primitives::views::ReceiptEnumView::Action {
                output_data_receivers,
                ..
            } = &receipt.receipt
            {
                Some(output_data_receivers.iter().map(move |receiver| {
                    models::ActionReceiptOutputData::from_data_receiver(
                        block_timestamp,
                        receipt.receipt_id.to_string(),
                        receiver,
                    )
                }))
            } else {
                None
            }
        })
        .flatten()
        .collect();

    try_join!(
        store_action_receipts(pool, &receipt_actions),
        store_action_receipt_actions(pool, &receipt_action_actions),
        store_action_receipts_input_data(pool, &receipt_action_input_data),
        store_action_receipts_output_data(pool, &receipt_action_output_data),
    )?;

    Ok(())
}

async fn store_action_receipts(
    pool: &sqlx::Pool<sqlx::MySql>,
    receipts: &[models::ActionReceipt],
) -> anyhow::Result<()> {
    for action_receipts_part in receipts.chunks(crate::db_adapters::CHUNK_SIZE_FOR_BATCH_INSERT) {
        let mut args = sqlx::mysql::MySqlArguments::default();
        let mut action_receipts_count = 0;

        action_receipts_part.iter().for_each(|action_receipt| {
            action_receipt.add_to_args(&mut args);
            action_receipts_count += 1;
        });

        let query = models::receipts::ActionReceipt::get_query(action_receipts_count)?;
        sqlx::query_with(&query, args).execute(pool).await?;
    }

    Ok(())
}

async fn store_action_receipt_actions(
    pool: &sqlx::Pool<sqlx::MySql>,
    receipts: &[models::ActionReceiptAction],
) -> anyhow::Result<()> {
    for action_receipts_part in receipts.chunks(crate::db_adapters::CHUNK_SIZE_FOR_BATCH_INSERT) {
        let mut args = sqlx::mysql::MySqlArguments::default();
        let mut action_receipts_count = 0;

        action_receipts_part.iter().for_each(|action_receipt| {
            action_receipt.add_to_args(&mut args);
            action_receipts_count += 1;
        });

        let query = models::receipts::ActionReceiptAction::get_query(action_receipts_count)?;
        sqlx::query_with(&query, args).execute(pool).await?;
    }

    Ok(())
}

async fn store_action_receipts_input_data(
    pool: &sqlx::Pool<sqlx::MySql>,
    receipts: &[models::ActionReceiptInputData],
) -> anyhow::Result<()> {
    for action_receipts_part in receipts.chunks(crate::db_adapters::CHUNK_SIZE_FOR_BATCH_INSERT) {
        let mut args = sqlx::mysql::MySqlArguments::default();
        let mut action_receipts_count = 0;

        action_receipts_part.iter().for_each(|action_receipt| {
            action_receipt.add_to_args(&mut args);
            action_receipts_count += 1;
        });

        let query = models::receipts::ActionReceiptInputData::get_query(action_receipts_count)?;
        sqlx::query_with(&query, args).execute(pool).await?;
    }

    Ok(())
}

async fn store_action_receipts_output_data(
    pool: &sqlx::Pool<sqlx::MySql>,
    receipts: &[models::ActionReceiptOutputData],
) -> anyhow::Result<()> {
    for action_receipts_part in receipts.chunks(crate::db_adapters::CHUNK_SIZE_FOR_BATCH_INSERT) {
        let mut args = sqlx::mysql::MySqlArguments::default();
        let mut action_receipts_count = 0;

        action_receipts_part.iter().for_each(|action_receipt| {
            action_receipt.add_to_args(&mut args);
            action_receipts_count += 1;
        });

        let query = models::receipts::ActionReceiptOutputData::get_query(action_receipts_count)?;
        sqlx::query_with(&query, args).execute(pool).await?;
    }

    Ok(())
}

async fn store_data_receipts(
    pool: &sqlx::Pool<sqlx::MySql>,
    receipts: &[models::DataReceipt],
) -> anyhow::Result<()> {
    for data_receipts_part in receipts.chunks(crate::db_adapters::CHUNK_SIZE_FOR_BATCH_INSERT) {
        let mut args = sqlx::mysql::MySqlArguments::default();
        let mut data_receipts_count = 0;

        data_receipts_part.iter().for_each(|data_receipt| {
            data_receipt.add_to_args(&mut args);
            data_receipts_count += 1;
        });

        let query = models::receipts::DataReceipt::get_query(data_receipts_count)?;
        sqlx::query_with(&query, args).execute(pool).await?;
    }

    Ok(())
}
