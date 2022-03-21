use std::collections::HashMap;
use std::convert::TryFrom;

use anyhow::Context;
use bigdecimal::BigDecimal;
use futures::try_join;

use tracing::info;

use crate::{batch_insert, models, run_query};

/// Saves new Accounts to database or deletes the ones should be deleted
pub(crate) async fn handle_accounts(
    pool: &sqlx::Pool<sqlx::MySql>,
    outcomes: &[near_indexer_primitives::IndexerExecutionOutcomeWithReceipt],
    block_height: near_indexer_primitives::types::BlockHeight,
) -> anyhow::Result<()> {
    if outcomes.is_empty() {
        return Ok(());
    }

    let successful_receipts = models::execution_outcomes::ExecutionOutcome::get_successful_receipts(outcomes);

    let mut accounts =
        HashMap::<near_indexer_primitives::types::AccountId, models::accounts::Account>::new();

    for receipt in successful_receipts {
        if let near_indexer_primitives::views::ReceiptEnumView::Action { actions, .. } = &receipt.receipt {
            for action in actions {
                match action {
                    near_indexer_primitives::views::ActionView::CreateAccount => {
                        accounts.insert(
                            receipt.receiver_id.clone(),
                            models::accounts::Account::new_from_receipt(
                                &receipt.receiver_id,
                                &receipt.receipt_id,
                                block_height,
                            ),
                        );
                    }
                    near_indexer_primitives::views::ActionView::Transfer { .. } => {
                        if receipt.receiver_id.len() == 64usize {
                            // TODO add select if such account exists
                            accounts.insert(
                                receipt.receiver_id.clone(),
                                models::accounts::Account::new_from_receipt(
                                    &receipt.receiver_id,
                                    &receipt.receipt_id,
                                    block_height,
                                ),
                            );
                        }
                    }
                    near_indexer_primitives::views::ActionView::DeleteAccount { .. } => {
                        accounts
                            .entry(receipt.receiver_id.clone())
                            .and_modify(|existing_account| {
                                existing_account.deleted_by_receipt_id =
                                    Some(receipt.receipt_id.to_string());
                                existing_account.deleted_at_block_height = Some(block_height.into());
                            })
                            .or_insert_with(|| models::accounts::Account {
                                account_id: receipt.receiver_id.to_string(),
                                created_at_block_height: Default::default(),
                                deleted_at_block_height: block_height.into(),
                                created_by_receipt_id: None,
                                deleted_by_receipt_id: Some(receipt.receipt_id.to_string()),
                            });
                    }
                    _ => {}
                }
            }
        }
    }

    // created none, deleted some: update existing acc
    // created some, deleted none: insert new acc
    // created some, deleted some: insert new acc that was immediately deleted (not sure it's possible, but anyway)
    let (new_accounts_to_insert, existing_accounts_to_update): (
        Vec<models::accounts::Account>,
        Vec<models::accounts::Account>,
    ) = accounts
        .values()
        .cloned()
        .partition(|model| model.created_by_receipt_id.is_some());

    let delete_accounts_future = async {
        for value in existing_accounts_to_update {
            let q = format!(
                "UPDATE accounts SET deleted_at_block_height = '{0}', deleted_by_receipt_id = '{1}'\n
                 WHERE account_id = '{2}' AND deleted_by_receipt_id is NULL",
                value.deleted_at_block_height.unwrap(), value.deleted_by_receipt_id.unwrap(), value.account_id);
            run_query!(&pool.clone(), &q);
        }
        Ok(())
    };

    let create_accounts_future = async {
        batch_insert!(
        &pool.clone(),
        "INSERT INTO accounts VALUES {}",
        new_accounts_to_insert
    );
        Ok(())
    };

    // Joining it unless we can't execute it in the correct order
    // see https://github.com/nearprotocol/nearcore/issues/3467
    try_join!(delete_accounts_future, create_accounts_future)?;
    Ok(())
}

// pub(crate) async fn store_accounts_from_genesis(
//     pool: Database<PgConnection>,
//     accounts_models: Vec<models::accounts::Account>,
// ) -> anyhow::Result<()> {
//     info!(
//         target: crate::INDEXER_FOR_EXPLORER,
//         "Adding/updating accounts from genesis..."
//     );
//
//     crate::await_retry_or_panic!(
//         diesel::insert_into(schema::accounts::table)
//             .values(accounts_models.clone())
//             .on_conflict_do_nothing()
//             .execute_async(&pool),
//         10,
//         "Accounts were stored from genesis".to_string(),
//         &accounts_models
//     );
//
//     Ok(())
// }
