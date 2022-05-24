pub(crate) async fn store_block(
    pool: &sqlx::Pool<sqlx::Postgres>,
    block: &near_indexer_primitives::views::BlockView,
) -> anyhow::Result<()> {
    crate::models::chunked_insert(
        pool,
        &vec![crate::models::blocks::Block::from_block_view(block)],
        10,
    )
    .await?;
    Ok(())
}
