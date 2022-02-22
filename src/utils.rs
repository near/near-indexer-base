#[macro_export]
macro_rules! batch_insert {
    ($pool: expr, $query: expr, $rows: expr $(,)?) => {{
        if $rows.is_empty() {
            return Ok(());
        }

        // TODO find a better way to insert the objects to the DB
        let values = $rows.iter().map(|item| item.to_string()).join(", ");
        sqlx::query(&format!($query, values)).execute($pool).await?;
        Ok(())
    }};
}
