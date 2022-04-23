use crate::models;

pub(crate) async fn store_block(
    pool: &sqlx::Pool<sqlx::MySql>,
    block: &near_indexer_primitives::views::BlockView,
) -> anyhow::Result<()> {
    crate::models::chunked_insert(
        pool,
        &vec![models::blocks::Block::from_block_view(block)],
        10,
    )
    .await?;
    Ok(())
}
