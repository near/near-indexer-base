use crate::models;
use cached::Cached;
use futures::future::try_join_all;
use futures::try_join;
use itertools::Itertools;
use num_traits::FromPrimitive;
use std::collections::HashMap;
use tracing::warn;

/// Saves receipts to database
pub(crate) async fn store_receipts(
    pool: &sqlx::Pool<sqlx::MySql>,
    shards: &[near_indexer_primitives::IndexerShard],
    block_hash: &near_indexer_primitives::CryptoHash,
    block_timestamp: u64,
    receipts_cache: crate::ReceiptsCache,
) -> anyhow::Result<()> {
    let futures = shards
        .iter()
        .filter_map(|shard| shard.chunk.as_ref())
        .filter(|chunk| !chunk.receipts.is_empty())
        .map(|chunk| {
            store_chunk_receipts(
                pool,
                &chunk.receipts,
                block_hash,
                &chunk.header.chunk_hash,
                block_timestamp,
                receipts_cache.clone(),
            )
        });

    try_join_all(futures).await.map(|_| ())
}

async fn store_chunk_receipts(
    pool: &sqlx::Pool<sqlx::MySql>,
    receipts: &[near_indexer_primitives::views::ReceiptView],
    block_hash: &near_indexer_primitives::CryptoHash,
    chunk_hash: &near_indexer_primitives::CryptoHash,
    block_timestamp: u64,
    receipts_cache: crate::ReceiptsCache,
) -> anyhow::Result<()> {
    let mut skipping_receipt_ids =
        std::collections::HashSet::<near_indexer_primitives::CryptoHash>::new();

    let tx_hashes_for_receipts = find_tx_hashes_for_receipts(
        pool,
        receipts.to_vec(),
        block_hash,
        chunk_hash,
        std::sync::Arc::clone(&receipts_cache),
    )
    .await?;

    let receipt_models: Vec<models::Receipt> = receipts
        .iter()
        .enumerate()
        .filter_map(|(index, r)| {
            // We need to search for parent transaction hash in cache differently
            // depending on the Receipt kind
            // In case of Action Receipt we are looking for ReceiptId
            // In case of Data Receipt we are looking for DataId
            let receipt_or_data_id = match r.receipt {
                near_indexer_primitives::views::ReceiptEnumView::Action { .. } => {
                    crate::ReceiptOrDataId::ReceiptId(r.receipt_id)
                }
                near_indexer_primitives::views::ReceiptEnumView::Data { data_id, .. } => {
                    crate::ReceiptOrDataId::DataId(data_id)
                }
            };
            if let Some(transaction_hash) = tx_hashes_for_receipts.get(&receipt_or_data_id) {
                Some(models::Receipt::from_receipt_view(
                        r,
                        block_hash,
                        transaction_hash,
                        chunk_hash,
                        index as i32,
                        block_timestamp,
                    ))
            } else {
                warn!(
                    target: crate::INDEXER,
                    "Skipping Receipt {} as we can't find parent Transaction for it. Happen in block hash {}, chunk hash {}",
                    r.receipt_id.to_string(),
                    block_hash,
                    chunk_hash,
                );
                skipping_receipt_ids.insert(r.receipt_id);
                None
            }
        })
        .collect();

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

    save_receipts(pool, receipt_models).await?;

    let (action_receipts, data_receipts): (
        Vec<&near_indexer_primitives::views::ReceiptView>,
        Vec<&near_indexer_primitives::views::ReceiptView>,
    ) = receipts
        .iter()
        .filter(|r| !skipping_receipt_ids.contains(&r.receipt_id))
        .partition(|receipt| {
            matches!(
                receipt.receipt,
                near_indexer_primitives::views::ReceiptEnumView::Action { .. }
            )
        });

    let process_receipt_actions_future =
        store_receipt_actions(pool, action_receipts, block_timestamp);

    let process_receipt_data_future = store_data_receipts(pool, &data_receipts);

    try_join!(process_receipt_actions_future, process_receipt_data_future)?;
    Ok(())
}

/// Looks for already created parent transaction hash for given receipts
async fn find_tx_hashes_for_receipts(
    pool: &sqlx::Pool<sqlx::MySql>,
    mut receipts: Vec<near_indexer_primitives::views::ReceiptView>,
    block_hash: &near_indexer_primitives::CryptoHash,
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

    if !receipts.is_empty() {
        panic!("temp solution, receipts should be empty here");
    }
    Ok(tx_hashes_for_receipts)

    // TODO we still need the logic with getting info from DB
    // if receipts.is_empty() {
    //     return Ok(tx_hashes_for_receipts);
    // }
    //
    // warn!(
    //     target: crate::utils::INDEXER,
    //     "Looking for parent transaction hash in database for {} receipts {:#?}",
    //     &receipts.len(),
    //     &receipts,
    // );
    //
    // let mut retries_left: u8 = 4; // retry at least times even in no-strict mode to avoid data loss
    // let mut find_tx_retry_interval = crate::INTERVAL;
    // loop {
    //     let data_receipt_ids: Vec<String> = receipts
    //         .iter()
    //         .filter_map(|r| match r.receipt {
    //             near_indexer_primitives::views::ReceiptEnumView::Data { data_id, .. } => {
    //                 Some(data_id.to_string())
    //             }
    //             _ => None,
    //         })
    //         .collect();
    //
    //     if !data_receipt_ids.is_empty() {
    //         let mut interval = crate::utils::INTERVAL;
    //         let tx_hashes_for_data_id_via_data_output: Vec<(
    //             crate::ReceiptOrDataId,
    //             crate::ParentTransactionHashString,
    //         )> = loop {
    //             let account = sqlx::query!(
    //     // just pretend "accounts" is a real table
    //     "select * from (select (1) as id, 'Herp Derpinson' as name) accounts where id = ?",
    //     1i32
    // )
    //                 .fetch_one(&mut conn)
    //                 .await?;
    //
    //
    //             let a = r#"
    //                 SELECT action_receipt_output_data.output_data_id, receipts.originated_from_transaction_hash
    //                 FROM action_receipt_output_data JOIN receipts ON action_receipt_output_data.output_from_receipt_id = receipts.receipt_id
    //                 WHERE action_receipt_output_data.output_data_id IN ${data_receipt_ids}
    //             "#;
    //             match schema::action_receipt_output_data::table
    //                 .inner_join(
    //                     schema::receipts::table.on(
    //                         schema::action_receipt_output_data::dsl::output_from_receipt_id
    //                             .eq(schema::receipts::dsl::receipt_id),
    //                     ),
    //                 )
    //                 .filter(
    //                     schema::action_receipt_output_data::dsl::output_data_id
    //                         .eq(any(data_receipt_ids.clone())),
    //                 )
    //                 .select((
    //                     schema::action_receipt_output_data::dsl::output_data_id,
    //                     schema::receipts::dsl::originated_from_transaction_hash,
    //                 ))
    //                 .load_async(pool)
    //                 .await
    //             {
    //                 Ok(res) => {
    //                     break res
    //                         .into_iter()
    //                         .map(
    //                             |(receipt_id_string, transaction_hash_string): (String, String)| {
    //                                 (
    //                                     crate::ReceiptOrDataId::DataId( // NEW FIX!
    //                                         near_indexer_primitives::CryptoHash::from_str(
    //                                             &receipt_id_string,
    //                                         )
    //                                             .expect("Failed to convert String to CryptoHash"),
    //                                     ),
    //                                     transaction_hash_string,
    //                                 )
    //                             },
    //                         )
    //                         .collect();
    //                 }
    //                 Err(async_error) => {
    //                     error!(
    //                         target: crate::utils::INDEXER,
    //                         "Error occurred while fetching the parent receipt for Receipt. Retrying in {} milliseconds... \n {:#?}",
    //                         interval.as_millis(),
    //                         async_error,
    //                     );
    //                     tokio::time::sleep(interval).await;
    //                     if interval < crate::MAX_DELAY_TIME {
    //                         interval *= 2;
    //                     }
    //                 }
    //             }
    //         };
    //     } // this one should not be here
    //         panic!("temp solution");
    //
    //
    //
    //         let mut tx_hashes_for_data_id_via_data_output_hashmap =
    //             HashMap::<crate::ReceiptOrDataId, crate::ParentTransactionHashString>::new();
    //         tx_hashes_for_data_id_via_data_output_hashmap
    //             .extend(tx_hashes_for_data_id_via_data_output);
    //         let tx_hashes_for_receipts_via_data_output: Vec<(
    //             crate::ReceiptOrDataId,
    //             crate::ParentTransactionHashString,
    //         )> = receipts
    //             .iter()
    //             .filter_map(|r| match r.receipt {
    //                 near_indexer_primitives::views::ReceiptEnumView::Data {
    //                     data_id, ..
    //                 } => tx_hashes_for_data_id_via_data_output_hashmap
    //                     .get(&crate::ReceiptOrDataId::DataId(data_id))
    //                     .map(|tx_hash| {
    //                         (
    //                             crate::ReceiptOrDataId::ReceiptId(r.receipt_id),
    //                             tx_hash.to_string(),
    //                         )
    //                     }),
    //                 _ => None,
    //             })
    //             .collect();
    //
    //         let found_hashes_len = tx_hashes_for_receipts_via_data_output.len();
    //         tx_hashes_for_receipts.extend(tx_hashes_for_receipts_via_data_output);
    //
    //         if found_hashes_len == receipts.len() {
    //             break;
    //         }
    //
    //         receipts.retain(|r| {
    //             !tx_hashes_for_receipts
    //                 .contains_key(&crate::ReceiptOrDataId::ReceiptId(r.receipt_id))
    //         });
    //     }
    //
    //
    //
    //
    //
    //
    //
    //
    //
    //     let a = r#"
    //                 SELECT execution_outcome_receipts.produced_receipt_id, receipts.originated_from_transaction_hash
    //                 FROM execution_outcome_receipts JOIN receipts ON execution_outcome_receipts.executed_receipt_id = receipts.receipt_id
    //                 WHERE execution_outcome_receipts.produced_receipt_id IN ${action_receipts}
    //             "#;
    //     let tx_hashes_for_receipts_via_outcomes: Vec<(String, crate::ParentTransactionHashString)> =
    //         crate::await_retry_or_panic!(
    //             schema::execution_outcome_receipts::table
    //                 .inner_join(
    //                     schema::receipts::table
    //                         .on(schema::execution_outcome_receipts::dsl::executed_receipt_id
    //                             .eq(schema::receipts::dsl::receipt_id)),
    //                 )
    //                 .filter(
    //                     schema::execution_outcome_receipts::dsl::produced_receipt_id.eq(any(
    //                         receipts
    //                             .clone()
    //                             .iter()
    //                             .filter(|r| {
    //                                 matches!(
    //                             r.receipt,
    //                             near_indexer::near_primitives::views::ReceiptEnumView::Action { .. }
    //                         )
    //                             })
    //                             .map(|r| r.receipt_id.to_string())
    //                             .collect::<Vec<String>>()
    //                     )),
    //                 )
    //                 .select((
    //                     schema::execution_outcome_receipts::dsl::produced_receipt_id,
    //                     schema::receipts::dsl::originated_from_transaction_hash,
    //                 ))
    //                 .load_async::<(String, crate::ParentTransactionHashString)>(pool),
    //             10,
    //             "Parent Transaction for Receipts were fetched".to_string(),
    //             &receipts
    //         )
    //         .unwrap_or_default();
    //
    //     let found_hashes_len = tx_hashes_for_receipts_via_outcomes.len();
    //     tx_hashes_for_receipts.extend(tx_hashes_for_receipts_via_outcomes.into_iter().map(
    //         |(receipt_id_string, transaction_hash_string)| {
    //             (
    //                 crate::ReceiptOrDataId::ReceiptId(
    //                     near_primitives::hash::CryptoHash::from_str(&receipt_id_string)
    //                         .expect("Failed to convert String to CryptoHash"),
    //                 ),
    //                 transaction_hash_string,
    //             )
    //         },
    //     ));
    //
    //     if found_hashes_len == receipts.len() {
    //         break;
    //     }
    //
    //     receipts.retain(|r| {
    //         !tx_hashes_for_receipts.contains_key(&crate::ReceiptOrDataId::ReceiptId(r.receipt_id))
    //     });
    //
    //
    //
    //
    //     let a = r#"
    //                 SELECT converted_into_receipt_id, transaction_hash
    //                 FROM transactions
    //                 WHERE converted_into_receipt_id IN ${action_receipts}
    //             "#;
    //     let tx_hashes_for_receipt_via_transactions: Vec<(
    //         String,
    //         crate::ParentTransactionHashString,
    //     )> = crate::await_retry_or_panic!(
    //         schema::transactions::table
    //             .filter(
    //                 schema::transactions::dsl::converted_into_receipt_id.eq(any(receipts
    //                     .clone()
    //                     .iter()
    //                     .filter(|r| {
    //                         matches!(
    //                             r.receipt,
    //                             near_indexer::near_primitives::views::ReceiptEnumView::Action { .. }
    //                         )
    //                     })
    //                     .map(|r| r.receipt_id.to_string())
    //                     .collect::<Vec<String>>())),
    //             )
    //             .select((
    //                 schema::transactions::dsl::converted_into_receipt_id,
    //                 schema::transactions::dsl::transaction_hash,
    //             ))
    //             .load_async::<(String, crate::ParentTransactionHashString)>(pool),
    //         10,
    //         "Parent Transaction for ExecutionOutcome were fetched".to_string(),
    //         &receipts
    //     )
    //     .unwrap_or_default();
    //
    //     let found_hashes_len = tx_hashes_for_receipt_via_transactions.len();
    //     tx_hashes_for_receipts.extend(tx_hashes_for_receipt_via_transactions.into_iter().map(
    //         |(receipt_id_string, transaction_hash_string)| {
    //             (
    //                 crate::ReceiptOrDataId::ReceiptId(
    //                     near_primitives::hash::CryptoHash::from_str(&receipt_id_string)
    //                         .expect("Failed to convert String to CryptoHash"),
    //                 ),
    //                 transaction_hash_string,
    //             )
    //         },
    //     ));
    //
    //     //////////
    //
    //
    //
    //     if found_hashes_len == receipts.len() {
    //         break;
    //     }
    //
    //     receipts.retain(|r| {
    //         !tx_hashes_for_receipts.contains_key(&crate::ReceiptOrDataId::ReceiptId(r.receipt_id))
    //     });
    //
    //     if !strict_mode {
    //         if retries_left > 0 {
    //             retries_left -= 1;
    //         } else {
    //             break;
    //         }
    //     }
    //     warn!(
    //         target: crate::utils::INDEXER,
    //         "Going to retry to find parent transactions for receipts in {} milliseconds... \n {:#?}\n block hash {} \nchunk hash {}",
    //         find_tx_retry_interval.as_millis(),
    //         &receipts,
    //         block_hash,
    //         chunk_hash
    //     );
    //     tokio::time::sleep(find_tx_retry_interval).await;
    //     if find_tx_retry_interval < crate::MAX_DELAY_TIME {
    //         find_tx_retry_interval *= 2;
    //     }
    // }
    //
    // Ok(tx_hashes_for_receipts)
}

async fn save_receipts(
    pool: &sqlx::Pool<sqlx::MySql>,
    receipts: Vec<models::Receipt>,
) -> anyhow::Result<()> {
    for receipts_part in receipts.chunks(crate::db_adapters::CHUNK_SIZE_FOR_BATCH_INSERT) {
        let mut args = sqlx::mysql::MySqlArguments::default();
        let mut receipts_count = 0;

        receipts_part.iter().for_each(|receipt| {
            receipt.add_to_args(&mut args);
            receipts_count += 1;
        });

        let query = models::receipts::Receipt::get_query(receipts_count)?;
        sqlx::query_with(&query, args).execute(pool).await?;
    }
    Ok(())
}

async fn store_receipt_actions(
    pool: &sqlx::Pool<sqlx::MySql>,
    receipts: Vec<&near_indexer_primitives::views::ReceiptView>,
    block_timestamp: u64,
) -> anyhow::Result<()> {
    let receipt_actions: Vec<models::ActionReceipt> = receipts
        .iter()
        .filter_map(|receipt| models::ActionReceipt::try_from_action_receipt_view(*receipt).ok())
        .collect();

    let receipt_action_actions: Vec<models::ActionReceiptAction> = receipts
        .iter()
        .filter_map(|receipt| {
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
        .filter_map(|receipt| {
            if let near_indexer_primitives::views::ReceiptEnumView::Action {
                input_data_ids, ..
            } = &receipt.receipt
            {
                Some(input_data_ids.iter().map(move |data_id| {
                    models::ActionReceiptInputData::from_data_id(
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
        .filter_map(|receipt| {
            if let near_indexer_primitives::views::ReceiptEnumView::Action {
                output_data_receivers,
                ..
            } = &receipt.receipt
            {
                Some(output_data_receivers.iter().map(move |receiver| {
                    models::ActionReceiptOutputData::from_data_receiver(
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

    // TODO should we rename the tables? We may do that now (while migration goes) or never
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
    receipts: &[&near_indexer_primitives::views::ReceiptView],
) -> anyhow::Result<()> {
    for data_receipts_part in &receipts
        .iter()
        .filter_map(|receipt| models::DataReceipt::try_from_data_receipt_view(*receipt).ok())
        .chunks(crate::db_adapters::CHUNK_SIZE_FOR_BATCH_INSERT)
    {
        let mut args = sqlx::mysql::MySqlArguments::default();
        let mut data_receipts_count = 0;

        data_receipts_part.for_each(|data_receipt| {
            data_receipt.add_to_args(&mut args);
            data_receipts_count += 1;
        });

        let query = models::receipts::DataReceipt::get_query(data_receipts_count)?;
        sqlx::query_with(&query, args).execute(pool).await?;
    }

    Ok(())
}
