use crate::{batch_insert, models};

use itertools::Itertools;

// todo recheck first block on mainnet. Looks like we miss the data in S3
pub(crate) async fn store_account_changes(
    pool: &sqlx::Pool<sqlx::MySql>,
    state_changes: &near_indexer_primitives::views::StateChangesView,
    block_hash: &near_indexer_primitives::CryptoHash,
    block_timestamp: u64,
) -> anyhow::Result<()> {
    if state_changes.is_empty() {
        return Ok(());
    }

    let account_changes_models: Vec<models::account_changes::AccountChange> = state_changes
        .iter()
        .enumerate()
        .filter_map(|(index_in_block, state_change)| {
            models::account_changes::AccountChange::from_state_change_with_cause(
                state_change,
                block_hash,
                block_timestamp,
                index_in_block as i32,
            )
        })
        .collect();

    batch_insert!(&pool.clone(), "INSERT INTO account_changes VALUES {}", account_changes_models);
    Ok(())
}
