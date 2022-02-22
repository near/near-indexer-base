use crate::{batch_insert, models};

use itertools::Itertools;

/// Saves block to database
pub(crate) async fn store_block(
    pool: &sqlx::Pool<sqlx::MySql>,
    block: &near_indexer_primitives::views::BlockView,
) -> anyhow::Result<()> {
    let block_model = models::blocks::Block::from(block);
    // TODO now it fails if it tries to insert already inserted line. Think how to act better
    batch_insert!(
        &pool.clone(),
        "INSERT INTO blocks VALUES {}",
        vec![&block_model]
    )
}

// /// Gets the latest block's height from database
// pub(crate) async fn latest_block_height(
//     pool: &actix_diesel::Database<PgConnection>,
// ) -> Result<Option<u64>, String> {
//     tracing::debug!(target: crate::INDEXER_FOR_EXPLORER, "fetching latest");
//     Ok(schema::blocks::table
//         .select((schema::blocks::dsl::block_height,))
//         .order(schema::blocks::dsl::block_height.desc())
//         .limit(1)
//         .get_optional_result_async::<(bigdecimal::BigDecimal,)>(pool)
//         .await
//         .map_err(|err| format!("DB Error: {}", err))?
//         .and_then(|(block_height,)| block_height.to_u64()))
// }
//
// pub(crate) async fn get_latest_block_before_timestamp(
//     pool: &actix_diesel::Database<PgConnection>,
//     timestamp: u64,
// ) -> anyhow::Result<models::Block> {
//     Ok(schema::blocks::table
//         .filter(schema::blocks::dsl::block_timestamp.le(BigDecimal::from(timestamp)))
//         .order(schema::blocks::dsl::block_timestamp.desc())
//         .first_async::<models::Block>(pool)
//         .await
//         .context("DB Error")?)
// }
