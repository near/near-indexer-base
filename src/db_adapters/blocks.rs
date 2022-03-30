use crate::models;

pub(crate) async fn store_block(
    pool: &sqlx::Pool<sqlx::MySql>,
    block: &near_indexer_primitives::views::BlockView,
) -> anyhow::Result<()> {
    let mut args = sqlx::mysql::MySqlArguments::default();
    models::blocks::Block::from_block_view(block).add_to_args(&mut args);
    let query = models::blocks::Block::get_query(1)?;
    sqlx::query_with(&query, args).execute(pool).await?;
    Ok(())
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
