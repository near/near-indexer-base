use crate::models;

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

    for account_changes_part in &state_changes
        .iter()
        .filter_map(|state_change| {
            models::account_changes::AccountChange::from_state_change_with_cause(
                state_change,
                block_hash,
                block_timestamp,
                0, // we will fill it later
            )
        })
        .enumerate()
        .chunks(crate::db_adapters::CHUNK_SIZE_FOR_BATCH_INSERT)
    {
        let mut args = sqlx::mysql::MySqlArguments::default();
        let mut changes_count = 0;

        account_changes_part.for_each(|(i, mut account_change)| {
            account_change.index_in_block = i as i32;
            account_change.add_to_args(&mut args);
            changes_count += 1;
        });

        let query = models::account_changes::AccountChange::get_query(changes_count)?;
        sqlx::query_with(&query, args).execute(pool).await?;
    }

    Ok(())
}
