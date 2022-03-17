use crate::models::PrintEnum;
use crate::{batch_insert, models, run_query};

use std::collections::HashMap;
use std::convert::TryFrom;

use bigdecimal::BigDecimal;
use futures::try_join;
// TODO it is required my macro. Maybe put it inside the macro?
use itertools::Itertools;
use near_indexer_primitives::views::AccessKeyPermissionView;

pub(crate) async fn handle_access_keys(
    pool: &sqlx::Pool<sqlx::MySql>,
    outcomes: &[near_indexer_primitives::IndexerExecutionOutcomeWithReceipt],
    block_height: near_indexer_primitives::types::BlockHeight,
) -> anyhow::Result<()> {
    if outcomes.is_empty() {
        return Ok(());
    }
    let successful_receipts = outcomes
        .iter()
        .filter(|outcome_with_receipt| {
            matches!(
                outcome_with_receipt.execution_outcome.outcome.status,
                near_indexer_primitives::views::ExecutionStatusView::SuccessValue(_)
                    | near_indexer_primitives::views::ExecutionStatusView::SuccessReceiptId(_)
            )
        })
        .map(|outcome_with_receipt| &outcome_with_receipt.receipt);

    let mut access_keys = HashMap::<(String, String), models::access_keys::AccessKey>::new();
    let mut deleted_accounts = HashMap::<String, String>::new();

    for receipt in successful_receipts {
        if let near_indexer_primitives::views::ReceiptEnumView::Action { actions, .. } =
            &receipt.receipt
        {
            for action in actions {
                match action {
                    near_indexer_primitives::views::ActionView::DeleteAccount { .. } => {
                        deleted_accounts.insert(
                            receipt.receiver_id.to_string(),
                            receipt.receipt_id.to_string(),
                        );
                        access_keys
                            .iter_mut()
                            .filter(|((_, receiver_id), _)| {
                                receiver_id == receipt.receiver_id.as_ref()
                            })
                            .for_each(|(_, access_key)| {
                                access_key.deleted_by_receipt_id =
                                    Some(receipt.receipt_id.to_string());
                            });
                    }
                    near_indexer_primitives::views::ActionView::AddKey {
                        public_key,
                        access_key,
                    } => {
                        access_keys.insert(
                            (public_key.to_string(), receipt.receiver_id.to_string()),
                            models::access_keys::AccessKey::from_action_view(
                                public_key,
                                &receipt.receiver_id,
                                access_key,
                                &receipt.receipt_id,
                                block_height,
                            ),
                        );
                    }
                    near_indexer_primitives::views::ActionView::DeleteKey { public_key } => {
                        access_keys
                            .entry((public_key.to_string(), receipt.receiver_id.to_string()))
                            .and_modify(|existing_access_key| {
                                existing_access_key.deleted_by_receipt_id =
                                    Some(receipt.receipt_id.to_string());
                            })
                            .or_insert_with(|| models::access_keys::AccessKey {
                                public_key: public_key.to_string(),
                                account_id: receipt.receiver_id.to_string(),
                                created_by_receipt_id: None,
                                deleted_by_receipt_id: Some(receipt.receipt_id.to_string()),
                                // this is a workaround to avoid additional struct with optional field
                                // permission_kind is not supposed to change on delete action
                                permission_kind: AccessKeyPermissionView::FullAccess
                                    .print()
                                    .to_string(), //models::enums::AccessKeyPermission::FullAccess,
                                last_update_block_height: block_height.into(),
                            });
                    }
                    near_indexer_primitives::views::ActionView::Transfer { .. } => {
                        if receipt.receiver_id.len() != 64usize {
                            continue;
                        }
                        if let Ok(public_key_bytes) = hex::decode(receipt.receiver_id.as_ref()) {
                            if let Ok(public_key) =
                                near_crypto::ED25519PublicKey::try_from(&public_key_bytes[..])
                            {
                                access_keys.insert(
                                    (near_crypto::PublicKey::from(public_key.clone()).to_string(), receipt.receiver_id.to_string()),
                                    models::access_keys::AccessKey::from_action_view(
                                        &near_crypto::PublicKey::from(public_key.clone()),
                                        &receipt.receiver_id,
                                        &near_indexer_primitives::views::AccessKeyView {
                                            nonce: 0,
                                            permission: near_indexer_primitives::views::AccessKeyPermissionView::FullAccess
                                        },
                                        &receipt.receipt_id,
                                        block_height,
                                    ),
                                );
                            }
                        }
                    }
                    _ => continue,
                }
            }
        }
    }

    let (access_keys_to_insert, access_keys_to_update): (
        Vec<models::access_keys::AccessKey>,
        Vec<models::access_keys::AccessKey>,
    ) = access_keys
        .values()
        .cloned()
        .partition(|model| model.created_by_receipt_id.is_some());

    try_join!(
        delete_access_keys_for_deleted_accounts(pool, block_height.into(), &deleted_accounts),
        update_access_keys(pool, &access_keys_to_update),
        add_access_keys(pool, &access_keys_to_insert),
    )?;

    Ok(())
}

// pub(crate) async fn store_access_keys_from_genesis(
//     pool: Database<PgConnection>,
//     access_keys_models: Vec<models::access_keys::AccessKey>,
// ) -> anyhow::Result<()> {
//     info!(
//         target: crate::INDEXER_FOR_EXPLORER,
//         "Adding/updating access keys from genesis..."
//     );
//
//     crate::await_retry_or_panic!(
//         diesel::insert_into(schema::access_keys::table)
//             .values(access_keys_models.clone())
//             .on_conflict_do_nothing()
//             .execute_async(&pool),
//         10,
//         "AccessKeys were stored from genesis".to_string(),
//         &access_keys_models
//     );
//     Ok(())
// }

async fn delete_access_keys_for_deleted_accounts(
    pool: &sqlx::Pool<sqlx::MySql>,
    last_update_block_height: BigDecimal,
    deleted_accounts: &HashMap<String, String>,
) -> anyhow::Result<()> {
    for (account_id, deleted_by_receipt_id) in deleted_accounts {
        let q = format!(
            "UPDATE access_keys SET deleted_by_receipt_id = '{0}', last_update_block_height = '{1}'\n
                 WHERE deleted_by_receipt_id is NULL AND\n
                     last_update_block_height < '{1}' AND\n
                     account_id = '{2}'", deleted_by_receipt_id, last_update_block_height, account_id);
        run_query!(&pool.clone(), &q);
    }
    Ok(())
}

async fn update_access_keys(
    pool: &sqlx::Pool<sqlx::MySql>,
    access_keys_to_update: &Vec<models::access_keys::AccessKey>,
) -> anyhow::Result<()> {
    for value in access_keys_to_update {
        let q = format!(
            "UPDATE access_keys SET deleted_by_receipt_id = '{0}', last_update_block_height = '{1}'\n
             WHERE public_key = '{2}' AND\n
                 last_update_block_height < '{1}' AND\n
                 account_id = '{3}'", value.deleted_by_receipt_id.as_ref().unwrap_or(&"NULL".to_string()), value.last_update_block_height, value.public_key, value.account_id);
        run_query!(&pool.clone(), &q);
    }
    Ok(())
}

async fn add_access_keys(
    pool: &sqlx::Pool<sqlx::MySql>,
    access_keys_to_insert: &Vec<models::access_keys::AccessKey>,
) -> anyhow::Result<()> {
    batch_insert!(
        &pool.clone(),
        "INSERT INTO access_keys VALUES {}",
        access_keys_to_insert
    );

    // TODO we could update the values on rust side and don't abuse the DB with this update
    for value in access_keys_to_insert {
        let q = format!(
            "UPDATE access_keys SET created_by_receipt_id = '{0}', deleted_by_receipt_id = '{1}', last_update_block_height = '{2}'\n
             WHERE public_key = '{3}' AND\n
                 last_update_block_height < '{2}' AND\n
                 account_id = '{4}'",
            value.created_by_receipt_id.as_ref().unwrap_or(&"NULL".to_string()),
            value.deleted_by_receipt_id.as_ref().unwrap_or(&"NULL".to_string()),
            value.last_update_block_height,
            value.public_key,
            value.account_id);
        run_query!(&pool.clone(), &q);
    }
    Ok(())
}
